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
use controller::Controller;
use shape::cube::{
    self,
    maze::{CubeMaze, CubeNode, Edge, Face},
};

mod controller;
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
        .add_plugins(Controller::default())
        .add_systems(
            Startup,
            (
                load_maze,
                setup.after(load_maze),
                cube::spawn.after(load_maze),
            ),
        )
        .run();
}

/// A marker component for our shapes so we can query them separately from the ground plane
#[derive(Component)]
struct Player;

#[derive(Component, Debug)]
enum PlayerMazePosition {
    Node(CubeNode),
    Edge(CubeNode, CubeNode),
}

fn load_maze(mut commands: Commands) {
    let maze = CubeMaze::build(3, 2.0, 0.2);
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

    let initial_node = cube_maze.maze.solution.first().unwrap().clone();
    let player_transform = compute_initial_player_transform(initial_node);
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
        .insert(PlayerMazePosition::Node(initial_node))
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
        .with_translation(start_node.position + 0.2 * face_normal)
}
