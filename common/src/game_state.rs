use crate::{
    game_save::{CurrentPuzzle, PuzzleIdentifier, WorkingLevelIndex},
    player::PlayerMazeState,
    shape::loader::SolutionComponent,
    player_path::PlayerPath,
};
use bevy::prelude::*;

#[derive(States, Default, Debug, Clone, PartialEq, Eq, Hash)]
pub enum GameState {
    #[default]
    Setup,
    Selector,
    Menu,
    LoadingRemoteLevel,
    Puzzle,
}

#[derive(SubStates, Default, Debug, Clone, PartialEq, Eq, Hash)]
#[source(GameState = GameState::Puzzle)]
pub enum PuzzleState {
    #[default]
    Loading,
    Playing,
    Victory,
}

pub fn victory_transition(
    mut next_controller_state: ResMut<NextState<PuzzleState>>,
    player_state_query: Query<&PlayerMazeState>,
    maze_component: Query<&SolutionComponent>,
) {
    let Ok(SolutionComponent(solution)) = maze_component.get_single() else {
        return;
    };

    let Ok(PlayerMazeState::Node(room)) = player_state_query.get_single() else {
        return;
    };

    let final_room = solution.last().unwrap();

    if room == final_room {
        next_controller_state.set(PuzzleState::Victory)
    }
}

