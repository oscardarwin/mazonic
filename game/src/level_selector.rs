use bevy::{prelude::*, utils::HashMap, window::PrimaryWindow};
use bevy_rapier3d::prelude::*;

use crate::{
    assets::GameAssetHandles,
    camera::MainCamera,
    game_settings::GameSettings,
    game_state::GameState,
    levels::LEVELS,
    shape::{icosahedron::Icosahedron, loader::Shape, shape_loader::ShapeMeshLoader},
};

const SYMBOL_TEXTURE_DIMENSIONS: Vec2 = Vec2::new(5.0, 3.0);

const FACE_ORDER: [usize; 20] = [
    0, 2, 1, 4, 3, 11, 12, 5, 6, 7, 8, 19, 17, 16, 15, 14, 13, 10, 9, 18,
];

#[derive(SubStates, Hash, Eq, Clone, PartialEq, Debug, Default)]
#[source(GameState = GameState::Selector)]
pub enum SelectorCameraState {
    Dolly,
    #[default]
    Idle,
}

pub fn load(
    mut commands: Commands,
    mut asset_server: ResMut<AssetServer>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    game_settings: Res<GameSettings>,
) {
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

    let color_palette = color_palette(game_settings.into_inner());

    let face_material_handles = color_palette
        .into_iter()
        .map(|material| materials.add(material))
        .collect::<Vec<Handle<StandardMaterial>>>();

    for ((level, face_material_handle), face_index) in
        LEVELS.iter().zip(face_material_handles).zip(FACE_ORDER)
    {
        let face_mesh = face_meshes[face_index].clone();
        let face_mesh_handle = meshes.add(face_mesh);
        commands
            .spawn(Mesh3d(face_mesh_handle))
            .insert(MeshMaterial3d(face_material_handle.clone()));

        let face = Icosahedron::FACES[face_index];
        let face_normal = Icosahedron::face_normal(&face);
        let face_center = Icosahedron::vertices(&face)
            .iter()
            .fold(Vec3::ZERO, |acc, item| acc + item)
            / 6.0;

        let transform = Transform::IDENTITY
            .with_scale(Vec3::splat(0.5))
            .looking_at(-face_normal.clone(), face_normal.any_orthogonal_vector())
            .with_translation(face_center + face_normal * 0.003);

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
            .insert(transform.clone());

        let number_mesh_handle = number_mesh_handles.get(&level.nodes_per_edge).unwrap();

        commands
            .spawn(Mesh3d(number_mesh_handle.clone()))
            .insert(MeshMaterial3d(sprite_sheet_material_handle.clone()))
            .insert(transform.clone());
    }
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
        // if the cursor is not inside the window, we can't do anything
        return;
    };

    let (camera_global_transform, camera) = camera_query.single();

    let Some(ray) = camera
        .viewport_to_world(camera_global_transform, cursor_position)
        .ok()
    else {
        // if it was impossible to compute for whatever reason; we can't do anything
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
