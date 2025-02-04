use bevy::{
    input::{mouse::MouseButtonInput, ButtonState},
    pbr::ExtendedMaterial,
    prelude::*,
    utils::{hashbrown::HashSet, HashMap},
    window::PrimaryWindow,
};
use bevy_hanabi::{EffectMaterial, ParticleEffectBundle};
use bevy_rapier3d::prelude::*;

use crate::{
    assets::{
        material_handles::MaterialHandles,
        mesh_generators::{FaceMeshGenerator, TriangleFaceMeshGenerator},
        shaders::{MenuSelectionHoverShader, PulsingShader},
    },
    camera::{CameraTarget, MainCamera},
    constants::SQRT_3,
    effects::musical_notes::{MusicalNoteEffectHandle, MusicalNoteImageHandles, MusicalNoteMarker},
    game_save::{
        CurrentLevelIndex, DiscoveredMelodies, PerfectScoreLevelIndices, WorkingLevelIndex,
    },
    game_settings::GameSettings,
    game_state::GameState,
    levels::{Shape, LEVELS},
    maze::{maze_mesh_builder::MazeMeshBuilder, mesh::get_cross_face_edge_transform},
    shape::{icosahedron, shape_utils::compute_face_normal},
    sound::Melody,
};

const SYMBOL_TEXTURE_DIMENSIONS: Vec2 = Vec2::new(5.0, 3.0);

