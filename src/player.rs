use std::fmt::Debug;

use crate::{
    load_maze,
    shape::cube::maze::{CubeMaze, CubeNode},
};
use bevy::{math::VectorSpace, prelude::*};
use bevy_rapier3d::geometry::Collider;

#[derive(Component)]
pub struct Player;

#[derive(Component, Debug)]
pub enum PlayerMazeState {
    Node(CubeNode),
    Edge(CubeNode, CubeNode, Vec3),
}

#[derive(Default)]
pub struct PlayerPlugin;

impl Plugin for PlayerPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Update, move_player)
            .add_systems(Startup, setup_player.after(load_maze));
    }
}

pub fn setup_player(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    cube_maze: Res<CubeMaze>,
) {
    let white = Color::srgb_u8(247, 247, 0);

    let white_material = materials.add(StandardMaterial::from_color(white));

    let initial_node = cube_maze.maze.solution.first().unwrap().clone();
    let player_transform = compute_initial_player_transform(initial_node);
    let player_shape = Sphere::new(0.1);
    let player_mesh = meshes.add(player_shape);

    commands
        .spawn(PbrBundle {
            mesh: player_mesh,
            material: white_material.clone(),
            transform: player_transform,
            ..default()
        })
        .insert(Player)
        .insert(PlayerMazeState::Node(initial_node))
        .insert(Collider::ball(player_shape.radius));
}

fn compute_initial_player_transform(start_node: CubeNode) -> Transform {
    let face_normal = start_node.face.normal();

    Transform::IDENTITY
        .looking_at(face_normal.any_orthogonal_vector(), face_normal)
        .with_translation(start_node.position + 0.2 * face_normal)
}

fn move_player(mut player_query: Query<(&mut Transform, &PlayerMazeState)>, maze: Res<CubeMaze>) {
    let (mut player_transform, player_maze_state) = player_query.single_mut();

    let target_position = match player_maze_state {
        PlayerMazeState::Node(node) => node.position + maze.player_elevation * node.face.normal(),
        PlayerMazeState::Edge(_, _, edge_position) => edge_position.clone(),
    };

    player_transform.translation = player_transform.translation.lerp(target_position, 0.1)
}
