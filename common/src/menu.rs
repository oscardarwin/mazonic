use bevy::prelude::*;

use crate::{game_save::WorkingLevelIndex, game_state::GameState};

pub fn setup(mut next_game_state: ResMut<NextState<GameState>>, working_level_index_query: Query<&WorkingLevelIndex>) {
    let WorkingLevelIndex(index) = working_level_index_query.single();

    if *index > 0 {
        next_game_state.set(GameState::Selector)
    } else {
        next_game_state.set(GameState::Playing)
    }

}
