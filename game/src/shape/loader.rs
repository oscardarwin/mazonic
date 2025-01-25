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
    f32::consts::FRAC_PI_2,
    fs::{self, File},
    hash::{DefaultHasher, Hash, Hasher},
    usize,
};

use petgraph::{graphmap::GraphMap, Directed};

use crate::{
    assets::{
        materials::FaceMaterialHandles,
        mesh_generators::{
            FaceMeshGenerator, PentagonFaceMeshGenerator, SquareFaceMeshGenerator,
            TriangleFaceMeshGenerator,
        },
        shaders::ShapeFaceMaterial,
    },
    constants::{SQRT_3, TAN_27},
    game_settings::{FaceColorPalette, GameSettings},
    game_state::PlayState,
    is_room_junction::is_junction,
    level_selector::SaveData,
    player::{Player, PlayerMazeState},
    room::{Face, Room},
    sound::{Note, NoteMapping},
};

use super::{
    cube, dodecahedron, icosahedron, octahedron,
    platonic_mesh_builder::MazeMeshBuilder,
    shape_loader::{BorderType, Edge},
    tetrahedron,
};
use crate::assets::materials::GameMaterialHandles;
use crate::levels::LEVELS;

use serde::{Deserialize, Serialize};

#[derive(Component)]
pub struct GraphComponent(pub GraphMap<Room, Edge, Directed>);

#[derive(Component)]
pub struct SolutionComponent(pub Vec<Room>);

#[derive(Serialize, Deserialize, Asset, TypePath)]
pub struct MazeLevelData {
    pub graph: GraphMap<Room, Edge, Directed>,
    pub solution: Vec<Room>,
    pub node_id_to_note: HashMap<u64, Note>,
    //pub encrypted_song: Vec<u8>,
    //pub song_melody_length: u8,
}

#[derive(Component)]
pub struct LevelData;

#[derive(Clone, Debug)]
pub enum Shape {
    Cube,
    Tetrahedron,
    Icosahedron,
    Octahedron,
    Dodecahedron,
}

impl Shape {
    pub fn get_face_meshes(&self) -> Vec<Mesh> {
        match self {
            Shape::Cube => SquareFaceMeshGenerator::get_face_meshes(cube::faces()),
            Shape::Tetrahedron => TriangleFaceMeshGenerator::get_face_meshes(tetrahedron::faces()),
            Shape::Octahedron => TriangleFaceMeshGenerator::get_face_meshes(octahedron::faces()),
            Shape::Dodecahedron => {
                PentagonFaceMeshGenerator::get_face_meshes(dodecahedron::faces())
            }
            Shape::Icosahedron => TriangleFaceMeshGenerator::get_face_meshes(icosahedron::faces()),
        }
    }
}

#[derive(Component, Clone, Debug)]
pub struct GameLevel {
    pub seed: u64,
    pub shape: Shape,
    pub nodes_per_edge: u8,
}

impl GameLevel {
    const fn new(seed: u64, shape: Shape, nodes_per_edge: u8) -> Self {
        GameLevel {
            seed,
            shape,
            nodes_per_edge,
        }
    }

    pub fn border_type(&self, from: &Face, to: &Face) -> Option<BorderType> {
        let from_vertex_set = self.get_face_indices(from);
        let to_vertex_set = self.get_face_indices(to);

        match from_vertex_set.intersection(&to_vertex_set).count() {
            0 | 1 => None,
            2 => Some(BorderType::Connected),
            _ => Some(BorderType::SameFace),
        }
    }

    fn get_face_indices(&self, face: &Face) -> HashSet<usize> {
        let indices = match self.shape {
            Shape::Tetrahedron => tetrahedron::FACE_INDICES[face.id()].to_vec(),
            Shape::Cube => cube::FACE_INDICES[face.id()].to_vec(),
            Shape::Octahedron => octahedron::FACE_INDICES[face.id()].to_vec(),
            Shape::Dodecahedron => dodecahedron::FACE_INDICES[face.id()].to_vec(),
            Shape::Icosahedron => icosahedron::FACE_INDICES[face.id()].to_vec(),
        };

        indices.into_iter().collect()
    }

    pub fn node_distance(&self) -> f32 {
        match &self.shape {
            Shape::Cube => 1.0 / (self.nodes_per_edge as f32),
            Shape::Tetrahedron => 1.0 / (self.nodes_per_edge as f32 - 1.0 + SQRT_3),
            Shape::Octahedron => 1.0 / (self.nodes_per_edge as f32 - 1.0 + SQRT_3),
            Shape::Dodecahedron => TAN_27,
            Shape::Icosahedron => 1.0 / (self.nodes_per_edge as f32 - 1.0 + SQRT_3),
        }
    }

