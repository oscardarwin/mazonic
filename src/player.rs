use std::fmt::Debug;

use crate::{
    game_settings::GameSettings,
    load_maze,
    shape::cube::{Cube, CubeRoom},
    shape::platonic_solid::{HasFace, IsRoom, PlatonicSolid},
    Level,
};
use bevy::prelude::*;
use bevy_rapier3d::geometry::Collider;

#[derive(Component)]
pub struct Player;

#[derive(Component, Debug)]
pub enum PlayerMazeState<P: PlatonicSolid> {
    Node(P::Room),
    Edge(P::Room, P::Room, Vec3),
}

#[derive(Default)]
pub struct PlayerPlugin;

impl Plugin for PlayerPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Update, move_player::<Cube>)
            .add_systems(Startup, setup_player::<Cube>.after(load_maze));
    }
}

pub fn setup_player<P: PlatonicSolid>(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    settings: Res<GameSettings>,
    level: Res<Level<P>>,
) {
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

fn move_player<P: PlatonicSolid>(
    mut player_query: Query<(&mut Transform, &PlayerMazeState<P>)>,
    settings: Res<GameSettings>,
) {
    let (mut player_transform, player_maze_state) = player_query.single_mut();

    let target_position = match player_maze_state {
        PlayerMazeState::<P>::Node(node) => {
            node.position() + settings.player_elevation * node.face().normal()
        }
        PlayerMazeState::<P>::Edge(_, _, edge_position) => edge_position.clone(),
    };

    player_transform.translation = player_transform.translation.lerp(target_position, 0.1)
}
