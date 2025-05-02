use crate::{
    game_save::{CompletedEasyDailies, CompletedHardDailies, CurrentPuzzle, PuzzleIdentifier, WorkingLevelIndex},
    player::PlayerMazeState,
    shape::loader::SolutionComponent,
    statistics::PlayerPath,
};
use bevy::prelude::*;

#[derive(States, Default, Debug, Clone, PartialEq, Eq, Hash)]
pub enum GameState {
    #[default]
    Setup,
    Selector,
    Menu,
    LoadingRemoteLevel,
    Playing,
}

#[derive(SubStates, Default, Debug, Clone, PartialEq, Eq, Hash)]
#[source(GameState = GameState::Playing)]
pub enum PlayState {
    #[default]
    Loading,
    Playing,
    Victory,
}

pub fn victory_transition(
    mut next_controller_state: ResMut<NextState<PlayState>>,
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
        next_controller_state.set(PlayState::Victory)
    }
}

pub fn update_working_level_on_victory(
    current_puzzle_query: Query<&CurrentPuzzle>,
    mut working_level_index_query: Query<&mut WorkingLevelIndex>,
    mut completed_easy_dailies: Query<&mut CompletedEasyDailies>,
    mut completed_hard_dailies: Query<&mut CompletedHardDailies>,
) {
    let Ok(CurrentPuzzle(puzzle_identifier)) = current_puzzle_query.get_single() else {
        return;
    };

    let Ok(mut working_level_index) = working_level_index_query.get_single_mut() else {
        return;
    };

    let Ok(mut completed_easy_dailies) = completed_easy_dailies.get_single_mut() else {
        return;
    };

    let Ok(mut completed_hard_dailies) = completed_hard_dailies.get_single_mut() else {
        return;
    };

    match puzzle_identifier {
        PuzzleIdentifier::Level(level) if *level == working_level_index.0 => {
            working_level_index.0 = level + 1;
            println!("Updating Working Level to {:?}", working_level_index);
        }
        PuzzleIdentifier::EasyDaily(id) => {
            completed_easy_dailies.0.insert(id.clone());
        }
        PuzzleIdentifier::HardDaily(id) => {
            completed_hard_dailies.0.insert(id.clone());
        }
        _ => {}
    }
}

