use bevy::{
    asset::Assets,
    color::Color,
    ecs::system::{Commands, ResMut},
    math::NormedVectorSpace,
    pbr::{ExtendedMaterial, PbrBundle, StandardMaterial},
    prelude::*,
    render::mesh::Mesh,
    transform::components::Transform,
};

use std::{
    f32::consts::FRAC_PI_2,
    fs::{self, File},
    hash::{DefaultHasher, Hash, Hasher},
    usize,
};

use bevy_rapier3d::geometry::Collider;
use petgraph::{graphmap::GraphMap, Directed};

use crate::{
    assets::{FaceMaterialHandles, ShapeFaceMaterial},
    game_settings::{FaceColorPalette, GameSettings},
    game_state::GameState,
    is_room_junction::is_junction,
    player::{Player, PlayerMazeState},
    room::{SolidFace, SolidRoom},
};

use super::{
    cube::Cube,
    dodecahedron::Dodecahedron,
    octahedron::Octahedron,
    platonic_mesh_builder::MazeMeshBuilder,
    shape_loader::{BorderType, Edge, ShapeLoader},
};
use super::{icosahedron::Icosahedron, tetrahedron::Tetrahedron};
use crate::assets::GameAssetHandles;
use serde::{Deserialize, Serialize};

#[derive(Component)]
pub struct GraphComponent(pub GraphMap<SolidRoom, Edge, Directed>);

#[derive(Component)]
pub struct SolutionComponent(pub Vec<SolidRoom>);

#[derive(Serialize, Deserialize, Asset, TypePath)]
pub struct MazeSaveData(pub GraphMap<SolidRoom, Edge, Directed>, pub Vec<SolidRoom>);

#[derive(Component)]
pub struct LevelData;

#[derive(Clone, Debug)]
pub enum Shape {
    Cube(Cube),
    Tetrahedron(Tetrahedron),
    Icosahedron(Icosahedron),
    Octahedron(Octahedron),
    Dodecahedron(Dodecahedron),
}

#[derive(Component, Clone, Debug)]
pub struct GameLevel {
    pub seed: u64,
    pub face_size: f32,
    pub shape: Shape,
    pub nodes_per_edge: u8,
}

impl GameLevel {
    fn new(seed: u64, face_size: f32, shape: Shape, nodes_per_edge: u8) -> Self {
        GameLevel {
            seed,
            face_size,
            shape,
            nodes_per_edge,
        }
    }

    pub fn border_type(&self, from: &SolidFace, to: &SolidFace) -> Option<BorderType> {
        match &self.shape {
            Shape::Tetrahedron(_) => Tetrahedron::border_type(&from, &to),
            Shape::Cube(_) => Cube::border_type(&from, &to),
            Shape::Octahedron(_) => Octahedron::border_type(&from, &to),
            Shape::Dodecahedron(_) => Dodecahedron::border_type(&from, &to),
            Shape::Icosahedron(_) => Icosahedron::border_type(&from, &to),
        }
    }

    pub fn node_distance(&self) -> f32 {
        match &self.shape {
            Shape::Cube(cube) => cube.distance_between_nodes,
            Shape::Tetrahedron(tetrahedron) => tetrahedron.distance_between_nodes,
            Shape::Octahedron(octahedron) => octahedron.distance_between_nodes,
            Shape::Dodecahedron(dodecahedron) => dodecahedron.distance_between_nodes,
            Shape::Icosahedron(icosahedron) => icosahedron.distance_between_nodes,
        }
    }

    pub fn get_maze_mesh_builder(&self) -> MazeMeshBuilder {
        let node_distance = self.node_distance();

        let face_angle = match self.shape {
            Shape::Cube(_) => FRAC_PI_2,
            Shape::Tetrahedron(_) => (1.0_f32 / 3.0).acos(),
            Shape::Octahedron(_) => (-1.0_f32 / 3.0).acos(),
            Shape::Dodecahedron(_) => (-5.0_f32.sqrt() / 5.0).acos(),
            Shape::Icosahedron(_) => (-5.0_f32.sqrt() / 3.0).acos(),
        };

        MazeMeshBuilder::new(node_distance, face_angle)
    }

