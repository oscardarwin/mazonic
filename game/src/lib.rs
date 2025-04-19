#![allow(warnings)]
use std::io::Cursor;

use assets::shaders::{
    DashedArrowShader, GlobalShader, MenuSelectionHoverShader, PlayerHaloShader, ShadersPlugin,
};
#[cfg(not(target_arch = "wasm32"))]
use bevy::pbr::wireframe::WireframePlugin;
use bevy::{pbr::ExtendedMaterial, prelude::*};
use bevy_common_assets::json::JsonAssetPlugin;
use bevy_hanabi::HanabiPlugin;
use bevy_pkv::PkvStore;
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
pub mod controller_screen_position;
mod effects;
pub mod game_save;
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
mod selector;
pub mod shape;
pub mod sound;
mod statistics;
mod ui;
mod victory;

pub fn add_common_plugins(app: &mut App) {
    app.add_plugins((
        DefaultPlugins,
        #[cfg(not(target_arch = "wasm32"))]
        WireframePlugin,
        JsonAssetPlugin::<MazeLevelData>::new(&[".json"]),
        RapierPhysicsPlugin::<NoUserData>::default(),
        GameSettingsPlugin::default(),
        Controller::default(),
        GameSystemsPlugin::default(),
        NoisyShaderPlugin,
        ShadersPlugin::default(),
        RustySynthPlugin {
            soundfont: Cursor::new(include_bytes!(
                "../../desktop/assets/marimba_chiapaneca.sf2"
            )),
        },
        HanabiPlugin,
    ));
}
