use bevy::prelude::*;

use crate::{
    maze::maze_mesh_builder::MazeMeshBuilder,
    shape::{cube, dodecahedron, icosahedron, octahedron, tetrahedron},
};

use super::mesh_generators::{
    FaceMeshGenerator, PentagonFaceMeshGenerator, SquareFaceMeshGenerator,
    TriangleFaceMeshGenerator,
};

pub struct ShapeMeshHandles {
    pub tetrahedron: [Handle<Mesh>; 4],
    pub cube: [Handle<Mesh>; 6],
    pub octahedron: [Handle<Mesh>; 8],
    pub dodecahedron: [Handle<Mesh>; 12],
    pub icosahedron: [Handle<Mesh>; 20],
}

#[derive(Resource)]
pub struct MeshHandles {
    pub player: Handle<Mesh>,
    pub player_halo: Handle<Mesh>,
    pub goal_room: Handle<Mesh>,
    pub junction_room: Handle<Mesh>,
    pub node_arrival_effect: Handle<Mesh>,
    pub shape_mesh_handles: ShapeMeshHandles,
}

pub fn setup_mesh_handles(mut meshes: ResMut<Assets<Mesh>>, mut commands: Commands) {
    let player = meshes.add(Sphere::new(0.25));
    let player_halo = meshes.add(Sphere::new(0.27));
    let goal_room = meshes.add(Circle::new(1.0 / 5.5));
    let junction_room = meshes.add(Circle::new(1.0 / 6.0));
    let node_arrival_effect = meshes.add(Circle::new(0.1));
    let shape_mesh_handles = get_shape_mesh_handles(&mut meshes);

    commands.insert_resource(MeshHandles {
        player,
        player_halo,
        goal_room,
        junction_room,
        node_arrival_effect,
        shape_mesh_handles,
    })
}

fn get_shape_mesh_handles(mut meshes: &mut Assets<Mesh>) -> ShapeMeshHandles {
    let tetrahedron = TriangleFaceMeshGenerator::load_mesh_asset(&mut meshes, tetrahedron::faces());
    let cube = SquareFaceMeshGenerator::load_mesh_asset(&mut meshes, cube::faces());
    let octahedron = TriangleFaceMeshGenerator::load_mesh_asset(&mut meshes, octahedron::faces());
    let dodecahedron =
        PentagonFaceMeshGenerator::load_mesh_asset(&mut meshes, dodecahedron::faces());
    let icosahedron = TriangleFaceMeshGenerator::load_mesh_asset(&mut meshes, icosahedron::faces());

    ShapeMeshHandles {
        tetrahedron,
        cube,
        octahedron,
        dodecahedron,
        icosahedron,
    }
}
