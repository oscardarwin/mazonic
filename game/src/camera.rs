use crate::{
    constants::PHI,
    game_settings::GameSettings,
    level_selector::Selectable,
    player::{Player, PlayerMazeState},
    shape::loader::{GameLevel, Shape},
};
use bevy::{
    math::{NormedVectorSpace, VectorSpace},
    prelude::*,
    window::PrimaryWindow,
};
use bevy_rapier3d::na::ComplexField;

const CAMERA_MOVE_THRESHOLD: f32 = 0.0005;

#[derive(Component)]
pub struct MainCamera;

#[derive(Component)]
pub struct CameraTarget {
    translation: Vec3,
    up: Vec3,
    looking_at: Vec3,
}

pub fn camera_setup(mut commands: Commands, game_settings: Res<GameSettings>) {
    let translation = Vec3::new(0., 0., game_settings.camera_distance);
    let looking_at = Vec3::ZERO;
    let up = Vec3::Y;

    let transform =
        Transform::from_translation(translation.clone()).looking_at(looking_at.clone(), up.clone());

    commands
        .spawn(Camera {
            clear_color: ClearColorConfig::Custom(game_settings.palette.background_color),
            ..Default::default()
        })
        .insert(Camera3d::default())
        .insert(transform.clone())
        .insert(CameraTarget {
            translation,
            up,
            looking_at,
        })
        .insert(IsDefaultUiCamera)
        .insert(MainCamera);
}

pub fn camera_follow_player(
    mut camera_target_query: Query<&mut CameraTarget, With<MainCamera>>,
    player_query: Query<&PlayerMazeState, (With<Player>, Without<MainCamera>)>,
    game_settings: Res<GameSettings>,
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

            from_face_normal.lerp(to_face_normal, 0.5)
        }
    };

    camera_target.translation = target_unit_translation * game_settings.camera_distance;
}

pub fn camera_move_to_closest_selector_face(
    mut camera_target_query: Query<(&mut CameraTarget, &Transform), With<MainCamera>>,
    selectable: Query<&Transform, (With<Selectable>, Without<MainCamera>)>,
    game_settings: Res<GameSettings>,
) {
    let (mut camera_target, camera_transform) = camera_target_query.single_mut();

    let camera_forward = camera_transform.forward();

    let Some(closest_face_transform) = selectable.iter().min_by_key(|selectable_transform| {
        let face_normal = -Vec3::from(selectable_transform.forward());
        (camera_forward.dot(face_normal) * 100.0) as i32
    }) else {
        return;
    };

    camera_target.translation = -closest_face_transform.forward() * game_settings.camera_distance;
    camera_target.up = *closest_face_transform.right();
}

pub fn camera_move_to_target(
    target_query: Query<&CameraTarget>,
    mut camera_query: Query<&mut Transform, With<MainCamera>>,
    game_settings: Res<GameSettings>,
) {
    let Ok(CameraTarget {
        translation,
        up,
        looking_at,
    }) = target_query.get_single()
    else {
        return;
    };

    let mut camera_transform = camera_query.single_mut();

    let target_transform = Transform::IDENTITY
        .with_translation(translation.clone())
        .looking_at(*looking_at, *up);

    let camera_follow_speed = game_settings.camera_follow_speed;
    let new_rotation = camera_transform
        .rotation
        .lerp(target_transform.rotation, camera_follow_speed);

    let new_translation = camera_transform
        .translation
        .lerp(*translation, camera_follow_speed);

    if new_translation.distance(camera_transform.translation) < CAMERA_MOVE_THRESHOLD
        && new_rotation.abs_diff_eq(camera_transform.rotation, CAMERA_MOVE_THRESHOLD)
    {
        return;
    }

    camera_transform.translation = new_translation;
    camera_transform.rotation = new_rotation;
}

pub fn camera_dolly(
    mut camera_query: Query<&mut Transform, With<MainCamera>>,
    primary_window: Query<&Window, With<PrimaryWindow>>,
    mut last_pos: Local<Option<Vec2>>,
    game_settings: Res<GameSettings>,
) {
    let Ok(window) = primary_window.get_single() else {
        return;
    };

    let Some(cursor_position) = window.cursor_position() else {
        return;
    };

    let previous_cursor_position = last_pos.clone();
    *last_pos = Some(cursor_position);

    let delta_device_pixels = cursor_position - previous_cursor_position.unwrap_or(cursor_position);

    if delta_device_pixels.norm() > 20.0 {
        return;
    }

    let mut camera_transform = camera_query.single_mut();
    let delta = camera_transform.right() * delta_device_pixels.x
        - camera_transform.up() * delta_device_pixels.y;
    let axis = delta
        .cross(camera_transform.forward().as_vec3())
        .normalize();

    if axis.norm() > 0.01 {
        let angle = delta.norm() / 150.0;

        let rotation = Quat::from_axis_angle(axis, angle);

        rotate_transform(camera_transform, rotation);
    }
}

fn rotate_transform(mut transform: Mut<Transform>, rotation: Quat) {
    let distance = transform.translation.norm();

    transform.rotate_around(Vec3::new(0.0, 0.0, 0.0), -rotation);

    let up_vector = transform.up();
    transform.look_at(Vec3::new(0., 0., 0.), up_vector);
    transform.translation = transform.translation.normalize() * distance;
}

pub fn update_camera_on_window_resize(
    mut camera_query: Query<(&Camera, &mut Transform, &GlobalTransform), With<MainCamera>>,
    level_query: Query<&GameLevel>,
) {
    let Ok((camera, mut camera_transform, global_transform)) = camera_query.get_single_mut() else {
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

    let target_camera_x_axis_point = camera_transform.right().normalize() * target_view_radius;
    let target_camera_y_axis_point = camera_transform.up().normalize() * target_view_radius;

    let Some(target_x_ndc) = camera.world_to_ndc(global_transform, target_camera_x_axis_point)
    else {
        return;
    };
    let Some(target_y_ndc) = camera.world_to_ndc(global_transform, target_camera_y_axis_point)
    else {
        return;
    };

    let max_abs_ndc = target_x_ndc.abs().max(target_y_ndc.abs()).max_element();

    if max_abs_ndc <= 1.1 {
        return;
    }

    let ndc_delta = max_abs_ndc - 1.;
    let distance_scaling_factor = 0.2 * ndc_delta;

    camera_transform.translation *= (distance_scaling_factor + 1.0);
}
