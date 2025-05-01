use bevy::prelude::*;

#[derive(Component, Clone, Debug, Default, Copy)]
pub enum ControllerScreenPosition {
    Position(Vec2),
    #[default]
    None,
}

pub fn setup(mut commands: Commands) {
    commands.spawn(ControllerScreenPosition::None);
}
