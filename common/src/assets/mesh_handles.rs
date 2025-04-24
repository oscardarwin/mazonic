use bevy::prelude::*;

use crate::{
    maze::maze_mesh_builder::MazeMeshBuilder,
    shape::{cube, dodecahedron, icosahedron, octahedron, tetrahedron},
};

use super::mesh_generators::{
    FaceMeshGenerator, PentagonFaceMeshGenerator, SquareFaceMeshGenerator,
    TriangleFaceMeshGenerator,
};

pub struct MazeEdgeMeshHandles {
    pub same_face_edge: Handle<Mesh>,
    pub one_way_same_face_edge: Handle<Mesh>,
    pub cross_face_edge: Handle<Mesh>,
    pub one_way_cross_face_edge: Handle<Mesh>,
}

pub struct ShapeMazeEdgeMeshHandles {
    pub tetrahedron: MazeEdgeMeshHandles,
    pub cube: MazeEdgeMeshHandles,
    pub octahedron: MazeEdgeMeshHandles,
    pub dodecahedron: MazeEdgeMeshHandles,
    pub icosahedron: MazeEdgeMeshHandles,
}

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
    pub shape_maze_edge_mesh_handles: ShapeMazeEdgeMeshHandles,
}

pub fn setup_mesh_handles(mut meshes: ResMut<Assets<Mesh>>, mut commands: Commands) {
    let player = meshes.add(Sphere::new(1.0));
    let player_halo = meshes.add(Sphere::new(1.08));
    let goal_room = meshes.add(Circle::new(1.0 / 5.5));
    let junction_room = meshes.add(Circle::new(1.0 / 6.0));
    let node_arrival_effect = meshes.add(Circle::new(0.1));
    let shape_mesh_handles = get_shape_mesh_handles(&mut meshes);
    let shape_maze_edge_mesh_handles = get_shape_maze_edge_mesh_handles(&mut meshes);

    commands.insert_resource(MeshHandles {
        player,
        player_halo,
        goal_room,
        junction_room,
        node_arrival_effect,
        shape_mesh_handles,
        shape_maze_edge_mesh_handles,
    })
}

fn get_shape_maze_edge_mesh_handles(mut meshes: &mut Assets<Mesh>) -> ShapeMazeEdgeMeshHandles {
    let tetrahedron = get_maze_edge_mesh_handles(&mut meshes, MazeMeshBuilder::tetrahedron());
    let cube = get_maze_edge_mesh_handles(&mut meshes, MazeMeshBuilder::cube());
    let octahedron = get_maze_edge_mesh_handles(&mut meshes, MazeMeshBuilder::octahedron());
    let dodecahedron = get_maze_edge_mesh_handles(&mut meshes, MazeMeshBuilder::dodecahedron());
    let icosahedron = get_maze_edge_mesh_handles(&mut meshes, MazeMeshBuilder::icosahedron());

    ShapeMazeEdgeMeshHandles {
        tetrahedron,
        cube,
        octahedron,
        dodecahedron,
        icosahedron,
    }
}

fn get_maze_edge_mesh_handles(
    mut meshes: &mut Assets<Mesh>,
    maze_edge_mesh_builder: MazeMeshBuilder,
) -> MazeEdgeMeshHandles {
    let same_face_edge = meshes.add(maze_edge_mesh_builder.same_face_edge());
    let one_way_same_face_edge = meshes.add(maze_edge_mesh_builder.one_way_same_face_edge());
    let cross_face_edge = meshes.add(maze_edge_mesh_builder.cross_face_edge());
    let one_way_cross_face_edge = meshes.add(maze_edge_mesh_builder.one_way_cross_face_edge());

    MazeEdgeMeshHandles {
        same_face_edge,
        one_way_same_face_edge,
        cross_face_edge,
        one_way_cross_face_edge,
    }
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
