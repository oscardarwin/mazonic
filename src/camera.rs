use std::f32::consts::PI;

use crate::{
    controller::ControllerState,
    player::Player,
    shape::cube::{
        self,
        maze::{BorderType, CubeMaze, CubeNode, Edge},
    },
};
#[cfg(not(target_arch = "wasm32"))]
use bevy::pbr::wireframe::{WireframeConfig, WireframePlugin};
use bevy::{
    app::Plugins,
    color::palettes::basic::SILVER,
    math::{vec2, NormedVectorSpace},
    prelude::*,
    render::{
        mesh::{Indices, PrimitiveTopology},
        render_asset::RenderAssetUsages,
        render_resource::{Extent3d, TextureDimension, TextureFormat},
    },
    window::PrimaryWindow,
};
use bevy_rapier3d::{
    geometry::Collider,
    pipeline::QueryFilter,
    plugin::{NoUserData, RapierContext, RapierPhysicsPlugin},
};

#[derive(Default)]
pub struct PlatonicCamera;

impl Plugin for PlatonicCamera {
    fn build(&self, app: &mut App) {
        app.init_state::<ControllerState>()
            .add_systems(
                Update,
                follow_player.run_if(in_state(ControllerState::Idleing)),
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
    camera_query: Query<(&GlobalTransform, &Camera)>,
    primary_window: Query<&Window, With<PrimaryWindow>>,
) {
}

fn view(
    mut camera_query: Query<&mut Transform, With<Camera>>,
    mut light_query: Query<
        &mut Transform,
        (With<DirectionalLight>, Without<Player>, Without<Camera>),
    >,
    primary_window: Query<&Window, With<PrimaryWindow>>,
    mouse_buttons: Res<ButtonInput<MouseButton>>,
    mut last_pos: Local<Option<Vec2>>,
    mut next_controller_state: ResMut<NextState<ControllerState>>,
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

        light_transform.rotate_around(Vec3::new(0.0, 0.0, 0.0), rotation);
        camera_transform.rotate_around(Vec3::new(0.0, 0.0, 0.0), rotation);
    }
}
