use bevy::prelude::*;

use crate::{
    maze::maze_mesh_builder::MazeMeshBuilder,
    shape::{cube, dodecahedron, icosahedron, octahedron, tetrahedron},
};

use super::mesh_generators::{
    FaceMeshGenerator, PentagonFaceMeshGenerator, SquareFaceMeshGenerator,
    TriangleFaceMeshGenerator,
};

#[derive(Resource)]
pub struct MeshHandles {
    pub player: Handle<Mesh>,
    pub player_halo: Handle<Mesh>,
    pub goal_room: Handle<Mesh>,
    pub junction_room: Handle<Mesh>,
    pub node_arrival_effect: Handle<Mesh>,
}

pub fn setup_mesh_handles(mut meshes: ResMut<Assets<Mesh>>, mut commands: Commands) {
    let player = meshes.add(Sphere::new(0.25));
    let player_halo = meshes.add(Sphere::new(0.27));
    let goal_room = meshes.add(Circle::new(1.0 / 5.5));
    let junction_room = meshes.add(Circle::new(1.0 / 6.0));
    let node_arrival_effect = meshes.add(Circle::new(0.1));

    commands.insert_resource(MeshHandles {
        player,
        player_halo,
        goal_room,
        junction_room,
        node_arrival_effect,
    })
}
