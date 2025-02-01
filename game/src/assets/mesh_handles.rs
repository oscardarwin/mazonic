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
    pub faces: Vec<Handle<Mesh>>,
    pub same_face_edge: Handle<Mesh>,
    pub one_way_same_face_edge: Handle<Mesh>,
    pub cross_face_edge: Handle<Mesh>,
    pub one_way_cross_face_edge: Handle<Mesh>,
}

pub struct ShapesMeshHandles {
    pub tetrahedron: ShapeMeshHandles,
    pub cube: ShapeMeshHandles,
    pub octahedron: ShapeMeshHandles,
    pub dodecahedron: ShapeMeshHandles,
    pub icosahedron: ShapeMeshHandles,
}

#[derive(Resource)]
pub struct MeshHandles {
    pub player: Handle<Mesh>,
    pub player_halo: Handle<Mesh>,
    pub goal_room: Handle<Mesh>,
    pub junction_room: Handle<Mesh>,
    pub node_arrival_effect: Handle<Mesh>,
    pub shapes: ShapesMeshHandles,
}

pub fn setup_mesh_handles(mut meshes: ResMut<Assets<Mesh>>, mut commands: Commands) {
    let player = meshes.add(Sphere::new(0.25));
    let player_halo = meshes.add(Sphere::new(0.27));
    let goal_room = meshes.add(Circle::new(1.0 / 5.5));
    let junction_room = meshes.add(Circle::new(1.0 / 6.0));
    let node_arrival_effect = meshes.add(Circle::new(0.1));

    let shapes = setup_shapes_mesh_handles(&mut meshes);

    commands.insert_resource(MeshHandles {
        player,
        player_halo,
        goal_room,
        junction_room,
        node_arrival_effect,
        shapes,
    })
}

fn setup_shapes_mesh_handles(mut meshes: &mut ResMut<Assets<Mesh>>) -> ShapesMeshHandles {
    ShapesMeshHandles {
        tetrahedron: setup_shape_mesh_handles(
            meshes,
            TriangleFaceMeshGenerator::get_face_meshes(tetrahedron::faces()),
            MazeMeshBuilder::tetrahedron(1.0),
        ),
        cube: setup_shape_mesh_handles(
            meshes,
            SquareFaceMeshGenerator::get_face_meshes(cube::faces()),
            MazeMeshBuilder::cube(1.0),
        ),
        octahedron: setup_shape_mesh_handles(
            meshes,
            TriangleFaceMeshGenerator::get_face_meshes(octahedron::faces()),
            MazeMeshBuilder::octahedron(1.0),
        ),
        dodecahedron: setup_shape_mesh_handles(
            meshes,
            PentagonFaceMeshGenerator::get_face_meshes(dodecahedron::faces()),
            MazeMeshBuilder::dodecahedron(1.0),
        ),
        icosahedron: setup_shape_mesh_handles(
            meshes,
            TriangleFaceMeshGenerator::get_face_meshes(icosahedron::faces()),
            MazeMeshBuilder::icosahedron(1.0),
        ),
    }
}
fn setup_shape_mesh_handles(
    mut meshes: &mut ResMut<Assets<Mesh>>,
    face_meshes: Vec<Mesh>,
    maze_mesh_builder: MazeMeshBuilder,
) -> ShapeMeshHandles {
    let faces = face_meshes
        .into_iter()
        .map(|mesh| meshes.add(mesh))
        .collect();

    ShapeMeshHandles {
        faces,
        same_face_edge: meshes.add(maze_mesh_builder.same_face_edge()),
        one_way_same_face_edge: meshes.add(maze_mesh_builder.one_way_same_face_edge()),
        cross_face_edge: meshes.add(maze_mesh_builder.cross_face_edge()),
        one_way_cross_face_edge: meshes.add(maze_mesh_builder.one_way_cross_face_edge()),
    }
}
