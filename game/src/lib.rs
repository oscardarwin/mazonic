#![allow(warnings)]
use assets::DashedArrowMaterial;
#[cfg(not(target_arch = "wasm32"))]
use bevy::pbr::wireframe::WireframePlugin;
use bevy::{pbr::ExtendedMaterial, prelude::*};
use bevy_rapier3d::prelude::*;
use controller::Controller;
use game_settings::GameSettingsPlugin;
use game_systems::GameSystemsPlugin;
use shape::loader::{GameLevel, LoaderPlugin};

mod assets;
mod camera;
mod constants;
mod controller;
mod effects;
mod game_settings;
mod game_state;
mod game_systems;
mod is_room_junction;
mod light;
mod player;
pub mod room;
pub mod shape;
mod statistics;
mod ui;

pub fn save_level(name: &str, level: GameLevel) {}

pub fn run() {
    App::new()
        .add_plugins((
            DefaultPlugins.set(ImagePlugin::default_nearest()),
            #[cfg(not(target_arch = "wasm32"))]
            WireframePlugin,
        ))
        .add_plugins(RapierPhysicsPlugin::<NoUserData>::default())
        .add_plugins(GameSettingsPlugin::default())
        .add_plugins(LoaderPlugin::default())
        .add_plugins(Controller::default())
        .add_plugins(GameSystemsPlugin::default())
        .add_plugins(MaterialPlugin::<
            ExtendedMaterial<StandardMaterial, DashedArrowMaterial>,
        >::default())
        .run();
}