const FACE_ORDER: [usize; 20] = [
    0, 2, 1, 4, 3, 11, 12, 5, 6, 7, 8, 19, 17, 16, 15, 14, 13, 10, 9, 18,
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
pub struct SelectableLevel(pub usize);

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
    mut asset_server: ResMut<AssetServer>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    mut pulsing_materials: ResMut<Assets<ExtendedMaterial<StandardMaterial, PulsingShader>>>,
    game_save_query: Query<(
        &WorkingLevelIndex,
        &PerfectScoreLevelIndices,
        &DiscoveredMelodies,
    )>,
    game_settings: Res<GameSettings>,
    game_materials: Res<MaterialHandles>,
    mut mouse_button_event_reader: EventReader<MouseButtonInput>,
) {
    mouse_button_event_reader.clear();

    let (
        WorkingLevelIndex(completed_level_index),
        PerfectScoreLevelIndices(perfect_score_level_indices),
        DiscoveredMelodies(discovered_melodies),
    ) = game_save_query.single();

    let material_handles = &game_materials.selector;
    let ready_easy_color = &game_settings.palette.face_colors.colors[0];
    let ready_hard_color = &game_settings.palette.face_colors.colors[3];
    let faces = icosahedron::faces();
    let face_meshes = TriangleFaceMeshGenerator::get_face_meshes::<20>(faces);

    let line_color_vec = game_settings.palette.line_color.to_linear().to_vec3();
    let level_symbol_sprite_sheet = asset_server.load("sprites/symbols_sprite_sheet.png");
    let sprite_sheet_material_handle = MeshMaterial3d(materials.add(StandardMaterial {
        base_color_texture: Some(level_symbol_sprite_sheet.clone()),
        base_color: game_settings.palette.line_color,
        alpha_mode: AlphaMode::Blend,
        emissive: LinearRgba::from_vec3(line_color_vec * 30.0),
        ..Default::default()
    }));

    let player_color_vec = game_settings.palette.player_color.to_linear().to_vec3();
    let sprite_sheet_melody_found_material_handle =
        MeshMaterial3d(pulsing_materials.add(ExtendedMaterial {
            base: StandardMaterial {
                base_color: game_settings.palette.player_color,
                base_color_texture: Some(level_symbol_sprite_sheet.clone()),
                emissive: LinearRgba::from_vec3(player_color_vec * 2.0),
                alpha_mode: AlphaMode::Blend,
                ..Default::default()
            },
            extension: PulsingShader {},
        }));

    let tetrahedron_symbol_mesh_handle = meshes.add(coordinate_to_symbol_mesh(4, 0));
    let cube_symbol_mesh_handle = meshes.add(coordinate_to_symbol_mesh(3, 0));
    let octahedron_symbol_mesh_handle = meshes.add(coordinate_to_symbol_mesh(2, 0));
    let dodecahedron_symbol_mesh_handle = meshes.add(coordinate_to_symbol_mesh(1, 0));
    let icosahedron_symbol_mesh_handle = meshes.add(coordinate_to_symbol_mesh(0, 0));

    let number_mesh_handles = (1..10)
        .map(|number| {
            let mesh = number_symbol_mesh(number);
            let mesh_handle = meshes.add(mesh);
            (number as u8, mesh_handle)
        })
        .collect::<HashMap<u8, Handle<Mesh>>>();

    let face_local_transforms = (0..LEVELS.len())
        .map(|level_index| compute_face_transform(level_index, &faces))
        .collect::<Vec<Transform>>();

    for (level_index, level) in LEVELS.iter().enumerate() {
        let face_material_handle = if level_index > *completed_level_index {
            material_handles.unavailable.clone()
        } else if level_index == *completed_level_index {
            let mix_factor = (level_index as f32) / (LEVELS.len() as f32);
            let color = ready_easy_color.mix(ready_hard_color, mix_factor);
            materials.add(StandardMaterial::from_color(color))
        } else if perfect_score_level_indices.contains(&level_index) {
            material_handles.perfect_score.clone()
        } else {
            material_handles.completed.clone()
        };

        let face_index = FACE_ORDER[level_index];
        let face_mesh = face_meshes[face_index].clone();
        let face_mesh_handle = meshes.add(face_mesh);

        let transform = face_local_transforms[level_index];
        let symbol_mesh_handle = match level.shape {
            Shape::Tetrahedron => tetrahedron_symbol_mesh_handle.clone(),
            Shape::Cube => cube_symbol_mesh_handle.clone(),
            Shape::Octahedron => octahedron_symbol_mesh_handle.clone(),
            Shape::Dodecahedron => dodecahedron_symbol_mesh_handle.clone(),
            Shape::Icosahedron => icosahedron_symbol_mesh_handle.clone(),
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
            MeshMaterial3d(game_materials.selector.selection_hover.clone()),
        );

        let is_melody_discovered = discovered_melodies.contains_key(&level_index);
        commands
            .spawn(triangle_collider)
            .insert(face_object)
            .insert(SelectorEntity)
            .insert(SelectorOverlayState::None)
            .insert(SelectableLevel(level_index))
            .insert(CameraTargetTransform(transform.clone()))
            .with_children(|parent| {
                parent.spawn(transform).with_children(|parent| {
                    let mut symbol_entity_commands = parent.spawn(Mesh3d(symbol_mesh_handle));
                    if is_melody_discovered {
                        symbol_entity_commands
                            .insert(sprite_sheet_melody_found_material_handle.clone());
                    } else {
                        symbol_entity_commands.insert(sprite_sheet_material_handle.clone());
                    };

                    let number_mesh_handle =
                        number_mesh_handles.get(&level.nodes_per_edge).unwrap();
                    let mut number_entity_commands =
                        parent.spawn(Mesh3d(number_mesh_handle.clone()));

                    if is_melody_discovered {
                        number_entity_commands
                            .insert(sprite_sheet_melody_found_material_handle.clone());
                    } else {
                        number_entity_commands.insert(sprite_sheet_material_handle.clone());
                    };
                });

                if is_melody_discovered {
                    let face_center = face_vertices.iter().sum::<Vec3>() / 3.0;
                    let spawner_transform = Transform::IDENTITY
                        .looking_at(-face_center, face_center.any_orthogonal_vector())
                        .with_translation(face_center * 1.05);

                    parent.spawn((spawner_transform, MusicalNoteMarker));
                }
                parent
                    .spawn(Transform::from_translation(transform.translation * 0.00001))
                    .insert(selection_overlay_object)
                    .insert(SelectionOverlay)
                    .insert(Visibility::Hidden);
            });
    }

    let mesh_builder = MazeMeshBuilder::icosahedron(1.0 / SQRT_3 / 3.0);
    let edge_mesh_handle = meshes.add(mesh_builder.one_way_cross_face_edge());

    for (from_level_index, to_level_index) in (0..).zip(1..LEVELS.len()) {
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
                game_materials.bright_dashed_arrow_handle.clone(),
            ))
            .insert(edge_transform)
            .insert(SelectorEntity);
    }

    commands.spawn(SelectedLevel(None)).insert(SelectorEntity);
}

pub fn despawn_selector_entities(
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
    let y_coord = 1 + (number - 1) / 5;
    let x_coord = (number - 1) % 5;
    coordinate_to_symbol_mesh(x_coord, y_coord)
}

