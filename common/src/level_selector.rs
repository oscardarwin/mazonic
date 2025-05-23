use bevy::{
    input::{mouse::MouseButtonInput, ButtonState},
    pbr::ExtendedMaterial,
    prelude::*,
    utils::{hashbrown::HashSet, HashMap},
    window::PrimaryWindow, winit::CreateWindowParams,
};
use bevy_hanabi::{EffectMaterial, ParticleEffectBundle};
use bevy_rapier3d::prelude::*;
use chrono::Utc;

use crate::{
    assets::{
        material_handles::MaterialHandles,
        mesh_generators::{FaceMeshGenerator, TriangleFaceMeshGenerator},
        mesh_handles::MeshHandles,
        shaders::{MenuSelectionHoverShader, PulsingShader},
    }, camera::{CameraTarget, MainCamera}, constants::{SQRT_3, SYMBOL_TEXTURE_DIMENSIONS}, controller_screen_position::ControllerScreenPosition, effects::musical_notes::{MusicalNoteEffectColor, MusicalNoteEffectHandle, MusicalNoteImageHandles, MusicalNoteMarker}, game_save::{
        CurrentPuzzle, LevelIndex, PuzzleIdentifier, WorkingLevelIndex
    }, game_settings::GameSettings, game_state::GameState, levels::{Shape, LEVELS}, maze::{maze_mesh_builder::MazeMeshBuilder, mesh::get_cross_face_edge_transform}, play_statistics::PlayStatistics, shape::{icosahedron, shape_utils::compute_face_normal}, sound::Melody
};

const FACE_ORDER: [usize; 20] = [
    0, 2, 1, 4, 3, 11, 12, 5, 6, 7, 8, 19, 17, 16, 15, 14, 13, 10, 9, 18,
];

const EASY_DAILY_POSITION: usize = 7;
const HARD_DAILY_POSITION: usize = 15;

#[derive(Debug, Clone)]
pub enum SelectorOption {
    Level(LevelIndex),
    EasyDaily,
    HardDaily,
}

impl SelectorOption {
    fn daily_level_filename() -> String {
        let date = Utc::now();
        date.format("%Y-%m-%d").to_string()
    }
}

impl Into<PuzzleIdentifier> for SelectorOption {
    fn into(self) -> PuzzleIdentifier {
        match self {
            SelectorOption::Level(level_index) => PuzzleIdentifier::Level(level_index),
            SelectorOption::EasyDaily => PuzzleIdentifier::EasyDaily(Self::daily_level_filename()),
            SelectorOption::HardDaily => PuzzleIdentifier::HardDaily(Self::daily_level_filename()),
        }
    }
}

const SELECTOR_OPTIONS: [SelectorOption; 20] = [
    SelectorOption::Level(0),
    SelectorOption::Level(1),
    SelectorOption::Level(2),
    SelectorOption::Level(3),
    SelectorOption::Level(4),
    SelectorOption::Level(5),
    SelectorOption::Level(6),
    SelectorOption::EasyDaily,
    SelectorOption::Level(7),
    SelectorOption::Level(8),
    SelectorOption::Level(9),
    SelectorOption::Level(10),
    SelectorOption::Level(11),
    SelectorOption::Level(12),
    SelectorOption::Level(13),
    SelectorOption::Level(14),
    SelectorOption::HardDaily,
    SelectorOption::Level(15),
    SelectorOption::Level(16),
    SelectorOption::Level(17),
];

#[derive(SubStates, Hash, Eq, Clone, PartialEq, Debug, Default)]
#[source(GameState = GameState::Selector)]
pub enum SelectorState {
    Clicked,
    #[default]
    Idle,
}

#[derive(Component, Clone, Debug)]
pub struct SelectorEntity;

#[derive(Component, Clone, Debug)]
pub struct SelectableLevel(pub SelectorOption);

#[derive(Component, Clone, Debug)]
pub struct SelectedLevel(pub Option<usize>);

#[derive(Component, Clone, Debug, PartialEq)]
pub enum SelectorOverlayState {
    Hovered,
    Pressed,
    None,
}

#[derive(Component, Clone, Debug)]
pub struct CameraTargetTransform(Transform);

