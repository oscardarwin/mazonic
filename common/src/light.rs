use bevy::prelude::*;

use crate::camera::MainCamera;

const LERP_FACTOR: f32 = 0.4;
const CAMERA_OFFSET_FACTOR: f32 = 1.2;

#[derive(Component)]
pub struct MainLight;

pub fn setup(mut commands: Commands) {
    commands
        .spawn(DirectionalLight {
            illuminance: 8_000.,
            ..Default::default()
        })
        .insert(Transform::from_xyz(0.0, 0.0, 20.0).looking_at(Vec3::new(0., 0., 0.), Vec3::Y))
        .insert(MainLight);
}

pub fn follow_camera(
    mut camera_query: Query<&Transform, With<MainCamera>>,
    mut light_query: Query<&mut Transform, (With<MainLight>, Without<MainCamera>)>,
) {
    let Ok(camera_transform) = camera_query.get_single() else {
        return;
    };

    let Ok(mut light_transform) = light_query.get_single_mut() else {
        return;
    };

    let rotation_delta = camera_transform.rotation - light_transform.rotation;
    light_transform.rotation = light_transform
        .rotation
        .slerp(camera_transform.rotation, LERP_FACTOR);

    let target_translation = camera_transform.translation * CAMERA_OFFSET_FACTOR;

    light_transform.translation = light_transform
        .translation
        .lerp(target_translation, LERP_FACTOR);
}
