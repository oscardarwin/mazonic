use bevy::{
    ecs::{schedule::SystemConfigs, system::SystemId},
    prelude::*,
    text::Update2dText,
};

use crate::{
    assets::{material_handles::setup_materials, mesh_handles::setup_mesh_handles}, camera, controller::{self, idle, solve, view, ControllerState}, controller_screen_position, effects::{
        self,
        node_arrival::{spawn_node_arrival_particles, update_node_arrival_particles},
    }, game_save, game_state::{
        victory_transition,
        GameState, PuzzleState,
    }, level_selector::{self, SelectorState}, levels, light, load_level_asset, maze::{self, mesh::update_on_melody_discovered}, menu, play_statistics, player, player_path, shape, sound::{self, check_melody_solved, play_note}, ui, victory
};

#[derive(Default)]
pub struct GameSystemsPlugin;

impl Plugin for GameSystemsPlugin {
    fn build(&self, app: &mut App) {
        app.init_state::<GameState>()
            .add_sub_state::<PuzzleState>()
            .add_sub_state::<SelectorState>()
            .add_sub_state::<victory::VictoryState>();

        app.init_resource::<SystemHandles>();

        let enter_play_systems = (
            shape::spawn,
            maze::mesh::spawn,
            player::spawn,
            camera::update_distance.after(player::spawn),
            play_statistics::on_play,
            camera::reset_dolly_screen_positions,
            ui::navigation::update_previous_level_button_visibility,
            ui::navigation::update_next_level_button_visibility,
            ui::navigation::update_selector_and_replay_button_visibility,
        )
            .into_configs();

        let exit_puzzle_systems = (
            ui::navigation::despawn_level_navigation_ui,
            levels::despawn_puzzle_entities,
            ui::message::exit_puzzle_state,
        )
            .into_configs();

        let enter_solving_systems = (
            player::turn_off_player_halo,
            effects::player_particles::turn_off_player_particles,
        );
        let exit_solving_systems = (
            player::turn_on_player_halo,
            effects::player_particles::turn_on_player_particles,
        );

        let enter_victory_systems = (
            camera::follow_player,
            play_statistics::on_victory,
            ui::navigation::update_next_level_button_visibility
                .after(play_statistics::on_victory),
        );

        let enter_selector_init_systems = (
            level_selector::load,
            camera::reset_dolly_screen_positions,
            level_selector::set_initial_camera_target.after(level_selector::load),
        )
            .into_configs();

        let enter_loading_systems = (
            levels::despawn_puzzle_entities,
        )
            .into_configs();

        let startup_systems = (
            camera::setup,
            light::setup,
            setup_materials,
            game_save::setup,
            setup_mesh_handles,
            effects::player_particles::setup,
            effects::musical_notes::setup,
            effects::musical_note_burst::setup,
            controller_screen_position::setup,
            load_level_asset::setup,
            ui::message::spawn,
            menu::setup.after(game_save::setup),
            play_statistics::setup,
        );

        let update_systems = get_update_systems();

        app.add_systems(Startup, startup_systems)
            .add_systems(Update, update_systems)
            .add_systems(OnEnter(GameState::Selector), enter_selector_init_systems)
            .add_systems(
                OnExit(PuzzleState::Loading),
                level_selector::despawn,
            )
            .add_systems(OnEnter(PuzzleState::Loading), enter_loading_systems)
            .add_systems(OnEnter(PuzzleState::Playing), enter_play_systems)
            .add_systems(OnExit(PuzzleState::Playing), play_statistics::exit_play)
            .add_systems(OnEnter(PuzzleState::Victory), enter_victory_systems)
            .add_systems(OnEnter(victory::VictoryState::Viewing), camera::reset_dolly_screen_positions)
            .add_systems(OnEnter(GameState::Puzzle), ui::navigation::spawn)
            .add_systems(OnExit(GameState::Puzzle), exit_puzzle_systems)
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
                .or(in_state(PuzzleState::Victory))
                .or(in_state(GameState::Selector))),

        
    )
        .into_configs();

    (
        (
            player::update,
            game_save::update,
            player::update_halo,
            effects::player_particles::update_player_particles,
        )
            .run_if(in_state(GameState::Puzzle)),
        (
            ui::navigation::update_level_complete_ui,
            ui::navigation::next_level,
            ui::navigation::replay_level,
            ui::navigation::previous_level,
            ui::navigation::level_selector,
            effects::musical_note_burst::clear_up_effects,
            ui::message::update_lower_during_puzzle_state,
        )
            .run_if(in_state(GameState::Puzzle)),
        victory_transition.run_if(in_state(PuzzleState::Playing)),
        player_path::update.run_if(in_state(PuzzleState::Playing)),
        sound::play_note.run_if(in_state(PuzzleState::Playing)),
        sound::check_melody_solved.run_if(in_state(PuzzleState::Playing)),
        load_level_asset::spawn_level_data.run_if(in_state(PuzzleState::Loading)),
        (
            effects::node_arrival::update_node_arrival_particles,
            effects::node_arrival::spawn_node_arrival_particles,
        ),
        (
            controller::solve.run_if(in_state(ControllerState::Solving)),
            controller::idle.run_if(
                in_state(ControllerState::IdlePostSolve).or(in_state(ControllerState::IdlePostView)),
            ),
            controller::view.run_if(in_state(ControllerState::Viewing)),
        ),
        victory::update_state.run_if(in_state(PuzzleState::Victory)),
        light::follow_camera,
        play_statistics::during_play.run_if(in_state(PuzzleState::Playing)),
        effects::musical_notes::spawn,
        selector_systems,
        camera_systems,
        ui::message::update_upper,
        ui::message::on_change,
        game_save::update_working_level,
        load_level_asset::wait_until_loaded.run_if(in_state(GameState::LoadingRemoteLevel))
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
