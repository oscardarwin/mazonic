use std::fmt::Debug;

use crate::{
    assets::GameAssetHandles,
    game_settings::GameSettings,
    room::SolidRoom,
    shape::{loader::LevelData, platonic_mesh_builder::MazeMeshBuilder},
};
use bevy::prelude::*;

#[derive(Component)]
pub struct Player {
    pub size: f32,
}

#[derive(Component, Debug)]
pub enum PlayerMazeState {
    Node(SolidRoom),
    Edge(SolidRoom, SolidRoom, Vec3),
}

pub fn move_player(
    mut player_query: Query<(&mut Transform, &PlayerMazeState, &Player)>,
    settings: Res<GameSettings>,
) {
    let Ok((mut player_transform, player_maze_state, Player { size })) =
        player_query.get_single_mut()
    else {
        return;
    };

    let target_position = match player_maze_state {
        PlayerMazeState::Node(node) => {
            let height_above_node = settings.player_elevation + size;
            node.position() + height_above_node * node.face().normal()
        }
        PlayerMazeState::Edge(_, _, edge_position) => edge_position.clone(),
    };

    player_transform.translation = player_transform.translation.lerp(target_position, 0.1)
}

#[derive(Component)]
pub struct PlayerHalo;

pub fn spawn_player_halo(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    settings: Res<GameSettings>,
    player_query: Query<&Transform, With<Player>>,
    mesh_builder_query: Query<&MazeMeshBuilder>,
    asset_handles: Res<GameAssetHandles>,
) {
    let Ok(mesh_builder) = mesh_builder_query.get_single() else {
        return;
    };

    let Ok(player_transform) = player_query.get_single() else {
        return;
    };

    let player_halo_mesh = mesh_builder.player_halo_mesh();
    let player_mesh_handle = meshes.add(player_halo_mesh);

    commands
        .spawn(PbrBundle {
            mesh: Mesh3d(player_mesh_handle),
            material: MeshMaterial3d(asset_handles.player_halo_material.clone()),
            transform: *player_transform,
            ..default()
        })
        .insert(LevelData)
        .insert(PlayerHalo);
}

pub fn player_halo_follow_player(
    mut player_halo_query: Query<&mut Transform, With<PlayerHalo>>,
    player_query: Query<&Transform, (With<Player>, Without<PlayerHalo>)>,
) {
    let Ok(mut player_halo_transform) = player_halo_query.get_single_mut() else {
        return;
    };

    let Ok(player_transform) = player_query.get_single() else {
        return;
    };

    player_halo_transform.translation = player_transform.translation.clone();
}

pub fn despawn_player_halo(
    mut commands: Commands,
    player_halo_query: Query<Entity, With<PlayerHalo>>,
) {
    let Ok(player_halo_entity) = player_halo_query.get_single() else {
        return;
    };

    commands.entity(player_halo_entity).despawn();
}
