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
            Shape::Cube(coloring) => face_materials_handles.cube(&coloring).into_iter().collect(),
            Shape::Tetrahedron(coloring) => face_materials_handles.tetrahedron(&coloring).into_iter().collect(),
            Shape::Octahedron(coloring) => face_materials_handles.octahedron(&coloring).into_iter().collect(),
            Shape::Dodecahedron(coloring) => face_materials_handles.dodecahedron(&coloring).into_iter().collect(),
            Shape::Icosahedron(coloring) => face_materials_handles.icosahedron(&coloring).into_iter().collect(),
        };

    let face_mesh_handles = match &level.shape {
        Shape::Tetrahedron(_) => mesh_handles.shape_mesh_handles.tetrahedron.to_vec(),
        Shape::Cube(_) => mesh_handles.shape_mesh_handles.cube.to_vec(),
        Shape::Octahedron(_) => mesh_handles.shape_mesh_handles.octahedron.to_vec(),
        Shape::Dodecahedron(_) => mesh_handles.shape_mesh_handles.dodecahedron.to_vec(),
        Shape::Icosahedron(_) => mesh_handles.shape_mesh_handles.icosahedron.to_vec(),
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
