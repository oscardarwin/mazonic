use bevy::prelude::*;

#[derive(Component, Clone, Debug)]
pub enum ControllerScreenPosition {
    Position(Vec2),
    None,
}

pub fn setup(mut commands: Commands) {
    commands.spawn(ControllerScreenPosition::None);
}
