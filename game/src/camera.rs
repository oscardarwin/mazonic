use crate::{
    constants::PHI,
    game_settings::GameSettings,
    player::{Player, PlayerMazeState},
    shape::loader::{GameLevel, Shape},
};
use bevy::{math::NormedVectorSpace, prelude::*, window::PrimaryWindow};
use bevy_rapier3d::na::ComplexField;

#[derive(Component)]
pub struct MainCamera;

pub fn camera_setup(mut commands: Commands, game_settings: Res<GameSettings>) {
    commands
        .spawn(Camera3dBundle {
            camera: Camera {
                clear_color: ClearColorConfig::Custom(game_settings.palette.background_color),
                ..Default::default()
            },
            transform: Transform::from_xyz(0.0, 0.0, game_settings.camera_distance)
                .looking_at(Vec3::new(0., 0., 0.), Vec3::Y),
            ..default()
        })
        .insert(IsDefaultUiCamera)
        .insert(MainCamera);
}

pub fn camera_follow_player(
    mut camera_query: Query<&mut Transform, With<MainCamera>>,
    player_query: Query<&PlayerMazeState, (With<Player>, Without<MainCamera>)>,
    game_settings: Res<GameSettings>,
) {
    let Ok(player_maze_state) = player_query.get_single() else {
        return;
    };

    let mut camera_transform = camera_query.single_mut();

    let camera_translation = camera_transform.translation;

    let target_camera_vector = get_target_vector(player_maze_state);

    let step_camera_angle = target_camera_vector.angle_between(camera_translation);

    if step_camera_angle < 0.15 {
        return;
    }

    let target_angle = game_settings.camera_follow_speed * step_camera_angle;

    let player_camera_axis = camera_translation.cross(target_camera_vector.clone());

    let rotation = Quat::from_axis_angle(player_camera_axis, target_angle);

    rotate_transform(camera_transform, rotation);
}

fn get_target_vector(player_maze_state: &PlayerMazeState) -> Vec3 {
    match player_maze_state {
        PlayerMazeState::Node(node) => node.face().normal(),
        PlayerMazeState::Edge(from_node, to_node, _) => {
            let from_face_normal = from_node.face().normal();
            let to_face_normal = to_node.face().normal();

            from_face_normal.lerp(to_face_normal, 0.5)
        }
    }
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