fn coordinate_to_symbol_mesh(x_coord: u8, y_coord: u8) -> Mesh {
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

fn color_palette(game_settings: &GameSettings) -> Vec<StandardMaterial> {
    let face_colors = game_settings.palette.face_colors.colors;

    let color_key_frames = [
        face_colors[1],
        face_colors[2],
        game_settings.palette.player_color,
        face_colors[0],
        face_colors[3],
    ];

    color_key_frames
        .iter()
        .zip(color_key_frames[1..].iter())
        .flat_map(|(from_hsla, to_hsla)| {
            [
                from_hsla.mix(to_hsla, 0.0),
                from_hsla.mix(to_hsla, 0.2),
                from_hsla.mix(to_hsla, 0.4),
                from_hsla.mix(to_hsla, 0.6),
                from_hsla.mix(to_hsla, 0.8),
            ]
        })
        .map(|hsla| StandardMaterial::from_color(hsla))
        .collect()
}

pub fn set_selector_state(
    mut mouse_button_event_reader: EventReader<MouseButtonInput>,
    mut next_selector_state: ResMut<NextState<SelectorState>>,
) {
    let Some(mouse_button_event) = mouse_button_event_reader
        .read()
        .filter(|input| input.button == MouseButton::Left)
        .map(|input| input.state)
        .next()
    else {
        return;
    };

    match mouse_button_event {
        ButtonState::Pressed => {
            next_selector_state.set(SelectorState::Clicked);
        }
        ButtonState::Released => {
            next_selector_state.set(SelectorState::Idle);
        }
    }
}

pub fn update_interactables(
    buttons: Res<ButtonInput<MouseButton>>,
    rapier_context_query: Query<&RapierContext>,
    camera_query: Query<(&GlobalTransform, &Camera), With<MainCamera>>,
    primary_window: Query<&Window, With<PrimaryWindow>>,
    mut overlay_states_query: Query<(Entity, &mut SelectorOverlayState, &SelectableLevel)>,
    mut next_game_state: ResMut<NextState<GameState>>,
    mut selector_state: Res<State<SelectorState>>,
    mut current_level_index_query: Query<&mut CurrentLevelIndex>,
    completed_level_index_query: Query<&WorkingLevelIndex>,
) {
    let Ok(window) = primary_window.get_single() else {
        return;
    };

    let Some(cursor_position) = window.cursor_position() else {
        return;
    };

    let (camera_global_transform, camera) = camera_query.single();

    let Some(ray) = camera
        .viewport_to_world(camera_global_transform, cursor_position)
        .ok()
    else {
        return;
    };

    let intersection = rapier_context_query
        .single()
        .cast_ray(
            ray.origin,
            ray.direction.into(),
            30.,
            true,
            QueryFilter::default(),
        )
        .map(|(entity, _)| entity);

    let pressed = buttons.pressed(MouseButton::Left);

    for (entity, mut overlay_state, SelectableLevel(level_index)) in overlay_states_query.iter_mut()
    {
        let WorkingLevelIndex(completed_level_index) = completed_level_index_query.single();

        let level_playable = level_index <= completed_level_index;

        let new_overlay_state = match (intersection, pressed) {
            (Some(intersected_entity), _) if intersected_entity != entity => {
                SelectorOverlayState::None
            }
            (Some(intersected_entity), true)
                if intersected_entity == entity && *overlay_state != SelectorOverlayState::None =>
            {
                SelectorOverlayState::Pressed
            }
            (Some(intersected_entity), false) if intersected_entity == entity && level_playable => {
                SelectorOverlayState::Hovered
            }
            _ => SelectorOverlayState::None,
        };

        if *overlay_state == SelectorOverlayState::Pressed
            && new_overlay_state == SelectorOverlayState::Hovered
        {
            *current_level_index_query.single_mut() = CurrentLevelIndex(*level_index);
            next_game_state.set(GameState::Playing);
        }

        if *overlay_state != new_overlay_state {
            *overlay_state = new_overlay_state;
        }
    }
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
    current_level_index_query: Query<&CurrentLevelIndex>,
    game_settings: Res<GameSettings>,
) {
    let mut camera_target = camera_target_query.single_mut();

    let CurrentLevelIndex(current_level_index) = current_level_index_query.single();

    println!(
        "Setting selector look at level index: {:?}",
        current_level_index
    );

    let face_transform = selectable
        .iter()
        .filter(|(_, SelectableLevel(level_index))| level_index == current_level_index)
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

    println!("Setting selector camera target: {closest_face_transform:?}");

    camera_target.translation_dir = -closest_face_transform.forward().normalize();
    camera_target.translation_norm = game_settings.camera_distance;
    camera_target.up = *closest_face_transform.right();
}
