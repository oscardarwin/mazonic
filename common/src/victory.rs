use bevy::{
    input::{mouse::MouseButtonInput, ButtonState},
    prelude::*,
};

use crate::{
    controller_screen_position::ControllerScreenPosition,
    game_state::PuzzleState,
    shape::loader::{GraphComponent, SolutionComponent},
};

#[derive(SubStates, Default, Debug, Clone, PartialEq, Eq, Hash)]
#[source(PuzzleState = PuzzleState::Victory)]
pub enum VictoryState {
    #[default]
    Idle,
    Viewing,
}

// TODO: Make this independent of mouse events.
pub fn update_state(
    controller_screen_position_query: Query<&ControllerScreenPosition>,
    mut next_controller_state: ResMut<NextState<VictoryState>>,
) {
    match controller_screen_position_query.get_single() {
        Ok(ControllerScreenPosition::Position(_)) => {
            next_controller_state.set(VictoryState::Viewing);
        }
        _ => {
            next_controller_state.set(VictoryState::Idle);
        }
    }
}
