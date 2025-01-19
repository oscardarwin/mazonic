use bevy::{
    prelude::*,
    utils::{hashbrown::HashSet, HashMap},
    window::PrimaryWindow,
};
use bevy_rapier3d::prelude::*;

use crate::{
    camera::MainCamera,
    constants::SQRT_3,
    game_settings::GameSettings,
    game_state::GameState,
    levels::LEVELS,
    materials::GameMaterialHandles,
    shape::{
        icosahedron::Icosahedron,
        loader::{get_cross_face_edge_transform, Shape},
        platonic_mesh_builder::MazeMeshBuilder,
        shape_loader::ShapeMeshLoader,
    },
};

const SYMBOL_TEXTURE_DIMENSIONS: Vec2 = Vec2::new(5.0, 3.0);

const FACE_ORDER: [usize; 20] = [
    0, 2, 1, 4, 3, 11, 12, 5, 6, 7, 8, 19, 17, 16, 15, 14, 13, 10, 9, 18,
];

#[derive(Reflect, Component, Debug, Clone)]
pub struct SelectorSaveData {
    pub completed_index: usize,
    pub melody_found_indices: HashSet<usize>,
    pub perfect_score_indices: HashSet<usize>,
}

impl Default for SelectorSaveData {
    fn default() -> Self {
        SelectorSaveData {
            completed_index: 18,
            melody_found_indices: HashSet::new(),
            perfect_score_indices: HashSet::new(),
        }
    }
}

pub fn setup_save_data(mut commands: Commands) {
    let save_data = SelectorSaveData::default();
    commands.spawn(save_data);
}

#[derive(Component, Debug, Clone)]
pub struct TargetCameraFace(Transform);

#[derive(SubStates, Hash, Eq, Clone, PartialEq, Debug, Default)]
#[source(GameState = GameState::Selector)]
pub enum SelectorCameraState {
    Dolly,
    #[default]
    Idle,
}

#[derive(Component, Clone, Debug)]
pub struct SelectableLevel(pub usize);

pub fn load(
    mut commands: Commands,
    mut asset_server: ResMut<AssetServer>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    save_data_query: Query<&SelectorSaveData>,
    game_settings: Res<GameSettings>,
    game_materials: Res<GameMaterialHandles>,
) {
    println!("loading selector");

    let save_data = save_data_query.single();

    let material_handles = &game_materials.selector_material_handles;
    let ready_easy_color = &game_settings.palette.face_colors.colors[0];
    let ready_hard_color = &game_settings.palette.face_colors.colors[3];
    let icosahedron_shape = Icosahedron::new(1);
    let face_meshes = icosahedron_shape.get_face_meshes();

    let level_symbol_sprite_sheet = asset_server.load("sprites/symbols_sprite_sheet.png");
    let sprite_sheet_material_handle = materials.add(StandardMaterial {
        base_color_texture: Some(level_symbol_sprite_sheet.clone()),
        base_color: game_settings.palette.line_color,
        alpha_mode: AlphaMode::Blend,
        ..Default::default()
    });

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
        .map(|level_index| compute_face_transform(level_index))
        .collect::<Vec<Transform>>();

    for (level_index, level) in LEVELS.iter().enumerate() {
        let face_material_handle = if level_index > save_data.completed_index {
            material_handles.unavailable.clone()
        } else if level_index == save_data.completed_index {
            let mix_factor = (level_index as f32) / (LEVELS.len() as f32);
            let color = ready_easy_color.mix(ready_hard_color, mix_factor);
            materials.add(StandardMaterial::from_color(color))
        } else if save_data.perfect_score_indices.contains(&level_index) {
            material_handles.perfect_score.clone()
        } else {
            material_handles.completed.clone()
        };

        let face_index = FACE_ORDER[level_index];
        let face_mesh = face_meshes[face_index].clone();
        let face_mesh_handle = meshes.add(face_mesh);
        commands
            .spawn(Mesh3d(face_mesh_handle))
            .insert(MeshMaterial3d(face_material_handle.clone()));

        let transform = face_local_transforms[level_index];
        let symbol_mesh_handle = match level.shape {
            Shape::Tetrahedron(_) => tetrahedron_symbol_mesh_handle.clone(),
            Shape::Cube(_) => cube_symbol_mesh_handle.clone(),
            Shape::Octahedron(_) => octahedron_symbol_mesh_handle.clone(),
            Shape::Dodecahedron(_) => dodecahedron_symbol_mesh_handle.clone(),
            Shape::Icosahedron(_) => icosahedron_symbol_mesh_handle.clone(),
        };

        commands
            .spawn(Mesh3d(symbol_mesh_handle))
            .insert(MeshMaterial3d(sprite_sheet_material_handle.clone()))
            .insert(SelectableLevel(level_index))
            .insert(transform.clone());

        let number_mesh_handle = number_mesh_handles.get(&level.nodes_per_edge).unwrap();

        commands
            .spawn(Mesh3d(number_mesh_handle.clone()))
            .insert(MeshMaterial3d(sprite_sheet_material_handle.clone()))
            .insert(SelectableLevel(level_index))
            .insert(transform.clone());
    }

    let mesh_builder = MazeMeshBuilder::icosahedron(1.0 / SQRT_3 / 3.0);
    let edge_mesh_handle = meshes.add(mesh_builder.cross_face_edge());

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
            .insert(MeshMaterial3d(game_materials.line_material.clone()))
            .insert(edge_transform);
    }
}

fn compute_face_transform(level_index: usize) -> Transform {
    let face_index = FACE_ORDER[level_index];

    let face = Icosahedron::FACES[face_index];
    let face_normal = Icosahedron::face_normal(&face);
    let face_center = Icosahedron::vertices(&face)
        .iter()
        .fold(Vec3::ZERO, |acc, item| acc + item)
        / 3.0
        / 2.0;

    let other_level_index = if level_index == 0 { 1 } else { level_index - 1 };
    let other_face_index = FACE_ORDER[other_level_index];
    let other_face = Icosahedron::FACES[other_face_index];

    let edge_points = face
        .into_iter()
        .collect::<HashSet<usize>>()
        .intersection(&other_face.into_iter().collect::<HashSet<usize>>())
        .cloned()
        .collect::<Vec<usize>>();

    let edge_midpoint = edge_points.iter().fold(Vec3::ZERO, |acc, item| {
        acc + Vec3::from_array(Icosahedron::VERTICES[*item])
    }) / 2.0
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

pub fn idle(
    camera_query: Query<(&GlobalTransform, &Camera), With<MainCamera>>,
    primary_window: Query<&Window, With<PrimaryWindow>>,
    mouse_buttons: Res<ButtonInput<MouseButton>>,
    rapier_context_query: Query<&RapierContext>,
    mut next_camera_state: ResMut<NextState<SelectorCameraState>>,
) {
    if !mouse_buttons.pressed(MouseButton::Left) || mouse_buttons.just_pressed(MouseButton::Left) {
        return;
    }

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

    if rapier_context_query
        .single()
        .cast_ray(
            ray.origin,
            ray.direction.into(),
            30.,
            true,
            QueryFilter::default(),
        )
        .is_some()
    {
    } else {
        next_camera_state.set(SelectorCameraState::Dolly);
    }
}

pub fn view(
    mouse_buttons: Res<ButtonInput<MouseButton>>,
    mut next_controller_state: ResMut<NextState<SelectorCameraState>>,
) {
    if !mouse_buttons.pressed(MouseButton::Left) || mouse_buttons.just_pressed(MouseButton::Left) {
        next_controller_state.set(SelectorCameraState::Idle);
        return;
    }
}
