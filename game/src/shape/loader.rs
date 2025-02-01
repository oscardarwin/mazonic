use bevy::{
    asset::Assets,
    color::Color,
    ecs::system::{Commands, ResMut},
    math::NormedVectorSpace,
    pbr::{ExtendedMaterial, PbrBundle, StandardMaterial},
    prelude::*,
    render::mesh::Mesh,
    transform::components::Transform,
    utils::{HashMap, HashSet},
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
    },
    constants::{SQRT_3, TAN_27},
    game_save::CurrentLevelIndex,
    game_settings::{FaceColorPalette, GameSettings},
    game_state::PlayState,
    is_room_junction::is_junction,
    levels::{GameLevel, LevelData, Shape},
    maze::{border_type::BorderType, mesh},
    player::{Player, PlayerMazeState},
    room::{Edge, Face, Room},
    sound::{MelodyPuzzleTracker, Note, NoteMapping},
};

use super::{cube, dodecahedron, icosahedron, octahedron, tetrahedron};
use crate::assets::material_handles::MaterialHandles;
use crate::levels::LEVELS;

use serde::{Deserialize, Serialize};

#[derive(Component)]
pub struct GraphComponent(pub GraphMap<Room, Edge, Directed>);

#[derive(Component)]
pub struct SolutionComponent(pub Vec<Room>);

#[derive(Serialize, Deserialize)]
pub struct EncryptedMelody {
    pub encrypted_melody_bytes: Vec<u8>,
    pub melody_length: usize,
}

#[derive(Serialize, Deserialize, Asset, TypePath)]
pub struct MazeLevelData {
    pub graph: GraphMap<Room, Edge, Directed>,
    pub solution: Vec<Room>,
    pub node_id_to_note: HashMap<u64, Note>,
    pub encrypted_melody: Option<EncryptedMelody>,
}

#[derive(Component)]
pub struct MazeSaveDataHandle(Handle<MazeLevelData>);

pub fn despawn_level_data(mut commands: Commands, level_entities: Query<Entity, With<LevelData>>) {
    for entity in level_entities.iter() {
        commands.entity(entity).despawn_recursive();
    }
}

pub fn load_level_asset(
    mut commands: Commands,
    current_level_index_query: Query<&CurrentLevelIndex>,
    mut game_state: ResMut<NextState<PlayState>>,
    asset_server: Res<AssetServer>,
) {
    let CurrentLevelIndex(current_level_index) = current_level_index_query.single();

    let level = &LEVELS[*current_level_index];

    let file_path = level.filename();

    let maze_save_data_handle = asset_server.load::<MazeLevelData>(file_path);

    let distance_between_nodes = level.node_distance();

    commands.spawn((
        level.clone(),
        MazeSaveDataHandle(maze_save_data_handle),
        LevelData,
    ));
}

pub fn spawn_level_data(
    mut commands: Commands,
    mut game_state: ResMut<NextState<PlayState>>,
    maze_save_data_assets: Res<Assets<MazeLevelData>>,
    asset_server: Res<AssetServer>,
    maze_save_data_query: Query<&MazeSaveDataHandle>,
) {
    let MazeSaveDataHandle(maze_save_data_handle) = maze_save_data_query.single();

    let Some(MazeLevelData {
        graph,
        solution,
        node_id_to_note,
        encrypted_melody,
    }) = maze_save_data_assets.get(maze_save_data_handle)
    else {
        return;
    };

    println!("Loading Maze");

    let note_midi_handle = node_id_to_note
        .into_iter()
        .map(|(node_id, note)| {
            let midi_note = note.clone().into();
            let audio = MidiAudio::Sequence(vec![midi_note]);
            let audio_handle = asset_server.add::<MidiAudio>(audio);
            (*node_id, (audio_handle, note.clone()))
        })
        .collect::<HashMap<u64, (Handle<MidiAudio>, Note)>>();

    if let Some(EncryptedMelody {
        encrypted_melody_bytes,
        melody_length,
    }) = encrypted_melody
    {
        let room_ids = VecDeque::with_capacity(*melody_length);
        commands.spawn((
            MelodyPuzzleTracker {
                room_ids,
                encrypted_melody_bytes: encrypted_melody_bytes.clone(),
            },
            LevelData,
        ));
    }

    // TODO: perhaps think about how not to duplicate the data here.
    commands.spawn((
        LevelData,
        GraphComponent(graph.clone()),
        SolutionComponent(solution.clone()),
        NoteMapping(note_midi_handle),
    ));
    game_state.set(PlayState::Playing);
}

pub fn spawn_mesh(
    mut commands: Commands,
    mesh_handles: Res<MeshHandles>,
    level_query: Query<&GameLevel>,
    asset_handles: Res<MaterialHandles>,
) {
    let Ok(level) = level_query.get_single() else {
        return;
    };

    let face_materials_handles = &asset_handles.face_handles;

    let materials: Vec<Handle<ExtendedMaterial<StandardMaterial, GlobalShader>>> =
        match &level.shape {
            Shape::Cube => face_materials_handles.cube().into_iter().collect(),
            Shape::Tetrahedron => face_materials_handles.tetrahedron().into_iter().collect(),
            Shape::Octahedron => face_materials_handles.octahedron().into_iter().collect(),
            Shape::Dodecahedron => face_materials_handles.dodecahedron().into_iter().collect(),
            Shape::Icosahedron => face_materials_handles.icosahedron().into_iter().collect(),
        };

    let face_mesh_handles = match &level.shape {
        Shape::Cube => &mesh_handles.shapes.cube.faces,
        Shape::Tetrahedron => &mesh_handles.shapes.tetrahedron.faces,
        Shape::Octahedron => &mesh_handles.shapes.octahedron.faces,
        Shape::Dodecahedron => &mesh_handles.shapes.dodecahedron.faces,
        Shape::Icosahedron => &mesh_handles.shapes.icosahedron.faces,
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