    pub fn get_maze_mesh_builder(&self) -> MazeMeshBuilder {
        let distance_between_nodes = self.node_distance();

        match self.shape {
            Shape::Cube => MazeMeshBuilder::cube(distance_between_nodes),
            Shape::Tetrahedron => MazeMeshBuilder::tetrahedron(distance_between_nodes),
            Shape::Octahedron => MazeMeshBuilder::octahedron(distance_between_nodes),
            Shape::Dodecahedron => MazeMeshBuilder::dodecahedron(distance_between_nodes),
            Shape::Icosahedron => MazeMeshBuilder::icosahedron(distance_between_nodes),
        }
    }

    pub fn get_face_materials(
        &self,
        face_materials_handles: &FaceMaterialHandles,
    ) -> Vec<Handle<ExtendedMaterial<StandardMaterial, ShapeFaceMaterial>>> {
        match &self.shape {
            Shape::Cube => face_materials_handles.cube().into_iter().collect(),
            Shape::Tetrahedron => face_materials_handles.tetrahedron().into_iter().collect(),
            Shape::Octahedron => face_materials_handles.octahedron().into_iter().collect(),
            Shape::Dodecahedron => face_materials_handles.dodecahedron().into_iter().collect(),
            Shape::Icosahedron => face_materials_handles.icosahedron().into_iter().collect(),
        }
    }

    pub fn get_face_meshes(&self) -> Vec<Mesh> {
        self.shape.get_face_meshes()
    }

    pub const fn tetrahedron(nodes_per_edge: u8, seed: u64) -> GameLevel {
        let shape = Shape::Tetrahedron;
        GameLevel::new(seed, shape, nodes_per_edge)
    }

    pub const fn cube(nodes_per_edge: u8, seed: u64) -> GameLevel {
        let shape = Shape::Cube;
        GameLevel::new(seed, shape, nodes_per_edge)
    }

    pub const fn octahedron(nodes_per_edge: u8, seed: u64) -> GameLevel {
        let shape = Shape::Octahedron;
        GameLevel::new(seed, shape, nodes_per_edge)
    }

    pub const fn dodecahedron(seed: u64) -> GameLevel {
        let shape = Shape::Dodecahedron;
        GameLevel::new(seed, shape, 1)
    }

    pub const fn icosahedron(nodes_per_edge: u8, seed: u64) -> GameLevel {
        let shape = Shape::Icosahedron;
        GameLevel::new(seed, shape, nodes_per_edge)
    }

    pub fn filename(&self) -> String {
        let shape = match &self.shape {
            Shape::Cube => "cube",
            Shape::Tetrahedron => "tetrahedron",
            Shape::Octahedron => "octahedron",
            Shape::Dodecahedron => "dodecahedron",
            Shape::Icosahedron => "icosahedron",
        };

        format!(
            "levels/{}_s{:?}_n{:?}.json",
            shape, self.seed, self.nodes_per_edge
        )
    }
}

#[derive(Component)]
pub struct MazeSaveDataHandle(Handle<MazeLevelData>);

pub fn despawn_level_data(mut commands: Commands, level_entities: Query<Entity, With<LevelData>>) {
    for entity in level_entities.iter() {
        commands.entity(entity).despawn();
    }
}

pub fn load_level_asset(
    mut commands: Commands,
    save_data_query: Query<&SaveData>,
    mut game_state: ResMut<NextState<PlayState>>,
    asset_server: Res<AssetServer>,
) {
    let save_data = save_data_query.single();

    let level = &LEVELS[save_data.current_index];

    let file_path = level.filename();

    let maze_save_data_handle = asset_server.load::<MazeLevelData>(file_path);

    let mesh_builder = level.get_maze_mesh_builder();

    commands.spawn((
        level.clone(),
        mesh_builder,
        MazeSaveDataHandle(maze_save_data_handle),
        LevelData,
    ));
}

pub fn spawn_level_data_components(
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
            (*node_id, audio_handle)
        })
        .collect::<HashMap<u64, Handle<MidiAudio>>>();

    // TODO: perhaps think about how not to duplicate the data here.
    commands.spawn((
        LevelData,
        GraphComponent(graph.clone()),
        SolutionComponent(solution.clone()),
        NoteMapping(note_midi_handle),
    ));
    game_state.set(PlayState::Playing);
}