    pub fn get_face_materials(
        &self,
        face_materials_handles: &FaceMaterialHandles,
    ) -> Vec<Handle<ExtendedMaterial<StandardMaterial, ShapeFaceMaterial>>> {
        match &self.shape {
            Shape::Cube(_) => face_materials_handles.cube().into_iter().collect(),
            Shape::Tetrahedron(_) => face_materials_handles.tetrahedron().into_iter().collect(),
            Shape::Octahedron(_) => face_materials_handles.octahedron().into_iter().collect(),
            Shape::Dodecahedron(_) => face_materials_handles.dodecahedron().into_iter().collect(),
            Shape::Icosahedron(_) => face_materials_handles.icosahedron().into_iter().collect(),
        }
    }

    pub fn get_face_meshes(&self) -> Vec<Mesh> {
        match &self.shape {
            Shape::Cube(cube) => cube.get_face_meshes(),
            Shape::Tetrahedron(tetrahedron) => tetrahedron.get_face_meshes(),
            Shape::Octahedron(octahedron) => octahedron.get_face_meshes(),
            Shape::Dodecahedron(dodecahedron) => dodecahedron.get_face_meshes(),
            Shape::Icosahedron(icosahedron) => icosahedron.get_face_meshes(),
        }
    }

    pub fn tetrahedron(nodes_per_edge: u8, face_size: f32, seed: u64) -> GameLevel {
        let shape = Shape::Tetrahedron(Tetrahedron::new(nodes_per_edge, face_size));
        GameLevel::new(seed, face_size, shape, nodes_per_edge)
    }

    pub fn cube(nodes_per_edge: u8, face_size: f32, seed: u64) -> GameLevel {
        let shape = Shape::Cube(Cube::new(nodes_per_edge, face_size));
        GameLevel::new(seed, face_size, shape, nodes_per_edge)
    }

    pub fn octahedron(nodes_per_edge: u8, face_size: f32, seed: u64) -> GameLevel {
        let shape = Shape::Octahedron(Octahedron::new(nodes_per_edge, face_size));
        GameLevel::new(seed, face_size, shape, nodes_per_edge)
    }

    pub fn dodecahedron(face_size: f32, seed: u64) -> GameLevel {
        let shape = Shape::Dodecahedron(Dodecahedron::new(face_size));
        GameLevel::new(seed, face_size, shape, 1)
    }

    pub fn icosahedron(nodes_per_edge: u8, face_size: f32, seed: u64) -> GameLevel {
        let shape = Shape::Icosahedron(Icosahedron::new(nodes_per_edge, face_size));
        GameLevel::new(seed, face_size, shape, nodes_per_edge)
    }

    pub fn filename(&self) -> String {
        let shape = match &self.shape {
            Shape::Cube(_) => "cube",
            Shape::Tetrahedron(_) => "tetrahedron",
            Shape::Octahedron(_) => "octahedron",
            Shape::Dodecahedron(_) => "dodecahedron",
            Shape::Icosahedron(_) => "icosahedron",
        };

        format!(
            "levels/{}_s{:?}_n{:?}.json",
            shape, self.seed, self.nodes_per_edge
        )
    }
}

#[derive(Resource, Clone)]
pub struct LevelIndex(pub usize);

#[derive(Resource)]
pub struct Levels(pub Vec<GameLevel>);

#[derive(Default)]
pub struct LoaderPlugin;

impl Plugin for LoaderPlugin {
    fn build(&self, app: &mut App) {
        let levels = get_levels();
        app.insert_resource(LevelIndex(0));
        app.insert_resource(Levels(levels));
    }
}

pub fn get_levels() -> Vec<GameLevel> {
    vec![
        GameLevel::tetrahedron(1, 3.0, 1), // 4
        GameLevel::cube(2, 2.0, 2),        // 24
        GameLevel::octahedron(3, 2.4, 3),  // 48
        GameLevel::dodecahedron(1.0, 1),   // 60
        GameLevel::icosahedron(2, 2.0, 2), // 60
        GameLevel::tetrahedron(5, 3.0, 2), // 60
        GameLevel::octahedron(4, 2.0, 4),  // 80
        GameLevel::cube(4, 2.0, 3),        // 96
        GameLevel::icosahedron(3, 2.0, 2), // 120
        GameLevel::cube(6, 2.0, 1),        // 216
    ]
}

