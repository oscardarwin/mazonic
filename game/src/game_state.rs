use crate::{player::PlayerMazeState, shape::loader::SolutionComponent};
use bevy::prelude::*;

#[derive(States, Default, Debug, Clone, PartialEq, Eq, Hash)]
pub enum GameState {
    #[default]
    Loading,
    Playing,
    Victory,
}

pub fn victory_transition(
    mut next_controller_state: ResMut<NextState<GameState>>,
    player_state_query: Query<&PlayerMazeState>,
    maze_component: Query<&SolutionComponent>,
) {
    let Ok(SolutionComponent(solution)) = maze_component.get_single() else {
        return;
    };

    let player_state = player_state_query.single();

    let final_room = solution.last().unwrap();

    match player_state {
        PlayerMazeState::Node(room) if room == final_room => {
            next_controller_state.set(GameState::Victory)
        }
        _ => {}
    }
}