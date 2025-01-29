use crate::{
    game_save::{CurrentLevelIndex, PerfectScoreLevelIndices, WorkingLevelIndex},
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
    current_level_index_query: Query<&CurrentLevelIndex>,
    mut working_level_index_query: Query<&mut WorkingLevelIndex>,
) {
    let Ok(CurrentLevelIndex(current_level_index)) = current_level_index_query.get_single() else {
        return;
    };

    let Ok(mut working_level_index) = working_level_index_query.get_single_mut() else {
        return;
    };

    if *current_level_index == working_level_index.0 {
        working_level_index.0 = current_level_index + 1;
        println!("Updating Working Level to {:?}", working_level_index);
    }
}

pub fn update_perfect_score_on_victory(
    current_level_index_query: Query<&CurrentLevelIndex>,
    mut perfect_score_level_indices_query: Query<&mut PerfectScoreLevelIndices>,
    player_path_query: Query<&PlayerPath>,
    solution: Query<&SolutionComponent>,
) {
    let Ok(CurrentLevelIndex(current_level_index)) = current_level_index_query.get_single() else {
        return;
    };

    let Ok(mut perfect_score_level_indices) = perfect_score_level_indices_query.get_single_mut()
    else {
        return;
    };

    let Ok(PlayerPath(path)) = player_path_query.get_single() else {
        return;
    };

    let Ok(SolutionComponent(solution)) = solution.get_single() else {
        return;
    };

    if solution.len() == path.len() {
        perfect_score_level_indices.0.insert(*current_level_index);
    }
}