#[derive(Component, Clone, Debug)]
pub struct SelectionOverlay;

pub fn load(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    play_statistics: Res<PlayStatistics>,
    material_handles: Res<MaterialHandles>,
    mesh_handles: Res<MeshHandles>,
) {
    let working_level_index = play_statistics.get_working_level();
    let completed_puzzles = play_statistics.0
        .iter()
        .filter(|(_, puzzle_statistics)| puzzle_statistics.completed)
        .map(|(puzzle_identifier, _)| puzzle_identifier)
        .cloned()
        .collect::<HashSet<PuzzleIdentifier>>();

    let selector_material_handles = &material_handles.selector;
    let faces = icosahedron::faces();

    let tetrahedron_symbol_mesh_handle = meshes.add(coordinate_to_symbol_mesh(4, 1));
    let cube_symbol_mesh_handle = meshes.add(coordinate_to_symbol_mesh(3, 1));
    let octahedron_symbol_mesh_handle = meshes.add(coordinate_to_symbol_mesh(2, 1));
    let dodecahedron_symbol_mesh_handle = meshes.add(coordinate_to_symbol_mesh(1, 1));
    let icosahedron_symbol_mesh_handle = meshes.add(coordinate_to_symbol_mesh(0, 1));

    let number_mesh_handles = (1..10)
        .map(|number| {
            let mesh = number_symbol_mesh(number);
            let mesh_handle = meshes.add(mesh);
            (number as u8, mesh_handle)
        })
        .collect::<HashMap<u8, Handle<Mesh>>>();

    let daily_symbol_mesh_handle = meshes.add(coordinate_to_symbol_mesh(0, 4));

    let face_local_transforms = (0..SELECTOR_OPTIONS.len())
        .map(|level_index| compute_face_transform(level_index, &faces))
        .collect::<Vec<Transform>>();

    for (selector_option_index, selector_option) in SELECTOR_OPTIONS.iter().enumerate() {
        let puzzle_identifier = selector_option.clone().into(); 

        let face_material_handle = match puzzle_identifier {
            _ if completed_puzzles.contains(&puzzle_identifier) => selector_material_handles.completed.clone(),
            PuzzleIdentifier::Level(level_index) if level_index == working_level_index => selector_material_handles.incomplete_face_colors[level_index].clone(),
            PuzzleIdentifier::EasyDaily(_) if working_level_index >= EASY_DAILY_POSITION => selector_material_handles.incomplete_face_colors[EASY_DAILY_POSITION].clone(),
            PuzzleIdentifier::HardDaily(_) if working_level_index >= HARD_DAILY_POSITION => selector_material_handles.incomplete_face_colors[HARD_DAILY_POSITION].clone(),
            _ => selector_material_handles.unavailable.clone(),
        };

        let face_index = FACE_ORDER[selector_option_index];
        let face_mesh_handle = mesh_handles.shape_mesh_handles.icosahedron[face_index].clone();

        let transform = face_local_transforms[selector_option_index];
        

        let symbol_mesh_handle = match selector_option {
            SelectorOption::Level(level_index) => match LEVELS[*level_index].shape {
                Shape::Tetrahedron(_) => tetrahedron_symbol_mesh_handle.clone(),
                Shape::Cube(_) => cube_symbol_mesh_handle.clone(),
                Shape::Octahedron(_) => octahedron_symbol_mesh_handle.clone(),
                Shape::Dodecahedron(_) => dodecahedron_symbol_mesh_handle.clone(),
                Shape::Icosahedron(_) => icosahedron_symbol_mesh_handle.clone(),
            },
            SelectorOption::EasyDaily => daily_symbol_mesh_handle.clone(),
            SelectorOption::HardDaily => daily_symbol_mesh_handle.clone(),
        };



        let face_vertices = faces[face_index];
        let triangle_collider =
            Collider::triangle(face_vertices[0], face_vertices[1], face_vertices[2]);

        let face_object = (
            Mesh3d(face_mesh_handle.clone()),
            MeshMaterial3d(face_material_handle),
        );

        let selection_overlay_object = (
            Mesh3d(face_mesh_handle.clone()),
            MeshMaterial3d(selector_material_handles.selection_hover.clone()),
        );
        
        let is_melody_discovered = play_statistics.0
            .get(&puzzle_identifier)
            .map_or(false, |puzzle_statistics| puzzle_statistics.discovered_melody.is_some());

        commands
            .spawn(triangle_collider)
            .insert(face_object)
            .insert(SelectorEntity)
            .insert(SelectorOverlayState::None)
            .insert(SelectableLevel(selector_option.clone()))
            .insert(CameraTargetTransform(transform.clone()))
            .insert(Visibility::default())
            .with_children(|parent| {
                parent.spawn(transform).with_children(|parent| {
                    let mut symbol_entity_commands = parent.spawn(Mesh3d(symbol_mesh_handle));
                    
                    match selector_option {
                        SelectorOption::Level(level_index) => {
                            if is_melody_discovered {
                                symbol_entity_commands.insert(MeshMaterial3d(
                                    selector_material_handles.melody_found_selector_face.clone(),
                                ));
                            } else if *level_index > working_level_index {
                                symbol_entity_commands.insert(MeshMaterial3d(
                                    selector_material_handles.unavailable_level_symbols.clone(),
                                ));
                            } else {
                                symbol_entity_commands.insert(MeshMaterial3d(
                                    selector_material_handles.level_symbols.clone(),
                                ));
                            };

                            let number_mesh_handle =
                                number_mesh_handles.get(&LEVELS[*level_index].nodes_per_edge).unwrap();
                            let mut number_entity_commands =
                                parent.spawn(Mesh3d(number_mesh_handle.clone()));

                            if is_melody_discovered {
                                number_entity_commands.insert(MeshMaterial3d(
                                    selector_material_handles.melody_found_selector_face.clone(),
                                ));
                            } else if *level_index > working_level_index {
                                number_entity_commands.insert(MeshMaterial3d(
                                    selector_material_handles.unavailable_level_symbols.clone(),
                                ));
                            } else {
                                number_entity_commands.insert(MeshMaterial3d(
                                    selector_material_handles.level_symbols.clone(),
                                ));
                            };
                        }
                        SelectorOption::EasyDaily | SelectorOption::HardDaily if is_melody_discovered => {
                            symbol_entity_commands.insert(MeshMaterial3d(
                                    selector_material_handles.melody_found_selector_face.clone(),
                            ));
                        }
                        SelectorOption::EasyDaily if working_level_index >= EASY_DAILY_POSITION => {
                            symbol_entity_commands.insert(MeshMaterial3d(
                                selector_material_handles.level_symbols.clone(),
                            ));
                        }
                        SelectorOption::HardDaily if working_level_index >= HARD_DAILY_POSITION => {
                            symbol_entity_commands.insert(MeshMaterial3d(
                                selector_material_handles.level_symbols.clone(),
                            ));
                        }
                        _ => {
                            symbol_entity_commands.insert(MeshMaterial3d(
                                selector_material_handles.unavailable_level_symbols.clone(),
                            ));
                        }
                    }
                });

                if is_melody_discovered {
                    let face_center = face_vertices.iter().sum::<Vec3>() / 3.0;
                    let spawner_transform = Transform::IDENTITY
                        .looking_at(-face_center, face_center.any_orthogonal_vector())
                        .with_translation(face_center * 1.05);

                    parent.spawn((spawner_transform, MusicalNoteMarker(selector_option_index, MusicalNoteEffectColor::Player)));
                }
                parent
                    .spawn(Transform::from_translation(transform.translation * 0.00001))
                    .insert(selection_overlay_object)
                    .insert(SelectionOverlay)
                    .insert(Visibility::Hidden);
            });
    }

    let mesh_builder = MazeMeshBuilder::level_selector();
    let edge_mesh_handle = meshes.add(mesh_builder.one_way_cross_face_edge());

    let total_path_size = working_level_index + if working_level_index >= HARD_DAILY_POSITION { 
        2
    } else if working_level_index >= EASY_DAILY_POSITION { 
        1
    } else { 
        0
    };

    println!("total path size: {}", total_path_size);

    for (from_level_index, to_level_index) in
        (0..).zip(1..SELECTOR_OPTIONS.len()).take(total_path_size)
    {
        let from_transform = face_local_transforms[from_level_index];
        let to_transform = face_local_transforms[to_level_index];

        let edge_transform = get_cross_face_edge_transform(
            from_transform.translation,
            -*from_transform.forward(),
            to_transform.translation,
            -*to_transform.forward(),
        );

        commands
            .spawn(Mesh3d(edge_mesh_handle.clone()))
            .insert(MeshMaterial3d(
                material_handles.bright_dashed_arrow_handle.clone(),
            ))
            .insert(edge_transform)
            .insert(SelectorEntity);
    }

    commands.spawn(SelectedLevel(None)).insert(SelectorEntity);
}

