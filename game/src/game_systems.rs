use bevy::{prelude::*, text::Update2dText};

use crate::{
    assets::setup_game_assets,
    camera::{camera_dolly, camera_follow_player, camera_setup, update_camera_on_window_resize},
    controller::{idle, solve, view, ControllerState},
    effects::{
        setup_node_arrival_particle, spawn_node_arrival_particles, update_node_arrival_particles,
    },
    game_state::{victory_transition, GameState},
    light::{light_follow_camera, setup_light},
    player::{despawn_player_halo, move_player, player_halo_follow_player, spawn_player_halo},
    shape::loader::{load_level, spawn_level_meshes, spawn_player},
    statistics::{setup_statistics, update_player_path},
    ui::{
        despawn_level_complete_ui, next_level, previous_level, replay_level,
        spawn_level_complete_ui, update_level_complete_ui,
    },
};

#[derive(Default)]
pub struct GameSystemsPlugin;

impl Plugin for GameSystemsPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(OnEnter(GameState::Loading), load_level);
        let setup_systems = (spawn_level_meshes, setup_statistics, spawn_player).into_configs();
        let controller_solve_system = solve
            .run_if(in_state(ControllerState::Solving))
            .run_if(in_state(GameState::Playing));
        let victory_ui_transition = victory_transition.run_if(in_state(GameState::Playing));
        let update_statistics = update_player_path;

        let camera_follow_player_system =
            (camera_follow_player.run_if(in_state(ControllerState::IdlePostSolve)),)
                .run_if(in_state(GameState::Playing));

        app.add_systems(OnExit(ControllerState::Solving), spawn_player_halo);
        app.add_systems(OnEnter(ControllerState::Solving), despawn_player_halo);
        app.add_systems(Update, player_halo_follow_player);

        let update_systems = (
            move_player,
            controller_solve_system,
            victory_ui_transition,
            update_statistics,
            camera_follow_player_system,
            spawn_node_arrival_particles,
        )
            .into_configs();

        let on_victory_systems = spawn_level_complete_ui.into_configs();
        app.add_systems(OnEnter(GameState::Playing), setup_systems);
        app.add_systems(Update, update_systems);
        app.add_systems(OnEnter(GameState::Victory), on_victory_systems);

        app.init_state::<GameState>()
            .add_systems(OnExit(GameState::Victory), despawn_level_complete_ui)
            .add_systems(
                Update,
                (
                    update_level_complete_ui,
                    next_level,
                    replay_level,
                    previous_level,
                )
                    .run_if(in_state(GameState::Victory)),
            );

        let camera_update_systems = (
            idle.run_if(in_state(ControllerState::IdlePostSolve)),
            idle.run_if(in_state(ControllerState::IdlePostView)),
            view.run_if(in_state(ControllerState::Viewing)),
            camera_dolly.run_if(in_state(ControllerState::Viewing)),
        );

        app.add_systems(
            Update,
            camera_update_systems.run_if(in_state(GameState::Playing)),
        )
        .add_systems(Startup, camera_setup)
        .add_systems(Update, update_camera_on_window_resize);

        app.add_systems(Startup, setup_light)
            .add_systems(Update, light_follow_camera);

        app.add_systems(Startup, setup_node_arrival_particle);
        app.add_systems(Update, update_node_arrival_particles);

        app.add_systems(Startup, setup_game_assets);
    }
}
