use std::{fmt::Debug, usize};

use bevy::{
    ecs::{component::Component, system::Resource},
    math::Vec3,
    reflect::Array,
    render::mesh::Mesh,
};

use crate::{
    constants::{PHI, SQRT_3},
    room::{Face, Room},
};

use super::{shape_loader::face_indices_to_vertices, triangle_face_generator};

const VERTEX_SCALING_FACTOR: f32 = 0.5;

pub const VERTICES: [Vec3; 12] = [
    Vec3::new(1.0, PHI, 0.0),
    Vec3::new(1.0, -PHI, 0.0),
    Vec3::new(-1.0, PHI, 0.0),
    Vec3::new(-1.0, -PHI, 0.0),
    Vec3::new(0.0, 1.0, PHI),
    Vec3::new(0.0, 1.0, -PHI),
    Vec3::new(0.0, -1.0, PHI),
    Vec3::new(0.0, -1.0, -PHI),
    Vec3::new(PHI, 0.0, 1.0),
    Vec3::new(-PHI, 0.0, 1.0),
    Vec3::new(PHI, 0.0, -1.0),
    Vec3::new(-PHI, 0.0, -1.0),
];

pub const FACE_INDICES: [[usize; 3]; 20] = [
    [0, 4, 8],
    [0, 10, 5],
    [0, 8, 10],
    [4, 0, 2],
    [5, 2, 0],
    [8, 4, 6],
    [1, 8, 6],
    [1, 10, 8],
    [1, 7, 10],
    [7, 1, 3],
    [6, 3, 1],
    [2, 9, 4],
    [9, 6, 4],
    [3, 6, 9],
    [3, 9, 11],
    [2, 11, 9],
    [2, 5, 11],
    [11, 5, 7],
    [3, 11, 7],
    [10, 7, 5],
];

pub fn vertices() -> [Vec3; 12] {
    VERTICES.map(|position| position * VERTEX_SCALING_FACTOR)
}

pub fn faces() -> [[Vec3; 3]; 20] {
    face_indices_to_vertices(FACE_INDICES, &vertices())
}