pub fn despawn(
    mut commands: Commands,
    selector_entities: Query<Entity, With<SelectorEntity>>,
) {
    for entity in selector_entities.iter() {
        commands.entity(entity).despawn_recursive();
    }
}

fn compute_face_transform(level_index: usize, faces: &[[Vec3; 3]; 20]) -> Transform {
    let face_index = FACE_ORDER[level_index];

    let face = faces[face_index];
    let face_normal = compute_face_normal(&face);
    let face_center = face.iter().fold(Vec3::ZERO, |acc, item| acc + item) / 3.0;

    let other_level_index = if level_index == 0 { 1 } else { level_index - 1 };
    let other_face_index = FACE_ORDER[other_level_index];
    let other_face = faces[other_face_index];

    let face_vertex_indices = icosahedron::FACE_INDICES[face_index]
        .into_iter()
        .collect::<HashSet<usize>>();

    let other_face_vertex_indices = icosahedron::FACE_INDICES[other_face_index]
        .into_iter()
        .collect::<HashSet<usize>>();

    let edge_vertex_indices = face_vertex_indices
        .intersection(&other_face_vertex_indices)
        .cloned()
        .collect::<Vec<usize>>();

    let icosahedron_vertices = icosahedron::vertices();

    let edge_midpoint = edge_vertex_indices
        .iter()
        .fold(Vec3::ZERO, |acc, item| acc + icosahedron_vertices[*item])
        / 2.0
        / 2.0;

    let center_to_edge = if level_index == 0 {
        face_center - edge_midpoint
    } else {
        edge_midpoint - face_center
    };

    Transform::IDENTITY
        .with_scale(Vec3::splat(0.4))
        .looking_at(-face_normal.clone(), center_to_edge.cross(face_normal))
        .with_translation(face_center + face_normal * 0.003)
}

