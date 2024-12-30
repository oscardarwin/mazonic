use bevy::{ecs::schedule::SystemConfigs, prelude::*};

use crate::{
    camera::{camera_dolly, camera_follow_player, camera_setup},
    controller::{idle, solve, view, ControllerState},
    game_state::{victory_transition, GameState},
    player::{move_player, PlayerMazeState},
    shape::{
        cube::Cube,
        dodecahedron::Dodecahedron,
        icosahedron::Icosahedron,
        loader::{load_level, spawn_level_meshes, LevelType, PlatonicLevelData},
        octahedron::Octahedron,
        platonic_solid::PlatonicSolid,
        tetrahedron::Tetrahedron,
    },
    ui::{
        despawn_level_complete_ui, next_level, spawn_level_complete_ui, update_level_complete_ui,
    },
};

use strum::IntoEnumIterator;

struct LevelSystems {
    pub setup_systems: SystemConfigs,
    pub update_systems: SystemConfigs,
}

#[derive(Default)]
pub struct GameSystemsPlugin;

impl GameSystemsPlugin {
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
        let setup_systems = spawn_level_meshes::<P>.into_configs();
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

impl Plugin for GameSystemsPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(OnEnter(GameState::Loading), load_level);

        for level_type in LevelType::iter() {
            let level_systems = self.get_systems_for_level_type(level_type);

            let LevelSystems {
                setup_systems,
                update_systems,
            } = level_systems;

            app.add_systems(
                OnEnter(GameState::Playing),
                setup_systems.run_if(in_state(level_type)),
            );
            app.add_systems(
                Update,
                update_systems
                    .run_if(in_state(level_type))
                    .run_if(in_state(GameState::Playing)),
            );
        }

        app.init_state::<GameState>()
            .add_systems(OnEnter(GameState::Victory), spawn_level_complete_ui)
            .add_systems(OnExit(GameState::Victory), despawn_level_complete_ui)
            .add_systems(
                Update,
                (update_level_complete_ui, next_level).run_if(in_state(GameState::Victory)),
            );

        app.add_systems(
            Update,
            idle.run_if(in_state(ControllerState::IdlePostSolve)),
        )
        .add_systems(Update, idle.run_if(in_state(ControllerState::IdlePostView)))
        .add_systems(Update, view.run_if(in_state(ControllerState::Viewing)));

        app.add_systems(
            Update,
            camera_follow_player.run_if(in_state(ControllerState::IdlePostSolve)),
        )
        .add_systems(
            Update,
            camera_dolly.run_if(in_state(ControllerState::Viewing)),
        )
        .add_systems(Startup, camera_setup);
    }
}
