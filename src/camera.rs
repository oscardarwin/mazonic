use crate::{
    controller::ControllerState,
    player::Player,
    shape::{loader::PlatonicSolidComponent, platonic_solid::PlatonicSolid},
};
use bevy::{
    math::{NormedVectorSpace, VectorSpace},
    prelude::*,
    window::PrimaryWindow,
};
use itertools::iproduct;

#[derive(Component)]
pub struct MainCamera;

pub fn camera_setup(mut commands: Commands) {
    let charcoal = Color::srgb_u8(57, 62, 70);

    commands
        .spawn(Camera3dBundle {
            camera: Camera {
                clear_color: ClearColorConfig::Custom(charcoal),
                ..Default::default()
            },
            transform: Transform::from_xyz(0.0, 0.0, 4.0)
                .looking_at(Vec3::new(0., 0., 0.), Vec3::Y),
            ..default()
        })
        .insert(IsDefaultUiCamera)
        .insert(MainCamera);

    commands.spawn(DirectionalLightBundle {
        transform: Transform::from_xyz(0.0, 0.0, 6.0).looking_at(Vec3::new(0., 0., 0.), Vec3::Y),
        ..default()
    });
}

pub fn camera_follow_player(
    mut camera_query: Query<&mut Transform, With<MainCamera>>,
    mut light_query: Query<
        &mut Transform,
        (With<DirectionalLight>, Without<Player>, Without<MainCamera>),
    >,
    player_query: Query<&Transform, (With<Player>, Without<MainCamera>)>,
    platonic_solid_component: Query<&PlatonicSolidComponent>,
) {
    let PlatonicSolidComponent(vertices) = platonic_solid_component.single();

    let player_transform = player_query.single();
    let mut camera_transform = camera_query.single_mut();

    let player_translation = player_transform.translation;
    let camera_translation = camera_transform.translation;

    // TODO: use faces for other types of shapes.
    let target_camera_vertex = vertices
        .into_iter()
        .min_by_key(|camera_unit_position| {
            (player_translation.angle_between(camera_unit_position.clone().clone()) * 100.0) as i32
        })
        .unwrap();

    let target_camera_angle = target_camera_vertex.lerp(player_translation, 0.5);

    let step_camera_angle = target_camera_angle.angle_between(camera_translation);

    if step_camera_angle < 0.15 {
        return;
    }

    let target_angle = 0.01 * step_camera_angle;

    let player_camera_axis = camera_translation.cross(target_camera_vertex.clone());

    let rotation = Quat::from_axis_angle(player_camera_axis, target_angle);
    camera_transform.rotate_around(Vec3::new(0.0, 0.0, 0.0), rotation);
    let mut light_transform = light_query.single_mut();

    light_transform.rotate_around(Vec3::new(0.0, 0.0, 0.0), rotation);
}

pub fn camera_dolly(
    mut camera_query: Query<&mut Transform, With<MainCamera>>,
    mut light_query: Query<
        &mut Transform,
        (With<DirectionalLight>, Without<Player>, Without<MainCamera>),
    >,
    primary_window: Query<&Window, With<PrimaryWindow>>,
    mut last_pos: Local<Option<Vec2>>,
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
        let mut light_transform = light_query.single_mut();

        light_transform.rotate_around(Vec3::new(0.0, 0.0, 0.0), -rotation);
        camera_transform.rotate_around(Vec3::new(0.0, 0.0, 0.0), -rotation);
    }
}