#[derive(Component)]
pub struct MazeSaveDataHandle(Handle<MazeSaveData>);

pub fn load_level_asset(
    mut commands: Commands,
    level_resource: Res<LevelIndex>,
    levels: Res<Levels>,
    mut game_state: ResMut<NextState<GameState>>,
    current_level_entities: Query<Entity, With<LevelData>>,
    asset_server: Res<AssetServer>,
) {
    let LevelIndex(index) = level_resource.into_inner();
    let Levels(levels) = levels.into_inner();

    for entity in current_level_entities.iter() {
        commands.entity(entity).despawn();
    }

    let level = levels.get(*index).unwrap();

    let file_path = level.filename();

    let maze_save_data_handle = asset_server.load::<MazeSaveData>(file_path);

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
    mut game_state: ResMut<NextState<GameState>>,
    current_level_entities: Query<Entity, With<LevelData>>,
    mut maze_save_data_event: EventReader<AssetEvent<MazeSaveData>>,
    maze_save_data_assets: Res<Assets<MazeSaveData>>,
) {
    for ev in maze_save_data_event.read() {
        let AssetEvent::Added { id } = ev else {
            continue;
        };

        let Some(MazeSaveData(graph, solution)) = maze_save_data_assets.get(*id) else {
            continue;
        };

        // TODO: perhaps think about how not to duplicate the data here.
        commands.spawn((
            LevelData,
            GraphComponent(graph.clone()),
            SolutionComponent(solution.clone()),
        ));
        game_state.set(GameState::Playing);
    }
}

pub fn spawn_level_meshes(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: Res<Assets<StandardMaterial>>,
    level_query: Query<(&MazeMeshBuilder, &GameLevel)>,
    maze_query: Query<(&GraphComponent, &SolutionComponent)>,
    settings: Res<GameSettings>,
    asset_handles: Res<GameAssetHandles>,
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

fn get_connection_transform(from: SolidRoom, to: SolidRoom, border_type: &BorderType) -> Transform {
    match border_type {
        BorderType::SameFace => {
            let forward = from.position() - to.position();
            Transform::IDENTITY
                .looking_to(forward, from.face().normal())
                .with_translation(from.position() + from.face().normal() * 0.001)
        }
        BorderType::Connected => {
            let from_normal = from.face().normal();
            let to_normal = to.face().normal();

            let half_angle = from_normal.angle_between(to_normal) / 2.0;

            let average_normal = from_normal.lerp(to_normal, 0.5).normalize();

            let edge_vec = to.position() - from.position();

            let intersection_point = from.position()
                + (edge_vec + edge_vec.norm() * half_angle.tan() * average_normal) / 2.0;

            Transform::IDENTITY
                .looking_to(intersection_point - to.position(), to.face().normal())
                .with_translation(intersection_point + average_normal * 0.001)
        }
    }
}

pub fn spawn_player(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mesh_builder_query: Query<&MazeMeshBuilder>,
    solution_query: Query<&SolutionComponent>,
    settings: Res<GameSettings>,
    asset_handles: Res<GameAssetHandles>,
) {
    let Ok(mesh_builder) = mesh_builder_query.get_single() else {
        return;
    };
    let Ok(SolutionComponent(solution)) = solution_query.get_single() else {
        return;
    };

    let initial_node = solution.first().unwrap().clone();
    let player_mesh = mesh_builder.player_mesh();
    let player_mesh_handle = meshes.add(player_mesh);

    let height_above_node = settings.player_elevation + player_mesh.radius;
    let player_transform = compute_initial_player_transform(initial_node, height_above_node);

    commands
        .spawn(PbrBundle {
            mesh: Mesh3d(player_mesh_handle),
            material: MeshMaterial3d(asset_handles.player_material.clone()),
            transform: player_transform,
            ..default()
        })
        .insert(Player {
            size: player_mesh.radius,
        })
        .insert(PlayerMazeState::Node(initial_node))
        .insert(Collider::ball(player_mesh.radius))
        .insert(LevelData);
}

fn compute_initial_player_transform(start_node: SolidRoom, player_elevation: f32) -> Transform {
    let face_normal = start_node.face().normal();

    Transform::IDENTITY
        .looking_at(face_normal.any_orthogonal_vector(), face_normal)
        .with_translation(start_node.position() + player_elevation * face_normal)
}
