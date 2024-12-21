use bevy::ecs::system::{Commands, Resource};
use bevy::prelude::*;

#[derive(Resource)]
pub struct GameSettings {
    pub player_elevation: f32,
}

impl Default for GameSettings {
    fn default() -> Self {
        Self {
            player_elevation: 0.2,
        }
    }
}

#[derive(Default)]
pub struct GameSettingsPlugin;

impl Plugin for GameSettingsPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Startup, setup_settings);
    }
}

fn setup_settings(mut commands: Commands) {
    commands.insert_resource(GameSettings::default());
}
