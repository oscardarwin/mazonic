use std::usize;

use bevy::{
    asset::Assets,
    color::Color,
    ecs::{
        schedule::SystemConfigs,
        system::{Commands, ResMut},
    },
    math::NormedVectorSpace,
    pbr::{PbrBundle, StandardMaterial},
    prelude::*,
    render::mesh::Mesh,
    transform::components::Transform,
};

use bevy_rapier3d::geometry::Collider;
use maze_generator::config::Maze;
use petgraph::Direction;

use itertools::Itertools;
use strum::IntoEnumIterator;
use strum_macros::{EnumDiscriminants, EnumIter};

use crate::{
    game_settings::GameSettings,
    player::{Player, PlayerMazeState},
};

use super::tetrahedron::Tetrahedron;
use super::{
    cube::Cube,
    platonic_solid::{BorderType, Edge, HasFace, IsRoom, PlatonicSolid},
};

#[derive(Resource)]
pub struct PlatonicLevelData<P: PlatonicSolid> {
    pub platonic_solid: P,
    pub maze: Maze<P::Room, Edge>,
}

#[derive(Resource, EnumDiscriminants, Clone)]
#[strum_discriminants(derive(EnumIter, Hash, States))]
#[strum_discriminants(name(LevelType))]
enum LevelLoadData {
    Cube(Cube),
    Tetrahedron(Tetrahedron),
}

#[derive(Resource)]
struct Levels(Vec<LevelLoadData>);

#[derive(Default)]
pub struct LoaderPlugin;

impl LoaderPlugin {
    fn get_systems_for_level_type(&self, level_type: LevelType) -> SystemConfigs {
        match level_type {
            LevelType::Cube => self.add_systems_for_solid_type::<Cube>(),
            LevelType::Tetrahedron => self.add_systems_for_solid_type::<Tetrahedron>(),
        }
    }

    fn add_systems_for_solid_type<P: PlatonicSolid>(&self) -> SystemConfigs {
        println!("First level: {:?}", std::any::type_name::<P>());
        (spawn_shape_meshes::<P>, setup_player::<P>).into_configs()
    }

    fn setup_first_level(&self, app: &mut App, first_level: LevelLoadData) {
        let first_level_type = LevelType::from(&first_level);

        let first_level_setup_systems = self.get_systems_for_level_type(first_level_type);
        app.add_systems(Startup, first_level_setup_systems.after(load_maze));
        app.insert_state(first_level_type);
        app.insert_resource(first_level);
    }
}

impl Plugin for LoaderPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Startup, load_maze);

        for level_type in LevelType::iter() {
            let systems = self.get_systems_for_level_type(level_type);
            app.add_systems(OnEnter(level_type), systems);
        }
        let levels = vec![
            LevelLoadData::Cube(Cube::new(2, 1.5)),
            LevelLoadData::Tetrahedron(Tetrahedron::new(4, 4.0)),
            LevelLoadData::Cube(Cube::new(3, 4.0)),
            LevelLoadData::Tetrahedron(Tetrahedron::new(6, 4.0)),
            LevelLoadData::Cube(Cube::new(6, 4.0)),
        ];

        let first_level = levels[0].clone();
        self.setup_first_level(app, first_level.clone());
        app.insert_resource(Levels(levels));
    }
}

pub fn load_maze(
    mut commands: Commands,
    mut next_level_type: ResMut<NextState<LevelType>>,
    level_resource: Res<LevelLoadData>,
) {
    let level: &LevelLoadData = level_resource.into_inner();

    match level {
        LevelLoadData::Cube(cube) => load_platonic_maze::<Cube>(commands, cube),
        LevelLoadData::Tetrahedron(tetrahedron) => {
            load_platonic_maze::<Tetrahedron>(commands, tetrahedron)
        }
    }
    next_level_type.set(LevelType::from(level));
}

fn load_platonic_maze<P: PlatonicSolid>(mut commands: Commands, platonic_solid: &P) {
    let maze = platonic_solid.build_maze();

    commands.insert_resource(PlatonicLevelData::<P> {
        maze,
        platonic_solid: platonic_solid.clone(),
    });
}

pub fn spawn_shape_meshes<P: PlatonicSolid>(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    level: Res<PlatonicLevelData<P>>,
) {
    println!("spawn shape mesh");

    let cyan = Color::srgb_u8(247, 247, 0);
    let beige = Color::srgb_u8(242, 231, 213);
    let green = Color::srgb_u8(109, 152, 134);

    let cyan_material = materials.add(StandardMaterial::from_color(cyan));
    let beige_material = materials.add(StandardMaterial::from_color(beige));
    let green_material = materials.add(StandardMaterial::from_color(green));

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

        let radius = if node == *goal_node { 0.1 } else { 0.06 };

        commands.spawn(PbrBundle {
            mesh: Mesh3d(meshes.add(Circle::new(radius))),
            material: MeshMaterial3d(material_handle),
            transform,
            ..default()
        });
    }

    let edge_mesh_builder = level.platonic_solid.get_mesh_builder();

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

        commands.spawn(PbrBundle {
            mesh: Mesh3d(mesh_handle),
            material: MeshMaterial3d(beige_material.clone()),
            transform,
            ..default()
        });
    }

    let cuboid = meshes.add(edge_mesh_builder.platonic_solid_mesh);
    commands.spawn(PbrBundle {
        mesh: Mesh3d(cuboid),
        material: MeshMaterial3d(green_material.clone()),
        transform: Transform::IDENTITY,
        ..default()
    });
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

pub fn setup_player<P: PlatonicSolid>(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    settings: Res<GameSettings>,
    level: Res<PlatonicLevelData<P>>,
) {
    println!["setup player"];

    let white = Color::srgb_u8(247, 247, 0);

    let white_material = materials.add(StandardMaterial::from_color(white));

    let initial_node = level.maze.solution.first().unwrap().clone();
    let player_transform =
        compute_initial_player_transform::<P>(initial_node, settings.player_elevation);
    let player_shape = Sphere::new(0.1);
    let player_mesh = meshes.add(player_shape);

    commands
        .spawn(PbrBundle {
            mesh: Mesh3d(player_mesh),
            material: MeshMaterial3d(white_material.clone()),
            transform: player_transform,
            ..default()
        })
        .insert(Player)
        .insert(PlayerMazeState::<P>::Node(initial_node))
        .insert(Collider::ball(player_shape.radius));
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
