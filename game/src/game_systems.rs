use bevy::{ecs::schedule::SystemConfigs, prelude::*, text::Update2dText};

use crate::{
    assets::{material_handles::setup_materials, mesh_handles::setup_mesh_handles},
    camera::{
        camera_dolly, camera_follow_player, camera_move_to_target, camera_setup,
        trigger_camera_resize_on_level_change, trigger_camera_resize_on_window_change,
        update_camera_distance, CameraResizeState,
    },
    controller::{idle, solve, view, ControllerState},
    effects::{
        setup_node_arrival_particle, spawn_node_arrival_particles, update_node_arrival_particles,
    },
    game_save::{setup_save_data, update_save_data},
    game_state::{
        update_perfect_score_on_victory, update_working_level_on_victory, victory_transition,
        GameState, PlayState,
    },
    level_selector::{self, SelectorState},
    light::{light_follow_camera, setup_light},
    maze, menu,
    player::{
        move_player, spawn_player, turn_off_player_halo, turn_on_player_halo,
        update_halo_follow_player,
    },
    shape::{
        self,
        loader::{despawn_level_data, load_level_asset, spawn_level_data},
    },
    sound::{check_melody_solved, play_note},
    statistics::update_player_path,
    ui::{self, update_next_level_button_visibility, update_previous_level_button_visibility},
    victory::{self},
};

#[derive(Default)]
pub struct GameSystemsPlugin;

impl Plugin for GameSystemsPlugin {
    fn build(&self, app: &mut App) {
        app.init_state::<GameState>()
            .add_sub_state::<PlayState>()
            .add_sub_state::<SelectorState>()
            .add_sub_state::<CameraResizeState>()
            .add_sub_state::<victory::VictoryState>();

        let enter_play_systems = (
            shape::loader::spawn_mesh,
            maze::mesh::spawn,
            spawn_player,
            trigger_camera_resize_on_level_change.after(spawn_player),
            update_previous_level_button_visibility,
            update_next_level_button_visibility,
        )
            .into_configs();

        let exit_play_systems = (ui::despawn_level_complete_ui, despawn_level_data).into_configs();

        let enter_victory_systems = (
            update_working_level_on_victory,
            update_next_level_button_visibility.after(update_working_level_on_victory),
            update_perfect_score_on_victory,
        );

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
            setup_mesh_handles,
        );

        let update_systems = get_update_systems();

        app.add_systems(Startup, startup_systems)
            .add_systems(Update, update_systems)
            .add_systems(OnEnter(GameState::Setup), menu::setup)
            .add_systems(OnEnter(GameState::Selector), enter_selector_init_systems)
            .add_systems(
                OnExit(GameState::Selector),
                level_selector::despawn_selector_entities,
            )
            .add_systems(OnEnter(PlayState::Loading), enter_loading_systems)
            .add_systems(OnEnter(PlayState::Playing), enter_play_systems)
            .add_systems(OnEnter(PlayState::Victory), enter_victory_systems)
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
        (move_player, update_save_data, update_halo_follow_player)
            .run_if(in_state(GameState::Playing)),
        (
            ui::update_level_complete_ui,
            ui::next_level,
            ui::replay_level,
            ui::previous_level,
            ui::level_selector,
        )
            .run_if(in_state(GameState::Playing)),
        victory_transition.run_if(in_state(PlayState::Playing)),
        update_player_path.run_if(in_state(PlayState::Playing)),
        play_note.run_if(in_state(PlayState::Playing)),
        check_melody_solved.run_if(in_state(PlayState::Playing)),
        shape::loader::spawn_level_data.run_if(in_state(PlayState::Loading)),
        camera_move_to_target.run_if(in_state(ControllerState::IdlePostSolve)),
        solve.run_if(in_state(ControllerState::Solving)),
        spawn_node_arrival_particles,
        idle.run_if(
            in_state(ControllerState::IdlePostSolve).or(in_state(ControllerState::IdlePostView)),
        ),
        view.run_if(in_state(ControllerState::Viewing)),
        camera_dolly.run_if(in_state(ControllerState::Viewing).or(in_state(VictoryState::Viewing))),
        victory::update_state,
        (
            update_camera_distance.run_if(in_state(CameraResizeState::Resizing)),
            trigger_camera_resize_on_window_change.run_if(in_state(CameraResizeState::Fixed)),
        ),
        light_follow_camera,
        update_node_arrival_particles,
        selector_systems,
    )
        .into_configs()
}