fn number_symbol_mesh(number: u8) -> Mesh {
    let y_coord = 2 + (number - 1) / 5;
    let x_coord = (number - 1) % 5;
    coordinate_to_symbol_mesh(x_coord, y_coord)
}

pub fn coordinate_to_symbol_mesh(x_coord: u8, y_coord: u8) -> Mesh {
    let coordinate = UVec2::new(x_coord.into(), y_coord.into());
    let max_uv = (coordinate.as_vec2() + Vec2::ONE) / SYMBOL_TEXTURE_DIMENSIONS;
    let min_uv = coordinate.as_vec2() / SYMBOL_TEXTURE_DIMENSIONS;

    let uvs = vec![
        min_uv.to_array(),
        [min_uv.x, max_uv.y],
        max_uv.to_array(),
        [max_uv.x, min_uv.y],
    ];

    let symbol_mesh = Mesh::from(Rectangle::new(1.0, 1.0));
    symbol_mesh.with_inserted_attribute(Mesh::ATTRIBUTE_UV_0, uvs)
}

pub fn set_selector_state(
    controller_screen_position_query: Query<
        &ControllerScreenPosition,
        Changed<ControllerScreenPosition>,
    >,
    mut mouse_button_event_reader: EventReader<MouseButtonInput>,
    mut next_selector_state: ResMut<NextState<SelectorState>>,
    mut previous_cursor_positions: Local<(ControllerScreenPosition, ControllerScreenPosition)>,
) {
    let Ok(controller_screen_position) = controller_screen_position_query.get_single() else {
        return;
    };

    let (penultimate_position, last_position) = *previous_cursor_positions;

    match (penultimate_position, last_position, controller_screen_position) {
        (ControllerScreenPosition::None, ControllerScreenPosition::None, ControllerScreenPosition::Position(_)) => {
            next_selector_state.set(SelectorState::Clicked);
        }

        (_, _, ControllerScreenPosition::None) => {
            next_selector_state.set(SelectorState::Idle);
        }
        _ => {}
    }

    *previous_cursor_positions = (last_position, controller_screen_position.clone());
}

