use bevy::ecs::system::Resource;
use bevy::prelude::*;

pub struct FaceColorPalette {
    pub colors: [Color; 6],
}

impl FaceColorPalette {}

pub struct GameColorPalette {
    pub player_color: Color,
    pub face_colors: FaceColorPalette,
    pub line_color: Color,
    pub background_color: Color,
}

impl Default for GameColorPalette {
    fn default() -> GameColorPalette {
        GameColorPalette {
            player_color: Color::srgba_u8(255, 209, 102, 254),
            line_color: Color::linear_rgba(0.95, 0.91, 0.835, 0.99),
            face_colors: FaceColorPalette {
                colors: [
                    Color::srgba_u8(247, 140, 107, 254), // orange
                    Color::srgba_u8(17, 138, 178, 254),  // blue
                    Color::srgba_u8(6, 214, 160, 254),   // green
                    Color::srgba_u8(239, 71, 111, 254),  // pink
                    Color::srgba_u8(7, 59, 76, 254),     // dark blue
                    Color::srgba_u8(255, 255, 255, 254), // white
                ],
            },
            background_color: Color::srgba_u8(57, 62, 70, 0),
        }
    }
}

#[derive(Resource)]
pub struct GameSettings {
    pub player_elevation: f32,
    pub camera_distance: f32,
    pub light_offset: f32,
    pub camera_follow_speed: f32,
    pub max_player_speed: f32,
    pub palette: GameColorPalette,
}

impl GameSettings {}

impl Default for GameSettings {
    fn default() -> Self {
        Self {
            player_elevation: 0.05,
            camera_distance: 3.0,
            light_offset: 3.0,
            camera_follow_speed: 0.08,
            max_player_speed: 1.5,
            palette: GameColorPalette::default(),
        }
    }
}

#[derive(Default)]
pub struct GameSettingsPlugin;

impl Plugin for GameSettingsPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<GameSettings>();
    }
}
