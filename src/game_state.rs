use crate::{
    player::PlayerMazeState,
    shape::{loader::PlatonicLevelData, platonic_solid::PlatonicSolid},
};
use bevy::prelude::*;

#[derive(States, Default, Debug, Clone, PartialEq, Eq, Hash)]
pub enum GameState {
    #[default]
    Playing,
    Victory,
}

pub fn victory_transition<P: PlatonicSolid>(
    mut next_controller_state: ResMut<NextState<GameState>>,
    player_state_query: Query<&PlayerMazeState<P>>,
    level: Res<PlatonicLevelData<P>>,
) {
    let player_state = player_state_query.single();

    let final_room = level.maze.solution.last().unwrap();

    match player_state {
        PlayerMazeState::Node(room) if room == final_room => {
            next_controller_state.set(GameState::Victory)
        }
        _ => {}
    }
}
