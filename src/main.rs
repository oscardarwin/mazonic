#[cfg(not(target_arch = "wasm32"))]
use bevy::pbr::wireframe::WireframePlugin;
use bevy::prelude::*;
use bevy_rapier3d::plugin::{NoUserData, RapierPhysicsPlugin};
use bevy_vector_shapes::ShapePlugin;
use camera::PlatonicCamera;
use controller::Controller;
use game_settings::GameSettingsPlugin;
use player::PlayerPlugin;
use shape::cube::{self, maze::CubeMaze};

mod camera;
mod controller;
mod game_settings;
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
        .add_plugins(GameSettingsPlugin::default())
        .add_plugins(RapierPhysicsPlugin::<NoUserData>::default())
        .add_plugins(Controller::default())
        .add_plugins(PlatonicCamera::default())
        .add_plugins(PlayerPlugin::default())
        .add_plugins(ShapePlugin::default())
        .add_systems(Startup, (load_maze, cube::spawn.after(load_maze)))
        .run();
}

fn load_maze(mut commands: Commands) {
    let maze = CubeMaze::build(3, 2.0);
    commands.insert_resource(maze);
}
