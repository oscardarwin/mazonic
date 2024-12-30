use std::usize;

use bevy::{
    asset::Assets,
    color::Color,
    ecs::{
        schedule::SystemConfigs,
        system::{Commands, ResMut},
    },
    log::Level,
    math::NormedVectorSpace,
    pbr::{PbrBundle, StandardMaterial},
    prelude::*,
    render::mesh::Mesh,
    transform::components::Transform,
};

use bevy_rapier3d::{geometry::Collider, parry::shape::Shape};
use maze_generator::config::Maze;
use petgraph::Direction;

use itertools::Itertools;
use strum::IntoEnumIterator;
use strum_macros::{EnumDiscriminants, EnumIter};

use crate::{
    controller::{solve, ControllerState},
    game_settings::GameSettings,
    game_state::GameState,
    player::{move_player, Player, PlayerMazeState},
};

use super::{
    cube::Cube,
    dodecahedron::Dodecahedron,
    octahedron::Octahedron,
    platonic_solid::{BorderType, Edge, HasFace, IsRoom, PlatonicSolid},
};
use super::{icosahedron::Icosahedron, tetrahedron::Tetrahedron};

#[derive(Resource)]
pub struct PlatonicLevelData<P: PlatonicSolid> {
    pub platonic_solid: P,
    pub maze: Maze<P::Room, Edge>,
}

#[derive(Resource, EnumDiscriminants, Clone)]
#[strum_discriminants(vis(pub))]
#[strum_discriminants(derive(EnumIter, Hash, States))]
#[strum_discriminants(name(LevelType))]
pub enum LevelLoadData {
    Cube(Cube),
    Tetrahedron(Tetrahedron),
    Icosahedron(Icosahedron),
    Octahedron(Octahedron),
    Dodecahedron(Dodecahedron),
}

#[derive(Resource, Clone)]
pub struct LevelIndex(pub usize);

#[derive(Resource)]
pub struct Levels(pub Vec<LevelLoadData>);

#[derive(Default)]
pub struct LoaderPlugin;

impl Plugin for LoaderPlugin {
    fn build(&self, app: &mut App) {
        let levels = vec![
            LevelLoadData::Cube(Cube::new(1, 2.0)),
            LevelLoadData::Tetrahedron(Tetrahedron::new(4, 3.0)),
            LevelLoadData::Cube(Cube::new(2, 2.0)),
            LevelLoadData::Cube(Cube::new(6, 2.0)),
            LevelLoadData::Dodecahedron(Dodecahedron::new(1.0)),
            LevelLoadData::Octahedron(Octahedron::new(3, 2.0)),
            LevelLoadData::Icosahedron(Icosahedron::new(3, 2.0)),
            LevelLoadData::Cube(Cube::new(3, 2.0)),
            LevelLoadData::Tetrahedron(Tetrahedron::new(6, 2.0)),
        ];

        let first_level = levels[0].clone();

        let first_level_type = LevelType::from(&first_level);
        app.insert_state(first_level_type);

        app.insert_resource(LevelIndex(0));
        app.insert_resource(Levels(levels));
    }
}

pub fn load_level(
    mut commands: Commands,
    level_resource: Res<LevelIndex>,
    levels: Res<Levels>,
    mut level_type_state: ResMut<NextState<LevelType>>,
    mut game_state: ResMut<NextState<GameState>>,
    current_level_entities: Query<Entity, With<LevelMesh>>,
) {
    for entity in current_level_entities.iter() {
        commands.entity(entity).despawn();
    }

    let LevelIndex(index) = level_resource.into_inner();
    let Levels(levels) = levels.into_inner();

    let level = levels.get(*index).unwrap();

    match level {
        LevelLoadData::Cube(cube) => load_platonic_maze::<Cube>(commands, cube),
        LevelLoadData::Tetrahedron(tetrahedron) => {
            load_platonic_maze::<Tetrahedron>(commands, tetrahedron)
        }
        LevelLoadData::Icosahedron(icosahedron) => {
            load_platonic_maze::<Icosahedron>(commands, icosahedron)
        }
        LevelLoadData::Octahedron(octahedron) => {
            load_platonic_maze::<Octahedron>(commands, octahedron)
        }
        LevelLoadData::Dodecahedron(dodecahedron) => {
            load_platonic_maze::<Dodecahedron>(commands, dodecahedron)
        }
    }

    println!("Loaded level {index}");
    let level_type = LevelType::from(level);

    level_type_state.set(level_type);
    game_state.set(GameState::Playing);
}

fn load_platonic_maze<P: PlatonicSolid>(mut commands: Commands, platonic_solid: &P) {
    let maze = platonic_solid.build_maze();

    let level_data = PlatonicLevelData::<P> {
        maze,
        platonic_solid: platonic_solid.clone(),
    };

    commands.insert_resource(level_data);
}

#[derive(Component)]
pub struct PlatonicSolidComponent(pub Vec<Vec3>);

#[derive(Component)]
pub struct LevelMesh;

