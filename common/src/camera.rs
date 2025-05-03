use std::collections::VecDeque;

use crate::{
    constants::PHI,
    controller_screen_position::ControllerScreenPosition,
    game_settings::GameSettings,
    game_state::GameState,
    game_systems::SystemHandles,
    level_selector::SelectableLevel,
    levels::{GameLevel, PuzzleEntityMarker, Shape},
    player::{Player, PlayerMazeState},
};
use bevy::{
    color::palettes::css::{BLUE, RED},
    ecs::system::SystemId,
    math::{NormedVectorSpace, VectorSpace},
    prelude::*,
    window::{PrimaryWindow, WindowResized},
};
use bevy_rapier3d::na::ComplexField;
use ringbuffer::RingBuffer;

const CAMERA_MOVE_THRESHOLD: f32 = 0.005;
pub const CAMERA_MAX_NORM: f32 = 10.0;
pub const CAMERA_MIN_NORM: f32 = 2.4;

#[derive(Component)]
pub struct MainCamera;

#[derive(Component, Debug)]
pub struct CameraTarget {
    pub translation_dir: Vec3,
    pub translation_norm: f32,
    pub up: Vec3,
    pub looking_at: Vec3,
}

impl CameraTarget {
    pub fn set_zoom(&mut self, zoom: f32) {
        self.translation_norm = zoom.clamp(CAMERA_MIN_NORM, CAMERA_MAX_NORM);
    }
}

#[derive(Component, Debug, Clone)]
pub struct DollyAngularMotion {
    axis: Vec3,
    angular_velocity: f32,
}

const NUM_STORED_POSITIONS: usize = 5;

#[derive(Component, Debug, Clone, Default)]
pub struct DollyScreenPositions(ringbuffer::ConstGenericRingBuffer<Vec2, NUM_STORED_POSITIONS>);

pub fn setup(mut commands: Commands, game_settings: Res<GameSettings>) {
    let translation_dir = Vec3::Z;
    let translation_norm = game_settings.camera_distance;
    let looking_at = Vec3::ZERO;
    let up = Vec3::Y;

    let transform = Transform::from_translation(translation_dir * translation_norm)
        .looking_at(looking_at.clone(), up.clone());

    commands
        .spawn(Camera {
            hdr: true,
            clear_color: ClearColorConfig::Custom(game_settings.palette.background_color),
            ..Default::default()
        })
        .insert(DollyAngularMotion {
            axis: Vec3::X,
            angular_velocity: 0.0,
        })
        .insert(DollyScreenPositions::default())
        .insert(Projection::Perspective(PerspectiveProjection {
            near: 1.0,
            far: 2.5,
            ..default()
        }))
        .insert(Camera3d::default())
        .insert(transform.clone())
        .insert(CameraTarget {
            translation_dir,
            translation_norm,
            up,
            looking_at,
        })
        .insert(IsDefaultUiCamera)
        .insert(MainCamera);
}

pub fn follow_player(
    mut camera_target_query: Query<&mut CameraTarget, With<MainCamera>>,
    player_query: Query<&PlayerMazeState, (With<Player>, Without<MainCamera>)>,
) {
    let Ok(player_maze_state) = player_query.get_single() else {
        return;
    };

    let mut camera_target = camera_target_query.single_mut();

    let target_unit_translation = match player_maze_state {
        PlayerMazeState::Node(node) => node.face().normal(),
        PlayerMazeState::Edge(from_node, to_node, _) => {
            let from_face_normal = from_node.face().normal();
            let to_face_normal = to_node.face().normal();

            from_face_normal.midpoint(to_face_normal).normalize()
        }
    };
    
    camera_target.translation_dir = target_unit_translation;
}

pub fn camera_rotate_to_target(
    target_query: Query<&CameraTarget>,
    mut camera_query: Query<&mut Transform, With<MainCamera>>,
    game_settings: Res<GameSettings>,
) {
    let Ok(CameraTarget {
        translation_dir,
        translation_norm,
        up,
        looking_at,
    }) = target_query.get_single()
    else {
        return;
    };

    let mut camera_transform = camera_query.single_mut();
    if camera_transform
        .translation
        .distance(translation_dir * translation_norm)
        < CAMERA_MOVE_THRESHOLD
    {
        return;
    }

    let camera_follow_speed = game_settings.camera_follow_speed;

    let normalized_new_translation = camera_transform
        .translation
        .lerp(*translation_dir, camera_follow_speed)
        .normalize();

    let new_translation = normalized_new_translation * camera_transform.translation.norm();

    let new_up = camera_transform.up().lerp(*up, camera_follow_speed);

    camera_transform.translation = new_translation;
    camera_transform.look_at(Vec3::ZERO, new_up);
}

pub fn camera_zoom_to_target(
    target_query: Query<&CameraTarget>,
    mut camera_query: Query<&mut Transform, With<MainCamera>>,
    game_settings: Res<GameSettings>,
) {
    let Ok(CameraTarget {
        translation_dir,
        translation_norm,
        up,
        looking_at,
    }) = target_query.get_single()
    else {
        return;
    };

    let mut camera_transform = camera_query.single_mut();
    let current_camera_norm = camera_transform.translation.norm();
    if (current_camera_norm - translation_norm).abs() < CAMERA_MOVE_THRESHOLD {
        return;
    }

    let new_translation_norm = FloatExt::lerp(
        camera_transform.translation.norm(),
        *translation_norm,
        game_settings.camera_zoom_speed,
    );

    camera_transform.translation *= new_translation_norm / current_camera_norm;
}

