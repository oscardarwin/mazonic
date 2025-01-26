use bevy::prelude::*;

use crate::shape::{cube, dodecahedron, icosahedron, octahedron, tetrahedron};

use super::mesh_generators::{
    FaceMeshGenerator, PentagonFaceMeshGenerator, SquareFaceMeshGenerator,
    TriangleFaceMeshGenerator,
};

#[derive(Resource)]
pub struct MeshHandles {
    pub player: Handle<Mesh>,
    pub player_halo: Handle<Mesh>,
    pub tetrahedron_faces: Vec<Handle<Mesh>>,
    pub cube_faces: Vec<Handle<Mesh>>,
    pub octahedron_faces: Vec<Handle<Mesh>>,
    pub dodecahedron_faces: Vec<Handle<Mesh>>,
    pub icosahedron_faces: Vec<Handle<Mesh>>,
}

pub fn setup_mesh_handles(mut meshes: ResMut<Assets<Mesh>>, mut commands: Commands) {
    let player = meshes.add(Sphere::default());
    let player_halo = meshes.add(Sphere::default());

    let tetrahedron_faces = TriangleFaceMeshGenerator::get_face_meshes(tetrahedron::faces())
        .into_iter()
        .map(|mesh| meshes.add(mesh))
        .collect();
    let cube_faces = SquareFaceMeshGenerator::get_face_meshes(cube::faces())
        .into_iter()
        .map(|mesh| meshes.add(mesh))
        .collect();
    let octahedron_faces = TriangleFaceMeshGenerator::get_face_meshes(octahedron::faces())
        .into_iter()
        .map(|mesh| meshes.add(mesh))
        .collect();
    let dodecahedron_faces = PentagonFaceMeshGenerator::get_face_meshes(dodecahedron::faces())
        .into_iter()
        .map(|mesh| meshes.add(mesh))
        .collect();
    let icosahedron_faces = TriangleFaceMeshGenerator::get_face_meshes(icosahedron::faces())
        .into_iter()
        .map(|mesh| meshes.add(mesh))
        .collect();

    commands.insert_resource(MeshHandles {
        player,
        player_halo,
        tetrahedron_faces,
        cube_faces,
        octahedron_faces,
        dodecahedron_faces,
        icosahedron_faces,
    })
}
