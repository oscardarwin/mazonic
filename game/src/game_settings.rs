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
            player_color: Color::srgb_u8(255, 209, 102),
            line_color: Color::linear_rgba(0.95, 0.91, 0.835, 1.0),
            face_colors: FaceColorPalette {
                colors: [
                    Color::srgb_u8(247, 140, 107), // orange
                    Color::srgb_u8(17, 138, 178),  // blue
                    Color::srgb_u8(6, 214, 160),   // green
                    Color::srgb_u8(239, 71, 111),  // pink
                    Color::srgb_u8(7, 59, 76),     // dark blue
                    Color::srgb_u8(255, 255, 255), // white
                ],
            },
            background_color: Color::srgb_u8(57, 62, 70),
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
            max_player_speed: 0.8,
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
