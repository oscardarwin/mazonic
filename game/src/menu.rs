use bevy::prelude::*;

use crate::game_state::GameState;

pub fn setup(mut next_game_state: ResMut<NextState<GameState>>) {
    println!("loading menu");
    next_game_state.set(GameState::Playing)
}
