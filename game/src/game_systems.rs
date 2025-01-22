use bevy::{ecs::schedule::SystemConfigs, prelude::*, text::Update2dText};

use crate::{
    camera::{
        camera_dolly, camera_follow_player, camera_move_to_target, camera_setup,
        trigger_camera_resize_on_level_change, trigger_camera_resize_on_window_change,
        update_camera_distance, CameraResizeState,
    },
    controller::{idle, solve, view, ControllerState},
    effects::{
        setup_node_arrival_particle, spawn_node_arrival_particles, update_node_arrival_particles,
    },
    game_state::{victory_transition, GameState, PlayState},
    level_selector::{self, setup_save_data, SelectorState},
    light::{light_follow_camera, setup_light},
    materials::setup_materials,
    menu,
    player::{
        move_player, spawn_player, spawn_player_halo, turn_off_player_halo, turn_on_player_halo,
        update_halo_follow_player,
    },
    shape::loader::{
        despawn_level_data, load_level_asset, spawn_level_data_components, spawn_level_meshes,
    },
    sound::play_note,
    statistics::{setup_statistics, update_player_path},
    ui,
};

#[derive(Default)]
pub struct GameSystemsPlugin;

impl Plugin for GameSystemsPlugin {
    fn build(&self, app: &mut App) {
        app.init_state::<GameState>()
            .add_sub_state::<PlayState>()
            .add_sub_state::<SelectorState>()
            .add_sub_state::<CameraResizeState>();

        let enter_play_systems = (
            spawn_level_meshes,
            setup_statistics,
            spawn_player,
            trigger_camera_resize_on_level_change.after(spawn_player),
            spawn_player_halo.after(spawn_player),
        )
            .into_configs();

        let exit_play_systems = (ui::despawn_level_complete_ui, despawn_level_data).into_configs();

        let enter_selector_init_systems = (
            level_selector::load,
            level_selector::set_initial_camera_target.after(level_selector::load),
        )
            .into_configs();

        let enter_loading_systems = (
            despawn_level_data,
            load_level_asset.after(despawn_level_data),
        )
            .into_configs();

        let startup_systems = (
            camera_setup,
            setup_light,
            setup_node_arrival_particle,
            setup_materials,
            setup_save_data,
        );

        let update_systems = get_update_systems();

        app.add_systems(Startup, startup_systems)
            .add_systems(Update, update_systems)
            .add_systems(OnEnter(GameState::Setup), menu::setup)
            .add_systems(OnEnter(GameState::Selector), enter_selector_init_systems)
            .add_systems(OnEnter(PlayState::Loading), enter_loading_systems)
            .add_systems(OnEnter(PlayState::Playing), enter_play_systems)
            .add_systems(OnEnter(GameState::Playing), ui::spawn_navigation_ui)
            .add_systems(OnExit(GameState::Playing), exit_play_systems)
            .add_systems(OnEnter(ControllerState::Solving), turn_off_player_halo)
            .add_systems(
                OnEnter(ControllerState::IdlePostSolve),
                camera_follow_player,
            )
            .add_systems(
                OnExit(SelectorState::Clicked),
                level_selector::set_camera_target_to_closest_face,
            )
            .add_systems(OnExit(ControllerState::Solving), turn_on_player_halo);
    }
}

fn get_update_systems() -> SystemConfigs {
    let selector_systems = (
        level_selector::set_selector_state.run_if(in_state(GameState::Selector)),
        level_selector::update_interactables.run_if(in_state(GameState::Selector)),
        level_selector::update_selection_overlay.run_if(in_state(GameState::Selector)),
        camera_move_to_target.run_if(in_state(SelectorState::Idle)),
        camera_dolly.run_if(in_state(SelectorState::Clicked)),
    )
        .into_configs();

    (
        move_player,
        solve.run_if(in_state(ControllerState::Solving)),
        victory_transition.run_if(in_state(PlayState::Playing)),
        update_player_path.run_if(in_state(PlayState::Playing)),
        play_note.run_if(in_state(PlayState::Playing)),
        camera_move_to_target.run_if(in_state(ControllerState::IdlePostSolve)),
        spawn_node_arrival_particles,
        (
            ui::update_level_complete_ui,
            ui::next_level,
            ui::replay_level,
            ui::previous_level,
            ui::level_selector,
        )
            .run_if(in_state(GameState::Playing)),
        idle.run_if(
            in_state(ControllerState::IdlePostSolve).or(in_state(ControllerState::IdlePostView)),
        ),
        view.run_if(in_state(ControllerState::Viewing)),
        camera_dolly.run_if(in_state(ControllerState::Viewing)),
        update_halo_follow_player.run_if(in_state(GameState::Playing)),
        (
            update_camera_distance.run_if(in_state(CameraResizeState::Resizing)),
            trigger_camera_resize_on_window_change.run_if(in_state(CameraResizeState::Fixed)),
        ),
        light_follow_camera,
        update_node_arrival_particles,
        spawn_level_data_components.run_if(in_state(PlayState::Loading)),
        selector_systems,
    )
        .into_configs()
}