pub fn spawn_level_meshes(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: Res<Assets<StandardMaterial>>,
    level_query: Query<(&MazeMeshBuilder, &GameLevel)>,
    maze_query: Query<(&GraphComponent, &SolutionComponent)>,
    settings: Res<GameSettings>,
    asset_handles: Res<GameMaterialHandles>,
) {
    let Ok((mesh_builder, level)) = level_query.get_single() else {
        return;
    };

    let Ok((GraphComponent(graph), SolutionComponent(solution))) = maze_query.get_single() else {
        return;
    };

    let palette = &settings.palette;

    let room_mesh_handle = meshes.add(mesh_builder.intersection_room_mesh());
    let goal_mesh_handle = meshes.add(mesh_builder.goal_mesh());

    let goal_node = solution.last().unwrap();
    for node in graph.nodes().filter(|room| is_junction(room, &graph)) {
        let material_handle = if node == *goal_node {
            asset_handles.player_material.clone()
        } else {
            asset_handles.line_material.clone()
        };

        let transform = Transform::IDENTITY
            .looking_at(
                -node.face().normal(),
                node.face().normal().any_orthogonal_vector(),
            )
            .with_translation(node.position() + node.face().normal() * 0.002);

        let mesh_handle = if node == *goal_node {
            goal_mesh_handle.clone()
        } else {
            room_mesh_handle.clone()
        };

        commands
            .spawn(PbrBundle {
                mesh: Mesh3d(mesh_handle),
                material: MeshMaterial3d(material_handle),
                transform,
                ..default()
            })
            .insert(LevelData);
    }

    let edge_mesh = meshes.add(mesh_builder.edge());
    let one_way_edge_mesh = meshes.add(mesh_builder.one_way_edge());
    let cross_face_edge_mesh = meshes.add(mesh_builder.cross_face_edge());
    let cross_face_one_way_edge_mesh = meshes.add(mesh_builder.cross_face_one_way_edge());

    for (source_node, target_node, _) in graph.all_edges() {
        let bidirectional = graph.contains_edge(target_node, source_node);

        if bidirectional && source_node.cmp(&target_node).is_lt() {
            continue;
        }

        let Some(border_type) = level.border_type(&source_node.face(), &target_node.face()) else {
            panic!["unknown edge type"];
        };

        let mesh_handle = match (&border_type, bidirectional) {
            (BorderType::SameFace, true) => edge_mesh.clone(),
            (BorderType::SameFace, false) => one_way_edge_mesh.clone(),
            (BorderType::Connected, true) => cross_face_edge_mesh.clone(),
            (BorderType::Connected, false) => cross_face_one_way_edge_mesh.clone(),
        };

        let transform = get_connection_transform(source_node, target_node, &border_type);

        let mut entity_commands =
            commands.spawn((Mesh3d(mesh_handle), transform.clone(), LevelData));

        let material_handle = if bidirectional {
            entity_commands.insert(MeshMaterial3d(asset_handles.line_material.clone()));
        } else {
            entity_commands.insert(MeshMaterial3d(asset_handles.dashed_arrow_material.clone()));
        };
    }

    let materials = level.get_face_materials(&asset_handles.face_materials);
    let face_meshes = level.get_face_meshes();

    for (face_mesh, face_material_handle) in face_meshes.into_iter().zip(materials.into_iter()) {
        let face_mesh_handle = meshes.add(face_mesh);

        commands
            .spawn(Mesh3d(face_mesh_handle))
            .insert(MeshMaterial3d(face_material_handle))
            .insert(LevelData);
    }
}

fn get_connection_transform(from: Room, to: Room, border_type: &BorderType) -> Transform {
    match border_type {
        BorderType::SameFace => {
            let forward = from.position() - to.position();
            Transform::IDENTITY
                .looking_to(forward, from.face().normal())
                .with_translation(from.position() + from.face().normal() * 0.001)
        }
        BorderType::Connected => get_cross_face_edge_transform(
            from.position(),
            from.face().normal(),
            to.position(),
            to.face().normal(),
        ),
    }
}

pub fn get_cross_face_edge_transform(
    from_position: Vec3,
    from_normal: Vec3,
    to_position: Vec3,
    to_normal: Vec3,
) -> Transform {
    let half_angle = from_normal.angle_between(to_normal) / 2.0;

    let average_normal = from_normal.lerp(to_normal, 0.5).normalize();

    let edge_vec = to_position - from_position;

    let intersection_point =
        from_position + (edge_vec + edge_vec.norm() * half_angle.tan() * average_normal) / 2.0;

    Transform::IDENTITY
        .looking_to(intersection_point - to_position, to_normal)
        .with_translation(intersection_point + average_normal * 0.0001)
}