pub fn update_dolly(
    mut camera_query: Query<(&mut Transform, &mut DollyAngularMotion), With<MainCamera>>,
) {
    let (mut transform, mut dolly_rotation_target) = camera_query.single_mut();
    
    if dolly_rotation_target.angular_velocity.abs() < 0.001 {
        return;
    }

    let rotation = Quat::from_axis_angle(dolly_rotation_target.axis, -dolly_rotation_target.angular_velocity);

    dolly_rotation_target.angular_velocity *= 0.95;

    let distance = transform.translation.norm();

    transform.rotate_around(Vec3::new(0.0, 0.0, 0.0), -rotation);

    let up_vector = transform.up();
    transform.look_at(Vec3::new(0., 0., 0.), up_vector);
    transform.translation = transform.translation.normalize() * distance;
}

pub fn reset_dolly_screen_positions(
    mut dolly_screen_positions_query: Query<&mut DollyScreenPositions>,
) {
    let Ok(mut dolly_screen_positions) = dolly_screen_positions_query.get_single_mut() else {
        return;
    };

    dolly_screen_positions.0.clear();
}

pub fn camera_dolly(
    controller_screen_position_query: Query<
        &ControllerScreenPosition,
        Changed<ControllerScreenPosition>,
    >,
    mut camera_query: Query<
        (
            &Transform,
            &mut DollyAngularMotion,
            &mut DollyScreenPositions,
        ),
        With<MainCamera>,
    >,
    game_settings: Res<GameSettings>,
) {
    let Ok(ControllerScreenPosition::Position(cursor_position)) =
        controller_screen_position_query.get_single()
    else {
        return;
    };

    let (camera_transform, mut dolly_rotation_target, mut dolly_screen_positions) = camera_query.single_mut();
    dolly_screen_positions.0.push(*cursor_position);

    let average_delta_device_pixels = get_average_delta(&dolly_screen_positions.0);

    dolly_rotation_target.angular_velocity = if average_delta_device_pixels.norm() < 1.0 {
        0.0
    } else {
        average_delta_device_pixels.norm() / 90.0
    };

    if dolly_rotation_target.angular_velocity < 0.001 {
        return;
    }

    let delta = camera_transform.right() * average_delta_device_pixels.x
        - camera_transform.up() * average_delta_device_pixels.y;
    let axis = delta
        .cross(camera_transform.forward().as_vec3())
        .normalize();

    dolly_rotation_target.axis = axis;
}

fn get_average_delta(
    last_positions: &ringbuffer::ConstGenericRingBuffer<Vec2, NUM_STORED_POSITIONS>,
) -> Vec2 {
    let size = (last_positions.len() - 1).max(1) as f32;
    (last_positions.back().unwrap() - last_positions.front().unwrap()) / size
}

pub fn trigger_camera_resize_on_window_change(
    mut resize_reader: EventReader<WindowResized>,
    mut commands: Commands,
    systems: Res<SystemHandles>,
    mut previous_window_size: Local<Option<(f32, f32)>>,
) {
    for e in resize_reader.read() {
        println!("Resizing camera on window size change");
        let trigger_resize = match *previous_window_size {
            Some((previous_width, previous_height)) => {
                let abs_width_delta = (e.width - previous_width).abs();
                let abs_height_delta = (e.height - previous_height).abs();
                let max_delta = f32::max(abs_width_delta, abs_height_delta);
                max_delta > 1.0
            }
            None => true,
        };

        if trigger_resize {
            commands.run_system(systems.resize_camera_distance);
        }

        *previous_window_size = Some((e.width, e.height));
    }
}

pub fn update_distance(
    mut camera_query: Query<
        (&Camera, &mut CameraTarget, &Transform, &GlobalTransform),
        With<MainCamera>,
    >,
    level_query: Query<&GameLevel>,
) {
    let Ok((camera, mut camera_target, transform, global_transform)) =
        camera_query.get_single_mut()
    else {
        return;
    };

    let Ok(level) = level_query.get_single() else {
        return;
    };

    let circumradius_factor = match &level.shape {
        Shape::Tetrahedron(_) => 1.5_f32.sqrt(),
        Shape::Cube(_) => 3.0_f32.sqrt(),
        Shape::Octahedron(_) => 2.0_f32.sqrt(),
        Shape::Dodecahedron(_) => 3.0_f32.sqrt() * PHI,
        Shape::Icosahedron(_) => PHI * (3.0 - PHI).sqrt(),
    };

    let circumradius = circumradius_factor / 2.0;
    let target_view_radius = circumradius * 1.3;

    let target_camera_y_axis_point = transform.up().normalize() * target_view_radius;
    let target_camera_x_axis_point = transform.right().normalize() * target_view_radius;

    let Some(target_x_ndc) = camera.world_to_ndc(global_transform, target_camera_x_axis_point)
    else {
        return;
    };
    let Some(target_y_ndc) = camera.world_to_ndc(global_transform, target_camera_y_axis_point)
    else {
        return;
    };

    let max_abs_ndc = target_x_ndc.abs().max(target_y_ndc.abs()).max_element();

    let new_zoom = camera_target.translation_norm * max_abs_ndc;

    camera_target.set_zoom(new_zoom);
    println!(
        "Adjusting camera norm to: {:?}, max absolute normalized device coordinate: {:?}",
        camera_target.translation_norm, max_abs_ndc
    );
}
