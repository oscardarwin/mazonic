use crate::{
    constants::PHI,
    game_settings::GameSettings,
    game_state::GameState,
    level_selector::{SaveData, SelectableLevel},
    levels::{GameLevel, Shape},
    player::{Player, PlayerMazeState},
    shape::loader::LevelData,
};
use bevy::{
    color::palettes::css::{BLUE, RED},
    math::{NormedVectorSpace, VectorSpace},
    prelude::*,
    window::{PrimaryWindow, WindowResized},
};
use bevy_rapier3d::na::ComplexField;

const CAMERA_MOVE_THRESHOLD: f32 = 0.001;

#[derive(Component)]
pub struct MainCamera;

#[derive(Component)]
pub struct CameraTarget {
    pub translation_dir: Vec3,
    pub translation_norm: f32,
    pub up: Vec3,
    pub looking_at: Vec3,
}

pub fn camera_setup(mut commands: Commands, game_settings: Res<GameSettings>) {
    let translation_dir = Vec3::Z;
    let translation_norm = game_settings.camera_distance;
    let looking_at = Vec3::ZERO;
    let up = Vec3::Y;

    let transform = Transform::from_translation(translation_dir * translation_norm)
        .looking_at(looking_at.clone(), up.clone());

    commands
        .spawn(Camera {
            clear_color: ClearColorConfig::Custom(game_settings.palette.background_color),
            ..Default::default()
        })
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

pub fn camera_follow_player(
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

            from_face_normal.lerp(to_face_normal, 0.5).normalize()
        }
    };

    println!(
        "Setting new camera target direction: {:?}",
        target_unit_translation
    );

    camera_target.translation_dir = target_unit_translation;
}

pub fn camera_move_to_target(
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

    let camera_follow_speed = game_settings.camera_follow_speed;
    let normalized_new_translation = camera_transform
        .translation
        .lerp(*translation_dir, camera_follow_speed)
        .normalize();

    let new_translation_norm = FloatExt::lerp(
        camera_transform.translation.norm(),
        *translation_norm,
        camera_follow_speed,
    );
    let new_translation = normalized_new_translation * new_translation_norm;

    if new_translation.distance(translation_dir * translation_norm) < CAMERA_MOVE_THRESHOLD {
        return;
    }

    let new_up = camera_transform.up().lerp(*up, camera_follow_speed);

    camera_transform.translation = new_translation;
    camera_transform.look_at(Vec3::ZERO, new_up);
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

#[derive(SubStates, Default, Debug, Clone, PartialEq, Eq, Hash)]
#[source(GameState = GameState::Playing)]
pub enum CameraResizeState {
    Resizing,
    #[default]
    Fixed,
}

pub fn trigger_camera_resize_on_window_change(
    mut resize_reader: EventReader<WindowResized>,
    mut next_camera_resize_state: ResMut<NextState<CameraResizeState>>,
) {
    for e in resize_reader.read() {
        println!("Resizing camera on window size change");
        next_camera_resize_state.set(CameraResizeState::Resizing);
    }
}

pub fn trigger_camera_resize_on_level_change(
    mut next_camera_resize_state: ResMut<NextState<CameraResizeState>>,
) {
    next_camera_resize_state.set(CameraResizeState::Resizing);
}

pub fn update_camera_distance(
    mut camera_query: Query<
        (&Camera, &mut CameraTarget, &Transform, &GlobalTransform),
        With<MainCamera>,
    >,
    level_query: Query<&GameLevel>,
    mut next_camera_resize_state: ResMut<NextState<CameraResizeState>>,
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
        Shape::Tetrahedron => 1.5_f32.sqrt(),
        Shape::Cube => 3.0_f32.sqrt(),
        Shape::Octahedron => 2.0_f32.sqrt(),
        Shape::Dodecahedron => 3.0_f32.sqrt() * PHI,
        Shape::Icosahedron => PHI * (3.0 - PHI).sqrt(),
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

    next_camera_resize_state.set(CameraResizeState::Fixed);
    camera_target.translation_norm = camera_target.translation_norm * max_abs_ndc;
    println!(
        "Adjusting camera norm to: {:?}, max absolute normalized device coordinate: {:?}",
        camera_target.translation_norm, max_abs_ndc
    );
}
