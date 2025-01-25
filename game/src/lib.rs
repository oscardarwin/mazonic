#![allow(warnings)]
use std::io::Cursor;

use assets::shaders::{
    DashedArrowMaterial, MenuSelectionHoverMaterial, PlayerHaloMaterial, ShapeFaceMaterial,
};
#[cfg(not(target_arch = "wasm32"))]
use bevy::pbr::wireframe::WireframePlugin;
use bevy::{pbr::ExtendedMaterial, prelude::*};
use bevy_common_assets::json::JsonAssetPlugin;
use bevy_rapier3d::prelude::*;
use bevy_rustysynth::RustySynthPlugin;
use controller::Controller;
use game_settings::GameSettingsPlugin;
use game_systems::GameSystemsPlugin;
use noisy_bevy::NoisyShaderPlugin;
use shape::loader::MazeLevelData;

mod assets;
mod camera;
pub mod constants;
mod controller;
mod effects;
mod game_settings;
mod game_state;
mod game_systems;
pub mod is_room_junction;
mod level_selector;
pub mod levels;
mod light;
pub mod maze;
mod menu;
mod player;
pub mod room;
pub mod shape;
pub mod sound;
mod statistics;
mod ui;

pub fn run() {
    App::new()
        .add_plugins((
            DefaultPlugins.set(ImagePlugin::default_nearest()),
            #[cfg(not(target_arch = "wasm32"))]
            WireframePlugin,
        ))
        .add_plugins(JsonAssetPlugin::<MazeLevelData>::new(&[".json"]))
        .add_plugins(RapierPhysicsPlugin::<NoUserData>::default())
        .add_plugins(GameSettingsPlugin::default())
        .add_plugins(Controller::default())
        .add_plugins(GameSystemsPlugin::default())
        .add_plugins(MaterialPlugin::<
            ExtendedMaterial<StandardMaterial, DashedArrowMaterial>,
        >::default())
        .add_plugins(MaterialPlugin::<
            ExtendedMaterial<StandardMaterial, PlayerHaloMaterial>,
        >::default())
        .add_plugins(MaterialPlugin::<
            ExtendedMaterial<StandardMaterial, ShapeFaceMaterial>,
        >::default())
        .add_plugins(MaterialPlugin::<
            ExtendedMaterial<StandardMaterial, MenuSelectionHoverMaterial>,
        >::default())
        .add_plugins(NoisyShaderPlugin)
        .add_plugins(RustySynthPlugin {
            soundfont: Cursor::new(include_bytes!("../../app/assets/marimba_chiapaneca.sf2")),
        })
        .run();
}
