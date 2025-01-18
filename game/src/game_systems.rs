use bevy::{prelude::*, text::Update2dText};

use crate::{
    assets::setup_game_assets,
    camera::{camera_dolly, camera_follow_player, camera_setup, update_camera_on_window_resize},
    controller::{idle, solve, view, ControllerState},
    effects::{
        setup_node_arrival_particle, spawn_node_arrival_particles, update_node_arrival_particles,
    },
    game_state::{victory_transition, GameState, PlayState},
    level_selector::{self, SelectorCameraState},
    light::{light_follow_camera, setup_light},
    player::{
        move_player, spawn_player, spawn_player_halo, turn_off_player_halo, turn_on_player_halo,
        update_halo_follow_player,
    },
    shape::loader::{load_level_asset, spawn_level_data_components, spawn_level_meshes},
    sound::play_note,
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
        app.init_state::<GameState>()
            .add_sub_state::<PlayState>()
            .add_sub_state::<SelectorCameraState>();

        let enter_play_systems = (
            spawn_level_meshes,
            setup_statistics,
            spawn_player,
            spawn_player_halo.after(spawn_player),
        )
            .into_configs();

        let startup_systems = (
            camera_setup,
            setup_light,
            setup_node_arrival_particle,
            setup_game_assets,
        );

        let update_systems = (
            move_player,
            solve.run_if(in_state(ControllerState::Solving)),
            victory_transition.run_if(in_state(PlayState::Playing)),
            update_player_path.run_if(in_state(PlayState::Playing)),
            play_note.run_if(in_state(PlayState::Playing)),
            camera_follow_player
                .run_if(in_state(ControllerState::IdlePostSolve))
                .run_if(in_state(PlayState::Playing)),
            spawn_node_arrival_particles,
            (
                update_level_complete_ui,
                next_level,
                replay_level,
                previous_level,
            )
                .run_if(in_state(PlayState::Victory)),
            idle.run_if(
                in_state(ControllerState::IdlePostSolve)
                    .or(in_state(ControllerState::IdlePostView)),
            ),
            view.run_if(in_state(ControllerState::Viewing)),
            camera_dolly.run_if(in_state(ControllerState::Viewing)),
            level_selector::idle.run_if(in_state(SelectorCameraState::Idle)),
            camera_dolly.run_if(in_state(SelectorCameraState::Dolly)),
            level_selector::view.run_if(in_state(SelectorCameraState::Dolly)),
            update_halo_follow_player.run_if(in_state(GameState::Playing)),
            update_camera_on_window_resize,
            light_follow_camera,
            update_node_arrival_particles,
            spawn_level_data_components,
        )
            .into_configs();

        app.add_systems(Startup, startup_systems)
            .add_systems(Update, update_systems)
            .add_systems(OnEnter(GameState::Selector), level_selector::load)
            .add_systems(OnEnter(PlayState::Loading), load_level_asset)
            .add_systems(OnEnter(PlayState::Playing), enter_play_systems)
            .add_systems(OnEnter(PlayState::Victory), spawn_level_complete_ui)
            .add_systems(OnExit(PlayState::Victory), despawn_level_complete_ui)
            .add_systems(OnEnter(ControllerState::Solving), turn_off_player_halo)
            .add_systems(OnExit(ControllerState::Solving), turn_on_player_halo);
    }
}