pub fn update_interactables(
    rapier_context_query: Query<&RapierContext>,
    camera_query: Query<(&GlobalTransform, &Camera), With<MainCamera>>,
    primary_window: Query<&Window, With<PrimaryWindow>>,
    mut overlay_states_query: Query<(Entity, &mut SelectorOverlayState, &SelectableLevel)>,
    mut game_state: ResMut<NextState<GameState>>,
    mut selector_state: Res<State<SelectorState>>,
    mut current_level_index_query: Query<&mut CurrentPuzzle>,
    completed_level_index_query: Query<&WorkingLevelIndex>,
    controller_screen_position_query: Query<&ControllerScreenPosition>,
    mut start_touch_entity: Local<Option<Entity>>,
    mut previous_controller_screen_position: Local<ControllerScreenPosition>,
) {
    let Ok(controller_screen_position) = controller_screen_position_query.get_single() else {
        return;
    };

    let Ok(window) = primary_window.get_single() else {
        return;
    };

    let Ok(WorkingLevelIndex(working_level_index)) = completed_level_index_query.get_single() else {
        return;
    };

    let (camera_global_transform, camera) = camera_query.single();
    let window_center_position = window.size() / 2.0;

    let Some(ray) = camera
        .viewport_to_world(camera_global_transform, window_center_position)
        .ok()
    else {
        return;
    };

    let Some((window_center_entity, _)) = rapier_context_query
        .single()
        .cast_ray(
            ray.origin,
            ray.direction.into(),
            30.,
            true,
            QueryFilter::default(),
        ) else {
        return;
    };


    let touch_intersected_entity = match *controller_screen_position {
        ControllerScreenPosition::Position(position) => {
            camera.viewport_to_world(camera_global_transform, position)
                .ok()
                .map(|ray| rapier_context_query
                    .single()
                    .cast_ray(
                        ray.origin,
                        ray.direction.into(),
                        30.,
                        true,
                        QueryFilter::default(),
                    )
                    .map(|(entity, _)| entity)
                )
                .flatten()
        },
        ControllerScreenPosition::None => None,
    };

    let touch_matches_window_center_entity = match touch_intersected_entity {
        Some(touch_entity) => window_center_entity == touch_entity,
        None => false,
    };

    *start_touch_entity = match (*previous_controller_screen_position, controller_screen_position) {
        (ControllerScreenPosition::None, _) if touch_matches_window_center_entity => touch_intersected_entity,
        (_, ControllerScreenPosition::None) => None,
        _ => *start_touch_entity,
    };

    let selected_face_pressed = touch_intersected_entity == *start_touch_entity && touch_matches_window_center_entity;

    for (entity, mut overlay_state, SelectableLevel(selector_puzzle)) in overlay_states_query.iter_mut()
    {

        let level_playable = match selector_puzzle { 
            SelectorOption::Level(level_index) => level_index <= working_level_index,
            SelectorOption::EasyDaily => *working_level_index >= EASY_DAILY_POSITION,
            SelectorOption::HardDaily => *working_level_index >= HARD_DAILY_POSITION,
        };

        let interacted_and_matches_touch = *overlay_state != SelectorOverlayState::None 
            && selected_face_pressed;

        let new_overlay_state = if window_center_entity != entity {
            SelectorOverlayState::None
        }
        else if window_center_entity == entity && selected_face_pressed && level_playable {
            SelectorOverlayState::Pressed
        } else if window_center_entity == entity && level_playable {
            SelectorOverlayState::Hovered
        } else {
            SelectorOverlayState::None 
        };

        if *overlay_state == SelectorOverlayState::Pressed
            && new_overlay_state == SelectorOverlayState::Hovered
            && start_touch_entity.is_none()
        {

            *current_level_index_query.single_mut() = CurrentPuzzle(selector_puzzle.clone().into());
            let next_game_state = match selector_puzzle {
                SelectorOption::Level(level_index) => GameState::Puzzle,
                SelectorOption::EasyDaily | SelectorOption::HardDaily => GameState::LoadingRemoteLevel,
            };
            game_state.set(next_game_state);
            break;
        }

        if *overlay_state != new_overlay_state {
            *overlay_state = new_overlay_state;
        }
    }
    
    *previous_controller_screen_position = *controller_screen_position;
}

