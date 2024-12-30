#[cfg(not(target_arch = "wasm32"))]
use bevy::pbr::wireframe::WireframePlugin;
use bevy::prelude::*;
use bevy_rapier3d::plugin::{NoUserData, RapierPhysicsPlugin};
use bevy_vector_shapes::ShapePlugin;
use controller::Controller;
use game_settings::GameSettingsPlugin;
use game_systems::GameSystemsPlugin;
use maze_generator::config::Maze;
use shape::loader::LoaderPlugin;
use ui::UiPlugin;

mod camera;
mod controller;
mod game_settings;
mod game_state;
mod game_systems;
mod player;
mod shape;
mod ui;

fn main() {
    App::new()
        .add_plugins((
            DefaultPlugins.set(ImagePlugin::default_nearest()),
            #[cfg(not(target_arch = "wasm32"))]
            WireframePlugin,
        ))
        .add_plugins(GameSettingsPlugin::default())
        .add_plugins(LoaderPlugin::default())
        .add_plugins(RapierPhysicsPlugin::<NoUserData>::default())
        .add_plugins(Controller::default())
        .add_plugins(UiPlugin::default())
        .add_plugins(GameSystemsPlugin::default())
        .run();
}
