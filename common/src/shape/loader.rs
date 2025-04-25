use bevy::{
    asset::Assets, color::Color, ecs::system::{Commands, ResMut}, math::NormedVectorSpace, pbr::{ExtendedMaterial, PbrBundle, StandardMaterial}, prelude::*, render::mesh::Mesh, tasks::{block_on, futures_lite::future}, transform::components::Transform, utils::{HashMap, HashSet}
};
use bevy_rustysynth::{MidiAudio, MidiNote};

use std::{
    collections::VecDeque,
    f32::consts::FRAC_PI_2,
    fs::{self, File},
    hash::{DefaultHasher, Hash, Hasher},
    usize,
};

use petgraph::{graphmap::GraphMap, Directed};

use crate::{
    assets::{
        material_handles::FaceMaterialHandles,
        mesh_generators::{
            FaceMeshGenerator, PentagonFaceMeshGenerator, SquareFaceMeshGenerator,
            TriangleFaceMeshGenerator,
        },
        mesh_handles::MeshHandles,
        shaders::GlobalShader,
    }, constants::{SQRT_3, TAN_27}, game_save::CurrentPuzzle, game_settings::{FaceColorPalette, GameSettings}, game_state::{GameState, PlayState}, is_room_junction::is_junction, levels::{GameLevel, LevelData, Shape}, load_level_asset::{DailyLevelLoadError, LoadedLevels, MazeSaveDataHandle}, maze::{border_type::BorderType, mesh}, player::{Player, PlayerMazeState}, room::{Edge, Face, Room}, sound::{MelodyPuzzleTracker, Note, NoteMapping}
};

use super::{cube, dodecahedron, icosahedron, octahedron, tetrahedron};
use crate::assets::material_handles::MaterialHandles;

use serde::{Deserialize, Serialize};

#[derive(Component)]
pub struct GraphComponent(pub GraphMap<Room, Edge, Directed>);

#[derive(Component)]
pub struct SolutionComponent(pub Vec<Room>);

#[derive(Serialize, Deserialize, Clone)]
pub struct EncryptedMelody {
    pub encrypted_melody_bytes: Vec<u8>,
    pub melody_length: usize,
}

#[derive(Serialize, Deserialize, Asset, TypePath, Clone)]
pub struct MazeLevelData {
    pub shape: Shape,
    pub nodes_per_edge: u8,
    pub graph: GraphMap<Room, Edge, Directed>,
    pub solution: Vec<Room>,
    pub node_id_to_note: HashMap<u64, Note>,
    pub encrypted_melody: Option<EncryptedMelody>,
}

pub fn despawn_level_data(mut commands: Commands, level_entities: Query<Entity, With<LevelData>>) {
    for entity in level_entities.iter() {
        commands.entity(entity).despawn_recursive();
    }
}

pub fn spawn_level_data(
    current_level_index_query: Query<&CurrentPuzzle>,
    mut commands: Commands,
    mut play_state: ResMut<NextState<PlayState>>,
    mut game_state: ResMut<NextState<GameState>>,
    maze_save_data_assets: Res<Assets<MazeLevelData>>,
    mut loaded_levels: ResMut<LoadedLevels>,
    asset_server: Res<AssetServer>,
) {
    let CurrentPuzzle(puzzle_identifier) = current_level_index_query.single();

    let Some(maze_save_data_handle) = loaded_levels.0.get_mut(puzzle_identifier) else {
        return;
    };

    let level_load_state: Option<Result<MazeLevelData, DailyLevelLoadError>> = match maze_save_data_handle {
        MazeSaveDataHandle::LocalLevel(handle) => maze_save_data_assets.get(handle).cloned().map(|val| Ok(val)),
        MazeSaveDataHandle::RemoteLevel(task) => block_on(future::poll_once(task)),
    };

    let Some(level_load_result) = level_load_state else {
        return;
    };

    let Ok(MazeLevelData {
        shape,
        nodes_per_edge,
        graph,
        solution,
        node_id_to_note,
        encrypted_melody,
    }) = level_load_result else {
        println!("Error loading remote level");
        game_state.set(GameState::Selector);
        return;
    };

    let note_midi_handle = node_id_to_note
        .into_iter()
        .map(|(node_id, note)| {
            let midi_note = note.clone().into();
            let audio = MidiAudio::Sequence(vec![midi_note]);
            let audio_handle = asset_server.add::<MidiAudio>(audio);
            (node_id, (audio_handle, note.clone()))
        })
        .collect::<HashMap<u64, (Handle<MidiAudio>, Note)>>();

    if let Some(EncryptedMelody {
        encrypted_melody_bytes,
        melody_length,
    }) = encrypted_melody
    {
        let room_ids = VecDeque::with_capacity(melody_length);
        commands.spawn((
            MelodyPuzzleTracker {
                room_ids,
                encrypted_melody_bytes: encrypted_melody_bytes.clone(),
            },
            LevelData,
        ));
    }

    let shape = shape.clone();
    // TODO: perhaps think about how not to duplicate the data here.
    commands.spawn((
        LevelData,
        GameLevel {
            shape,
            nodes_per_edge,
        },
        GraphComponent(graph.clone()),
        SolutionComponent(solution.clone()),
        NoteMapping(note_midi_handle),
    ));
    play_state.set(PlayState::Playing);
}

pub fn spawn_mesh(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mesh_handles: Res<MeshHandles>,
    level_query: Query<&GameLevel>,
    material_handles: Res<MaterialHandles>,
) {
    let Ok(level) = level_query.get_single() else {
        return;
    };

    let face_materials_handles = &material_handles.face_handles;

    let materials: Vec<Handle<ExtendedMaterial<StandardMaterial, GlobalShader>>> =
        match &level.shape {
            Shape::Cube => face_materials_handles.cube().into_iter().collect(),
            Shape::Tetrahedron => face_materials_handles.tetrahedron().into_iter().collect(),
            Shape::Octahedron => face_materials_handles.octahedron().into_iter().collect(),
            Shape::Dodecahedron => face_materials_handles.dodecahedron().into_iter().collect(),
            Shape::Icosahedron => face_materials_handles.icosahedron().into_iter().collect(),
        };

    let face_mesh_handles = match &level.shape {
        Shape::Tetrahedron => mesh_handles.shape_mesh_handles.tetrahedron.to_vec(),
        Shape::Cube => mesh_handles.shape_mesh_handles.cube.to_vec(),
        Shape::Octahedron => mesh_handles.shape_mesh_handles.octahedron.to_vec(),
        Shape::Dodecahedron => mesh_handles.shape_mesh_handles.dodecahedron.to_vec(),
        Shape::Icosahedron => mesh_handles.shape_mesh_handles.icosahedron.to_vec(),
    };

    for (face_mesh_handle, face_material_handle) in
        face_mesh_handles.into_iter().zip(materials.into_iter())
    {
        commands
            .spawn(Mesh3d(face_mesh_handle.clone()))
            .insert(MeshMaterial3d(face_material_handle))
            .insert(LevelData);
    }
}
