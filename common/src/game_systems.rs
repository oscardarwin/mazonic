use bevy::{
    ecs::{schedule::SystemConfigs, system::SystemId},
    prelude::*,
    text::Update2dText,
};

use crate::{
    assets::{material_handles::setup_materials, mesh_handles::setup_mesh_handles}, camera, controller::{idle, solve, view, ControllerState}, controller_screen_position, effects::{
        self,
        node_arrival::{spawn_node_arrival_particles, update_node_arrival_particles},
    }, game_save::{setup_save_data, update_save_data}, game_state::{
        update_working_level_on_victory, victory_transition,
        GameState, PlayState,
    }, level_selector::{self, SelectorState}, light::{light_follow_camera, setup_light}, load_level_asset, maze::{self, mesh::update_on_melody_discovered}, menu, player::{
        move_player, spawn_player, turn_off_player_halo, turn_on_player_halo,
        update_halo_follow_player,
    }, shape::{
        self,
        loader::{despawn_level_data, spawn_level_data},
    }, sound::{self, check_melody_solved, play_note}, statistics::update_player_path, ui, victory
};

#[derive(Default)]
pub struct GameSystemsPlugin;

impl Plugin for GameSystemsPlugin {
    fn build(&self, app: &mut App) {
        app.init_state::<GameState>()
            .add_sub_state::<PlayState>()
            .add_sub_state::<SelectorState>()
            .add_sub_state::<victory::VictoryState>();

        app.init_resource::<SystemHandles>();

        let enter_play_systems = (
            shape::loader::spawn_mesh,
            maze::mesh::spawn,
            spawn_player,
            camera::update_distance.after(spawn_player),
            camera::reset_dolly_screen_positions,
            ui::navigation::update_previous_level_button_visibility,
            ui::navigation::update_next_level_button_visibility,
        )
            .into_configs();

        let exit_play_systems = (
            ui::navigation::despawn_level_navigation_ui,
            despawn_level_data,
        )
            .into_configs();

        let enter_solving_systems = (
            turn_off_player_halo,
            effects::player_particles::turn_off_player_particles,
        );
        let exit_solving_systems = (
            turn_on_player_halo,
            effects::player_particles::turn_on_player_particles,
        );

        let enter_victory_systems = (
            camera::follow_player,
            update_working_level_on_victory,
            ui::navigation::update_next_level_button_visibility
                .after(update_working_level_on_victory),
        );

        let enter_selector_init_systems = (
            level_selector::load,
            level_selector::set_initial_camera_target.after(level_selector::load),
        )
            .into_configs();

        let enter_loading_systems = (
            despawn_level_data,
            load_level_asset::load_.after(despawn_level_data),
        )
            .into_configs();

        let startup_systems = (
            camera::setup,
            setup_light,
            setup_materials,
            setup_save_data,
            setup_mesh_handles,
            effects::player_particles::setup,
            effects::musical_notes::setup,
            effects::musical_note_burst::setup,
            controller_screen_position::setup,
            load_level_asset::setup,
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
            .add_systems(OnEnter(victory::VictoryState::Viewing), camera::reset_dolly_screen_positions)
            .add_systems(OnEnter(GameState::Playing), ui::navigation::spawn)
            .add_systems(OnExit(GameState::Playing), exit_play_systems)
            .add_systems(OnEnter(ControllerState::Solving), enter_solving_systems)
            .add_systems(
                OnEnter(ControllerState::IdlePostSolve),
                camera::follow_player,
            )
            .add_systems(
                OnExit(ControllerState::Viewing),
                camera::reset_dolly_screen_positions,
            )
            .add_systems(
                OnExit(SelectorState::Clicked),
                level_selector::set_camera_target_to_closest_face,
            )
            .add_systems(OnExit(ControllerState::Solving), exit_solving_systems);
    }
}

fn get_update_systems() -> SystemConfigs {
    let selector_systems = (
        level_selector::set_selector_state.run_if(in_state(GameState::Selector)),
        level_selector::update_interactables.run_if(in_state(GameState::Selector)),
        level_selector::update_selection_overlay.run_if(in_state(GameState::Selector))
    ).into_configs();

    let camera_systems = (
        camera::camera_dolly.run_if(
            in_state(ControllerState::Viewing)
                .or(in_state(victory::VictoryState::Viewing).or(in_state(SelectorState::Clicked))),
        ),
        camera::trigger_camera_resize_on_window_change,
        camera::camera_rotate_to_target.run_if(
            in_state(ControllerState::IdlePostSolve)
            .or(in_state(SelectorState::Idle))),
        camera::camera_zoom_to_target.run_if(
            in_state(ControllerState::IdlePostSolve)
            .or(in_state(ControllerState::IdlePostView))
            .or(in_state(SelectorState::Idle))
            .or(in_state(victory::VictoryState::Idle)),
        ),
        camera::update_dolly.run_if(
            in_state(ControllerState::Viewing)
                .or(in_state(ControllerState::IdlePostView))
                .or(in_state(PlayState::Victory))
                .or(in_state(GameState::Selector))),

        
    )
        .into_configs();

    (
        (
            move_player,
            update_save_data,
            update_halo_follow_player,
            effects::player_particles::update_player_particles,
        )
            .run_if(in_state(GameState::Playing)),
        (
            ui::navigation::update_level_complete_ui,
            ui::navigation::next_level,
            ui::navigation::replay_level,
            ui::navigation::previous_level,
            ui::navigation::level_selector,
            effects::musical_note_burst::clear_up_effects,
        )
            .run_if(in_state(GameState::Playing)),
        victory_transition.run_if(in_state(PlayState::Playing)),
        update_player_path.run_if(in_state(PlayState::Playing)),
        play_note.run_if(in_state(PlayState::Playing)),
        check_melody_solved.run_if(in_state(PlayState::Playing)),
        shape::loader::spawn_level_data.run_if(in_state(PlayState::Loading)),
        solve.run_if(in_state(ControllerState::Solving)),
        spawn_node_arrival_particles,
        idle.run_if(
            in_state(ControllerState::IdlePostSolve).or(in_state(ControllerState::IdlePostView)),
        ),
        view.run_if(in_state(ControllerState::Viewing)),
        victory::update_state.run_if(in_state(PlayState::Victory)),
        light_follow_camera,
        update_node_arrival_particles,
        effects::musical_notes::spawn_notes,
        selector_systems,
        camera_systems,
    )
        .into_configs()
}

#[derive(Resource)]
pub struct SystemHandles {
    pub spawn_maze: SystemId,
    pub note_burst: SystemId,
    pub update_on_melody_discovered: SystemId,
    pub play_melody: SystemId,
    pub resize_camera_distance: SystemId,
}

impl FromWorld for SystemHandles {
    fn from_world(world: &mut World) -> Self {
        let spawn_maze = world.register_system(maze::mesh::spawn);
        let note_burst = world.register_system(effects::musical_note_burst::spawn);
        let update_on_melody_discovered = world.register_system(update_on_melody_discovered);
        let play_melody = world.register_system(sound::play_melody);
        let resize_camera_distance = world.register_system(camera::update_distance);

        SystemHandles {
            spawn_maze,
            note_burst,
            update_on_melody_discovered,
            play_melody,
            resize_camera_distance,
        }
    }
}
