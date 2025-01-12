use std::fmt::Debug;

use crate::{
    assets::{GameAssetHandles, PlayerHaloMaterial},
    game_settings::GameSettings,
    room::SolidRoom,
    shape::{loader::LevelData, platonic_mesh_builder::MazeMeshBuilder},
};
use bevy::{math::NormedVectorSpace, pbr::ExtendedMaterial, prelude::*};

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
pub struct PlayerHalo {
    visible: bool,
}

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
        .spawn(Mesh3d(player_mesh_handle))
        .insert(MeshMaterial3d(asset_handles.player_halo_material.clone()))
        .insert(*player_transform)
        .insert(LevelData)
        .insert(PlayerHalo { visible: true });
}

pub fn turn_on_player_halo(mut player_halo_query: Query<&mut PlayerHalo>) {
    if let Ok(mut player_halo) = player_halo_query.get_single_mut() {
        player_halo.visible = true;
    }
}

pub fn turn_off_player_halo(mut player_halo_query: Query<&mut PlayerHalo>) {
    if let Ok(mut player_halo) = player_halo_query.get_single_mut() {
        player_halo.visible = false;
    }
}

pub fn update_halo_follow_player(
    mut player_halo_query: Query<(&mut Transform, &PlayerHalo)>,
    player_query: Query<&Transform, (With<Player>, Without<PlayerHalo>)>,
    mut player_halo_materials: ResMut<
        Assets<ExtendedMaterial<StandardMaterial, PlayerHaloMaterial>>,
    >,
    mut materials: ResMut<Assets<StandardMaterial>>,
    asset_handles: Res<GameAssetHandles>,
) {
    let Ok((mut player_halo_transform, halo)) = player_halo_query.get_single_mut() else {
        return;
    };

    let Ok(player_transform) = player_query.get_single() else {
        return;
    };

    player_halo_transform.translation = player_transform.translation.clone();

    let mut player_material = materials.get_mut(&asset_handles.player_material).unwrap();
    let target_luminance_factor = if halo.visible { 2.0 } else { 0.003 };
    let luminance_rate = if halo.visible { 0.005 } else { 0.2 };

    let target_color = Color::linear_rgb(
        1.0 * target_luminance_factor,
        0.81 * target_luminance_factor,
        0.4 * target_luminance_factor,
    );
    let new_color = player_material
        .emissive
        .mix(&target_color.into(), luminance_rate);

    if target_color
        .to_linear()
        .to_vec4()
        .distance(new_color.to_vec4())
        > 0.1
    {
        player_material.emissive = new_color;
    }

    let mut player_halo_material = player_halo_materials
        .get_mut(&asset_handles.player_halo_material)
        .unwrap();

    let target_alpha = if halo.visible { 0.8 } else { -0.1 };
    let halo_alpha_rate = if halo.visible { 0.006 } else { 0.2 };
    let current_alpha = player_halo_material.base.base_color.alpha();
    let delta_alpha = target_alpha - current_alpha;
    let mut new_alpha = current_alpha + delta_alpha * halo_alpha_rate;

    if delta_alpha.abs() > 0.01 {
        let new_color = player_halo_material.base.base_color.with_alpha(new_alpha);
        player_halo_material.base.base_color = new_color;
    }
}
