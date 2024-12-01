use std::f32::consts::PI;

#[cfg(not(target_arch = "wasm32"))]
use bevy::pbr::wireframe::{WireframeConfig, WireframePlugin};
use bevy::{
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
use shape::cube::{
    self,
    maze::{CubeMaze, CubeNode},
};

mod model;
mod shape;

fn main() {
    App::new()
        .add_plugins((
            DefaultPlugins.set(ImagePlugin::default_nearest()),
            #[cfg(not(target_arch = "wasm32"))]
            WireframePlugin,
        ))
        .add_plugins(RapierPhysicsPlugin::<NoUserData>::default())
        .add_systems(
            Startup,
            (
                load_maze,
                setup.after(load_maze),
                cube::spawn.after(load_maze),
            ),
        )
        .add_systems(Update, input)
        .run();
}

/// A marker component for our shapes so we can query them separately from the ground plane
#[derive(Component)]
struct Player;

fn load_maze(mut commands: Commands) {
    let maze = CubeMaze::build(3, 2.0);
    commands.insert_resource(maze);
}

fn setup(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    cube_maze: Res<CubeMaze>,
) {
    let white = Color::srgb_u8(247, 247, 0);
    let beige = Color::srgb_u8(242, 231, 213);
    let green = Color::srgb_u8(109, 152, 134);
    let charcoal = Color::srgb_u8(57, 62, 70);

    let white_material = materials.add(StandardMaterial::from_color(white));
    let beige_material = materials.add(StandardMaterial::from_color(beige));
    let green_material = materials.add(StandardMaterial::from_color(green));
    let charcoal_material = materials.add(StandardMaterial::from_color(charcoal));

    let player_transform =
        compute_initial_player_transform(cube_maze.maze.solution.first().unwrap().clone());
    let player_shape = Sphere::new(0.1);
    let player_mesh = meshes.add(player_shape);

    commands
        .spawn(PbrBundle {
            mesh: player_mesh,
            material: white_material.clone(),
            transform: player_transform,
            ..default()
        })
        .insert(Player)
        .insert(Collider::ball(player_shape.radius));

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

fn compute_initial_player_transform(start_node: CubeNode) -> Transform {
    let face_normal = start_node.face.normal();

    Transform::IDENTITY
        .looking_at(face_normal.any_orthogonal_vector(), face_normal)
        .with_translation(start_node.position + 0.5 * face_normal)
}

fn input(
    mut camera_query: Query<(&mut Transform, &GlobalTransform, &Camera), Without<Player>>,
    mut light_query: Query<
        &mut Transform,
        (With<DirectionalLight>, Without<Player>, Without<Camera>),
    >,
    primary_window: Query<&Window, With<PrimaryWindow>>,
    nodes: Query<
        (&Transform, &Handle<Mesh>),
        (With<Player>, Without<Camera>, Without<DirectionalLight>),
    >,
    mouse_buttons: Res<ButtonInput<MouseButton>>,
    rapier_context: Res<RapierContext>,
    mut last_pos: Local<Option<Vec2>>,
) {
    let Ok(window) = primary_window.get_single() else {
        return;
    };
    let window_size = window.size();

    let Some(cursor_position) = window.cursor_position() else {
        // if the cursor is not inside the window, we can't do anything
        return;
    };

    let cursor_position_vec = vec2(cursor_position.x, cursor_position.y);

    let (mut camera_transform, camera_global_transform, camera) = camera_query.single_mut();

    let Some(ray) = camera.viewport_to_world(camera_global_transform, cursor_position) else {
        // if it was impossible to compute for whatever reason; we can't do anything
        return;
    };

    let max_toi = 4.0;
    if let Some((entity, toi)) = rapier_context.cast_ray(
        ray.origin,
        ray.direction.into(),
        max_toi,
        true,
        QueryFilter::default(),
    ) {
        // The first collider hit has the entity `entity` and it hit after
        // the ray travelled a distance equal to `ray_dir * toi`.
        let hit_point = ray.origin + ray.direction * toi;
        println!("Entity {:?} hit at point {}", entity, hit_point);
    }
    let delta_device_pixels = cursor_position_vec - last_pos.unwrap_or(cursor_position_vec);

    let delta = if mouse_buttons.pressed(MouseButton::Left)
        && !mouse_buttons.just_pressed(MouseButton::Left)
    {
        delta_device_pixels
    } else {
        Vec2::ZERO
    };

    let delta = camera_transform.right() * delta.x + camera_transform.up() * delta.y;
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
    *last_pos = Some(cursor_position_vec);
}
