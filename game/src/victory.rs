use bevy::{
    input::{mouse::MouseButtonInput, ButtonState},
    prelude::*,
};

use crate::game_state::PlayState;

#[derive(SubStates, Default, Debug, Clone, PartialEq, Eq, Hash)]
#[source(PlayState = PlayState::Victory)]
pub enum VictoryState {
    #[default]
    Idle,
    Viewing,
}

pub fn update_state(
    mut next_controller_state: ResMut<NextState<VictoryState>>,
    mut mouse_button_event_reader: EventReader<MouseButtonInput>,
) {
    match mouse_button_event_reader
        .read()
        .filter(|input| input.button == MouseButton::Left)
        .map(|input| input.state)
        .next()
    {
        Some(ButtonState::Pressed) => {
            next_controller_state.set(VictoryState::Viewing);
        }
        Some(ButtonState::Released) => {
            next_controller_state.set(VictoryState::Idle);
        }
        None => {}
    }
}
