use std::f32::consts::PI;

use crate::shape::cube::{
    self,
    maze::{CubeMaze, CubeNode, Edge, Face},
};
#[cfg(not(target_arch = "wasm32"))]
use bevy::pbr::wireframe::{WireframeConfig, WireframePlugin};
use bevy::{
    color::palettes::basic::SILVER,
    math::{vec2, NormedVectorSpace},
    prelude::*,
    render::{
        mesh::{Indices, PrimitiveTopology},
        render_asset::RenderAssetUsages,
        render_resource::{Extent3d, TextureDimension, TextureFormat},
    },
    window::PrimaryWindow,
};
use bevy_rapier3d::{
    geometry::Collider,
    pipeline::QueryFilter,
    plugin::{NoUserData, RapierContext, RapierPhysicsPlugin},
};

/// A marker component for our shapes so we can query them separately from the ground plane
#[derive(Component)]
pub struct Player;

#[derive(Component, Debug)]
pub enum PlayerMazePosition {
    Node(CubeNode),
    Edge(CubeNode, CubeNode),
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
        .insert(PlayerMazePosition::Node(initial_node))
        .insert(Collider::ball(player_shape.radius));
}

fn compute_initial_player_transform(start_node: CubeNode) -> Transform {
    let face_normal = start_node.face.normal();

    Transform::IDENTITY
        .looking_at(face_normal.any_orthogonal_vector(), face_normal)
        .with_translation(start_node.position + 0.2 * face_normal)
}
