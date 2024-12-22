use crate::{controller::ControllerState, player::Player};
use bevy::{math::NormedVectorSpace, prelude::*, window::PrimaryWindow};
use itertools::iproduct;

#[derive(Default)]
pub struct PlatonicCamera;

impl Plugin for PlatonicCamera {
    fn build(&self, app: &mut App) {
        app.add_systems(
            Update,
            follow_player.run_if(in_state(ControllerState::IdlePostSolve)),
        )
        .add_systems(Update, view.run_if(in_state(ControllerState::Viewing)))
        .add_systems(Startup, setup);
    }
}

fn setup(mut commands: Commands) {
    let charcoal = Color::srgb_u8(57, 62, 70);

    commands.spawn(Camera3dBundle {
        camera: Camera {
            // clear the whole viewport with the given color
            clear_color: ClearColorConfig::Custom(charcoal),
            ..Default::default()
        },
        transform: Transform::from_xyz(0.0, 0.0, 3.0).looking_at(Vec3::new(0., 0., 0.), Vec3::Y),
        ..default()
    });

    commands.spawn(DirectionalLightBundle {
        transform: Transform::from_xyz(0.0, 0.0, 5.0).looking_at(Vec3::new(0., 0., 0.), Vec3::Y),
        ..default()
    });
}

fn follow_player(
    mut camera_query: Query<&mut Transform, With<Camera>>,
    mut light_query: Query<
        &mut Transform,
        (With<DirectionalLight>, Without<Player>, Without<Camera>),
    >,
    player_query: Query<&Transform, (With<Player>, Without<Camera>)>,
) {
    let player_transform = player_query.single();
    let mut camera_transform = camera_query.single_mut();

    let player_translation = player_transform.translation;
    let camera_translation = camera_transform.translation;

    // use faces for other types of shapes.
    let target_camera_unit_position = iproduct!(-1..=1, -1..=1, -1..=1)
        .map(|(x, y, z)| Vec3::new(x as f32, y as f32, z as f32))
        .filter(|vec| vec.norm() > 0.5)
        .min_by_key(|camera_unit_position| {
            (player_translation.angle_between(camera_unit_position.clone()) * 100.0) as i32
        })
        .unwrap();

    let target_camera_angle = target_camera_unit_position.angle_between(camera_translation);

    if target_camera_angle < 0.15 {
        return;
    }

    let target_angle = 0.01 * target_camera_angle;

    let player_camera_axis = camera_translation.cross(target_camera_unit_position);

    let rotation = Quat::from_axis_angle(player_camera_axis, target_angle);
    camera_transform.rotate_around(Vec3::new(0.0, 0.0, 0.0), rotation);
    let mut light_transform = light_query.single_mut();

    light_transform.rotate_around(Vec3::new(0.0, 0.0, 0.0), rotation);
}

fn view(
    mut camera_query: Query<&mut Transform, With<Camera>>,
    mut light_query: Query<
        &mut Transform,
        (With<DirectionalLight>, Without<Player>, Without<Camera>),
    >,
    primary_window: Query<&Window, With<PrimaryWindow>>,
    mut last_pos: Local<Option<Vec2>>,
) {
    let Ok(window) = primary_window.get_single() else {
        return;
    };

    let Some(cursor_position) = window.cursor_position() else {
        // if the cursor is not inside the window, we can't do anything
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
        + camera_transform.up() * delta_device_pixels.y;
    let axis = delta
        .cross(camera_transform.forward().as_vec3())
        .normalize();

    if axis.norm() > 0.01 {
        // println!("rotate_around: {:?} with delta: {:?}", axis, delta);
        let rotation = Quat::from_axis_angle(axis, delta.norm() / 150.0);
        let mut light_transform = light_query.single_mut();

        light_transform.rotate_around(Vec3::new(0.0, 0.0, 0.0), -rotation);
        camera_transform.rotate_around(Vec3::new(0.0, 0.0, 0.0), -rotation);
    }
}
