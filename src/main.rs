#[cfg(not(target_arch = "wasm32"))]
use bevy::pbr::wireframe::WireframePlugin;
use bevy::prelude::*;
use bevy_rapier3d::plugin::{NoUserData, RapierPhysicsPlugin};
use camera::PlatonicCamera;
use controller::Controller;
use player::{setup_player, PlayerPlugin};
use shape::cube::{self, maze::CubeMaze};

mod camera;
mod controller;
mod model;
mod player;
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
        .add_plugins(PlatonicCamera::default())
        .add_plugins(PlayerPlugin::default())
        .add_systems(Startup, (load_maze, cube::spawn.after(load_maze)))
        .run();
}

fn load_maze(mut commands: Commands) {
    let maze = CubeMaze::build(3, 2.0, 0.2);
    commands.insert_resource(maze);
}

fn setup(
    commands: Commands,
    meshes: ResMut<Assets<Mesh>>,
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
}