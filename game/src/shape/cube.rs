use std::{
    fmt::Debug,
    hash::{DefaultHasher, Hash, Hasher},
};

use bevy::{
    asset::RenderAssetUsages,
    ecs::{component::Component, system::Resource},
    math::Vec3,
    render::mesh::{Indices, Mesh, PrimitiveTopology},
};
use itertools::{iproduct, repeat_n};

use crate::{
    room::{Face, Room},
};

use super::shape_loader::face_indices_to_vertices;

const VERTEX_SCALING_FACTOR: f32 = 0.5;

const CUBE_VERTICES: [Vec3; 8] = [
    Vec3::new(-1.0, -1.0, -1.0),
    Vec3::new(-1.0, -1.0, 1.0),
    Vec3::new(-1.0, 1.0, -1.0),
    Vec3::new(-1.0, 1.0, 1.0),
    Vec3::new(1.0, -1.0, -1.0),
    Vec3::new(1.0, -1.0, 1.0),
    Vec3::new(1.0, 1.0, -1.0),
    Vec3::new(1.0, 1.0, 1.0),
];

pub const CUBE_FACES: [[usize; 4]; 6] = [
    [0, 2, 6, 4],
    [0, 1, 3, 2],
    [6, 7, 5, 4],
    [2, 3, 7, 6],
    [4, 5, 1, 0],
    [5, 7, 3, 1],
];

#[derive(Resource, Component, Clone, Debug)]
pub struct Cube {
    nodes_per_edge: u8,
    pub distance_between_nodes: f32,
}

impl Cube {
    pub const fn new(nodes_per_edge: u8) -> Self {
        let distance_between_nodes = 1.0 / (nodes_per_edge as f32);
        Self {
            nodes_per_edge,
            distance_between_nodes,
        }
    }

    pub fn get_faces() -> [[Vec3; 4]; 6] {
        face_indices_to_vertices(CUBE_FACES, &CUBE_VERTICES)
            .map(|face_vertices| face_vertices.map(|vertex| vertex * VERTEX_SCALING_FACTOR))
    }
}
