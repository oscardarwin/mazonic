use std::fmt::Debug;

use crate::{
    assets::{
        material_handles::MaterialHandles, mesh_handles::MeshHandles, shaders::PlayerHaloShader,
    },
    effects::player_particles::{PlayerParticleEffect, PlayerParticlesHandle},
    game_settings::GameSettings,
    levels::LevelData,
    maze::maze_mesh_builder::MazeMeshBuilder,
    room::Room,
    shape::loader::SolutionComponent,
    statistics::PlayerPath,
};
use bevy::{math::NormedVectorSpace, pbr::ExtendedMaterial, prelude::*};

use bevy_hanabi::prelude::*;
use bevy_rapier3d::geometry::Collider;

#[derive(Component)]
pub struct Player {
    pub size: f32,
}

#[derive(Component, Debug)]
pub enum PlayerMazeState {
    Node(Room),
    Edge(Room, Room, Vec3),
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
    mut player_halo_query: Query<&PlayerHalo>,
    player_query: Query<&Transform, (With<Player>, Without<PlayerHalo>)>,
    mut player_halo_materials: ResMut<Assets<ExtendedMaterial<StandardMaterial, PlayerHaloShader>>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    asset_handles: Res<MaterialHandles>,
) {
    let Ok(halo) = player_halo_query.get_single_mut() else {
        return;
    };

    let Ok(player_transform) = player_query.get_single() else {
        return;
    };

    let mut player_material = materials.get_mut(&asset_handles.player_handle).unwrap();
    let target_luminance_factor = if halo.visible { 3.0 } else { 1.5 };
    let luminance_rate = if halo.visible { 0.02 } else { 0.2 };

    let target_color_vec3 =
        player_material.base_color.to_linear().to_vec3() * target_luminance_factor;

    let target_color = Color::LinearRgba(LinearRgba::from_vec3(target_color_vec3));
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
        .get_mut(&asset_handles.player_halo_handle)
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

pub fn spawn_player(
    mut commands: Commands,
    mesh_handles: Res<MeshHandles>,
    mesh_builder_query: Query<&MazeMeshBuilder>,
    solution_query: Query<&SolutionComponent>,
    settings: Res<GameSettings>,
    material_handles: Res<MaterialHandles>,
    player_particle_handle_query: Query<&PlayerParticlesHandle>,
) {
    let Ok(mesh_builder) = mesh_builder_query.get_single() else {
        return;
    };
    let Ok(SolutionComponent(solution)) = solution_query.get_single() else {
        return;
    };
    let PlayerParticlesHandle(effect_handle) = player_particle_handle_query.single();

    let initial_node = solution.first().unwrap().clone();
    let player_size = mesh_builder.player_mesh_size();

    let height_above_node = settings.player_elevation + player_size;
    let player_transform = compute_initial_player_transform(initial_node, height_above_node);

    commands
        .spawn((
            player_transform,
            Player { size: player_size },
            PlayerMazeState::Node(initial_node),
            PlayerPath::default(),
            Collider::ball(player_size),
            LevelData,
        ))
        .with_children(|parent| {
            parent.spawn((
                Transform::IDENTITY.with_scale(Vec3::splat(2.0 * player_size)),
                Mesh3d(mesh_handles.player.clone()),
                MeshMaterial3d(material_handles.player_handle.clone()),
            ));

            parent.spawn((
                Mesh3d(mesh_handles.player_halo.clone()),
                MeshMaterial3d(material_handles.player_halo_handle.clone()),
                Transform::IDENTITY.with_scale(Vec3::splat(2.0 * player_size * 1.1)),
                PlayerHalo { visible: true },
            ));

            parent
                .spawn(ParticleEffectBundle {
                    effect: ParticleEffect::new(effect_handle.clone()),
                    transform: Transform::IDENTITY,
                    ..Default::default()
                })
                .insert(PlayerParticleEffect);
        });
}

fn compute_initial_player_transform(start_node: Room, player_elevation: f32) -> Transform {
    let face_normal = start_node.face().normal();

    Transform::IDENTITY
        .looking_at(face_normal.any_orthogonal_vector(), face_normal)
        .with_translation(start_node.position() + player_elevation * face_normal)
}
