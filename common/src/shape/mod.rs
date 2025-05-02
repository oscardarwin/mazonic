use bevy::{pbr::ExtendedMaterial, prelude::*};

use crate::{assets::{material_handles::MaterialHandles, mesh_handles::MeshHandles, shaders::GlobalShader}, levels::{GameLevel, PuzzleEntityMarker, Shape}};

pub mod cube;
pub mod dodecahedron;
pub mod icosahedron;
pub mod loader;
pub mod octahedron;
pub mod shape_utils;
pub mod tetrahedron;

pub fn spawn(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mesh_handles: Res<MeshHandles>,
    level_query: Query<&GameLevel>,
    material_handles: Res<MaterialHandles>,
) {
    let Ok(level) = level_query.get_single() else {
        return;
    };

    let face_materials_handles = &material_handles.face_handles;

    let materials: Vec<Handle<ExtendedMaterial<StandardMaterial, GlobalShader>>> =
        match &level.shape {
            Shape::Cube => face_materials_handles.cube(&level.face_color_permutation).into_iter().collect(),
            Shape::Tetrahedron => face_materials_handles.tetrahedron(&level.face_color_permutation).into_iter().collect(),
            Shape::Octahedron => face_materials_handles.octahedron(&level.face_color_permutation).into_iter().collect(),
            Shape::Dodecahedron => face_materials_handles.dodecahedron().into_iter().collect(),
            Shape::Icosahedron => face_materials_handles.icosahedron().into_iter().collect(),
        };

    let face_mesh_handles = match &level.shape {
        Shape::Tetrahedron => mesh_handles.shape_mesh_handles.tetrahedron.to_vec(),
        Shape::Cube => mesh_handles.shape_mesh_handles.cube.to_vec(),
        Shape::Octahedron => mesh_handles.shape_mesh_handles.octahedron.to_vec(),
        Shape::Dodecahedron => mesh_handles.shape_mesh_handles.dodecahedron.to_vec(),
        Shape::Icosahedron => mesh_handles.shape_mesh_handles.icosahedron.to_vec(),
    };

    for (face_mesh_handle, face_material_handle) in
        face_mesh_handles.into_iter().zip(materials.into_iter())
    {
        commands
            .spawn(Mesh3d(face_mesh_handle.clone()))
            .insert(MeshMaterial3d(face_material_handle))
            .insert(PuzzleEntityMarker);
    }
}
