use bevy::prelude::*;

use crate::{
    player::PlayerMazeState,
    shape::{loader::PlatonicLevelData, platonic_solid::PlatonicSolid},
    ui::{level_complete, ui_button_system},
};

#[derive(States, Default, Debug, Clone, PartialEq, Eq, Hash)]
pub enum GameState {
    #[default]
    Playing,
    Victory,
}

#[derive(Default)]
pub struct GameStatePlugin;

impl Plugin for GameStatePlugin {
    fn build(&self, app: &mut App) {
        app.init_state::<GameState>()
            .add_systems(OnEnter(GameState::Victory), level_complete)
            .add_systems(
                Update,
                ui_button_system.run_if(in_state(GameState::Victory)),
            );
    }
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