pub fn update_selection_overlay(
    changed_overlay_state_query: Query<
        (&SelectorOverlayState, &Children),
        Changed<SelectorOverlayState>,
    >,
    game_material_handles: Res<MaterialHandles>,
    mut overlay_query: Query<
        (
            &mut MeshMaterial3d<ExtendedMaterial<StandardMaterial, MenuSelectionHoverShader>>,
            &mut Visibility,
        ),
        With<SelectionOverlay>,
    >,
) {
    for (overlay_state, children) in changed_overlay_state_query.iter() {
        let child = children
            .into_iter()
            .filter(|child| overlay_query.get(**child).is_ok())
            .next()
            .unwrap();

        let (mut material, mut visibility) = overlay_query.get_mut(*child).unwrap();

        match overlay_state {
            SelectorOverlayState::None => {
                *visibility = Visibility::Hidden;
            }
            SelectorOverlayState::Hovered => {
                material.0 = game_material_handles.selector.selection_hover.clone();
                *visibility = Visibility::Visible;
            }
            SelectorOverlayState::Pressed => {
                material.0 = game_material_handles.selector.selection_pressed.clone();
                *visibility = Visibility::Visible;
            }
        }
    }
}

pub fn set_initial_camera_target(
    selectable: Query<(&CameraTargetTransform, &SelectableLevel)>,
    mut camera_target_query: Query<&mut CameraTarget>,
    current_level_index_query: Query<&CurrentPuzzle>,
    game_settings: Res<GameSettings>,
) {
    let mut camera_target = camera_target_query.single_mut();

    let CurrentPuzzle(puzzle_identifier) = current_level_index_query.single();

    println!(
        "Setting selector look at level index: {:?}",
        puzzle_identifier
    );

    let face_transform = selectable
        .iter()
        .filter(|(_, SelectableLevel(selector_level))| {
            match (selector_level, puzzle_identifier) {
                (SelectorOption::Level(selector_index), PuzzleIdentifier::Level(level_index)) => selector_index == level_index,
                (SelectorOption::EasyDaily, PuzzleIdentifier::EasyDaily(_)) => true,
                (SelectorOption::HardDaily, PuzzleIdentifier::HardDaily(_)) => true,
                _ => false,
            }
            
        })
        .map(|(CameraTargetTransform(transform), _)| transform)
        .next()
        .unwrap();

    camera_target.translation_dir = *-face_transform.forward();
    camera_target.translation_norm = game_settings.camera_distance;
    camera_target.up = *face_transform.right();
}

pub fn set_camera_target_to_closest_face(
    mut camera_target_query: Query<(&mut CameraTarget, &Transform)>,
    selectable: Query<&CameraTargetTransform, With<SelectableLevel>>,
    game_settings: Res<GameSettings>,
) {
    let (mut camera_target, camera_transform) = camera_target_query.single_mut();

    let camera_forward = camera_transform.forward();

    let Some(CameraTargetTransform(closest_face_transform)) =
        selectable
            .iter()
            .min_by_key(|CameraTargetTransform(selectable_transform)| {
                let face_normal = -Vec3::from(selectable_transform.forward());
                (camera_forward.dot(face_normal) * 100.0) as i32
            })
    else {
        return;
    };

    println!("Setting selector camera target to closest face");

    camera_target.translation_dir = -closest_face_transform.forward().normalize();
    camera_target.translation_norm = game_settings.camera_distance;
    camera_target.up = *closest_face_transform.right();
}