pub fn spawn_level_meshes<P: PlatonicSolid>(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    level: Res<PlatonicLevelData<P>>,
    settings: Res<GameSettings>,
) {
    let cyan = Color::srgb_u8(247, 247, 0);
    let beige = Color::srgb_u8(242, 231, 213);
    let green = Color::srgb_u8(109, 152, 134);

    let cyan_material = materials.add(StandardMaterial::from_color(cyan));
    let beige_material = materials.add(StandardMaterial::from_color(beige));
    let green_material = materials.add(StandardMaterial::from_color(green));

    let edge_mesh_builder = level.platonic_solid.get_mesh_builder();
    let room_mesh_handle = meshes.add(edge_mesh_builder.room_mesh());
    let goal_mesh_handle = meshes.add(edge_mesh_builder.goal_mesh());

    let goal_node = level.maze.solution.last().unwrap();
    for node in level.maze.graph.nodes().filter(|node| {
        let incoming_neighbors = level
            .maze
            .graph
            .neighbors_directed(*node, Direction::Incoming);
        let outgoing_neighbors = level
            .maze
            .graph
            .neighbors_directed(*node, Direction::Outgoing);

        let neighbors = incoming_neighbors
            .chain(outgoing_neighbors)
            .unique()
            .collect::<Vec<P::Room>>();

        neighbors.len() != 2 || {
            let first_neighbor = neighbors[0];
            let second_neighbor = neighbors[1];

            let node_to_first_vec = node.position() - first_neighbor.position();
            let node_to_second_vec = node.position() - second_neighbor.position();

            node_to_first_vec.dot(node_to_second_vec).abs() < 0.1
        }
    }) {
        let material_handle = if node == *goal_node {
            cyan_material.clone()
        } else {
            beige_material.clone()
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
            .insert(LevelMesh);
    }

    let face_connection_mesh = meshes.add(edge_mesh_builder.line());
    let face_arrow_mesh = meshes.add(edge_mesh_builder.dashed_arrow());

    let edge_mesh = meshes.add(edge_mesh_builder.edge_line());
    let edge_arrow_mesh = meshes.add(edge_mesh_builder.dashed_arrow_edge());

    for (source_node, target_node, _) in level.maze.graph.all_edges() {
        let bidirectional = level.maze.graph.contains_edge(target_node, source_node);

        if bidirectional && source_node.cmp(&target_node).is_lt() {
            continue;
        }

        let Some(border_type) = source_node.face().border_type(&target_node.face()) else {
            panic!["unknown edge type"];
        };

        let mesh_handle = match (&border_type, bidirectional) {
            (BorderType::SameFace, true) => face_connection_mesh.clone(),
            (BorderType::SameFace, false) => face_arrow_mesh.clone(),
            (BorderType::Connected, true) => edge_mesh.clone(),
            (BorderType::Connected, false) => edge_arrow_mesh.clone(),
        };

        let transform = get_connection_transform::<P>(source_node, target_node, &border_type);

        commands
            .spawn(PbrBundle {
                mesh: Mesh3d(mesh_handle),
                material: MeshMaterial3d(beige_material.clone()),
                transform,
                ..default()
            })
            .insert(LevelMesh);
    }
    let platonic_mesh = edge_mesh_builder.platonic_solid_mesh;
    let vertices = platonic_mesh
        .attribute(Mesh::ATTRIBUTE_POSITION)
        .unwrap()
        .as_float3()
        .unwrap()
        .iter()
        .map(|arr| Vec3::from_array(*arr))
        .collect::<Vec<Vec3>>();

    let platonic_solid_mesh_handle = meshes.add(platonic_mesh);

    commands
        .spawn(PbrBundle {
            mesh: Mesh3d(platonic_solid_mesh_handle),
            material: MeshMaterial3d(green_material.clone()),
            transform: Transform::IDENTITY,
            ..default()
        })
        .insert(PlatonicSolidComponent(vertices))
        .insert(LevelMesh);

    let initial_node = level.maze.solution.first().unwrap().clone();
    let player_transform =
        compute_initial_player_transform::<P>(initial_node, settings.player_elevation);
    let player_shape = Sphere::new(0.1);
    let player_mesh = meshes.add(player_shape);

    commands
        .spawn(PbrBundle {
            mesh: Mesh3d(player_mesh),
            material: MeshMaterial3d(cyan_material.clone()),
            transform: player_transform,
            ..default()
        })
        .insert(Player)
        .insert(PlayerMazeState::<P>::Node(initial_node))
        .insert(Collider::ball(player_shape.radius))
        .insert(LevelMesh);
}

fn get_connection_transform<P: PlatonicSolid>(
    from: P::Room,
    to: P::Room,
    border_type: &BorderType,
) -> Transform {
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

fn compute_initial_player_transform<P: PlatonicSolid>(
    start_node: P::Room,
    player_elevation: f32,
) -> Transform {
    let face_normal = start_node.face().normal();

    Transform::IDENTITY
        .looking_at(face_normal.any_orthogonal_vector(), face_normal)
        .with_translation(start_node.position() + player_elevation * face_normal)
}
