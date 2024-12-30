use bevy::{ecs::schedule::SystemConfigs, prelude::*};

use crate::{
    controller::{solve, ControllerState},
    player::{move_player, PlayerMazeState},
    shape::{
        cube::Cube,
        dodecahedron::Dodecahedron,
        icosahedron::Icosahedron,
        loader::{load_level, setup_player, spawn_shape_meshes, LevelType, PlatonicLevelData},
        octahedron::Octahedron,
        platonic_solid::PlatonicSolid,
        tetrahedron::Tetrahedron,
    },
    ui::{level_complete, ui_button_system},
};

use strum::IntoEnumIterator;

#[derive(States, Default, Debug, Clone, PartialEq, Eq, Hash)]
pub enum GameState {
    #[default]
    Playing,
    Victory,
}

struct LevelSystems {
    pub setup_systems: SystemConfigs,
    pub update_systems: SystemConfigs,
}

#[derive(Default)]
pub struct GameStatePlugin;

impl GameStatePlugin {
    fn get_systems_for_level_type(&self, level_type: LevelType) -> LevelSystems {
        match level_type {
            LevelType::Cube => self.get_systems_for_solid_type::<Cube>(),
            LevelType::Tetrahedron => self.get_systems_for_solid_type::<Tetrahedron>(),
            LevelType::Icosahedron => self.get_systems_for_solid_type::<Icosahedron>(),
            LevelType::Octahedron => self.get_systems_for_solid_type::<Octahedron>(),
            LevelType::Dodecahedron => self.get_systems_for_solid_type::<Dodecahedron>(),
        }
    }

    fn get_systems_for_solid_type<P: PlatonicSolid>(&self) -> LevelSystems {
        let setup_systems = (spawn_shape_meshes::<P>, setup_player::<P>).into_configs();
        let controller_solve_system = solve::<P>.run_if(in_state(ControllerState::Solving));
        let victory_ui_system = victory_transition::<P>.run_if(in_state(GameState::Playing));
        let update_systems =
            (move_player::<P>, controller_solve_system, victory_ui_system).into_configs();

        LevelSystems {
            setup_systems,
            update_systems,
        }
    }
}

impl Plugin for GameStatePlugin {
    fn build(&self, app: &mut App) {
        for level_type in LevelType::iter() {
            let level_systems = self.get_systems_for_level_type(level_type);

            let LevelSystems {
                setup_systems,
                update_systems,
            } = level_systems;

            app.add_systems(
                OnEnter(level_type),
                (load_level, setup_systems.after(load_level)),
            );
            app.add_systems(Update, update_systems.run_if(in_state(level_type)));
        }

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
