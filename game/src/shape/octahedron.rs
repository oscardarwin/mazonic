use std::{f32::consts::SQRT_2, fmt::Debug, usize};

use bevy::{
    ecs::{component::Component, system::Resource},
    math::Vec3,
    render::mesh::Mesh,
};

use crate::{
    constants::SQRT_3,
    room::{Face, Room},
};

use super::{shape_loader::face_indices_to_vertices, triangle_face_generator};

const VERTEX_SCALING_FACTOR: f32 = SQRT_2 / 2.0;

const VERTICES: [Vec3; 6] = [
    Vec3::new(1.0, 0.0, 0.0),
    Vec3::new(-1.0, 0.0, 0.0),
    Vec3::new(0.0, 1.0, 0.0),
    Vec3::new(0.0, -1.0, 0.0),
    Vec3::new(0.0, 0.0, 1.0),
    Vec3::new(0.0, 0.0, -1.0),
];

pub const FACE_INDICES: [[usize; 3]; 8] = [
    [0, 2, 4],
    [0, 4, 3],
    [0, 3, 5],
    [0, 5, 2],
    [1, 4, 2],
    [1, 3, 4],
    [1, 5, 3],
    [1, 2, 5],
];

fn vertices() -> [Vec3; 6] {
    VERTICES.map(|position| position * VERTEX_SCALING_FACTOR)
}

pub fn faces() -> [[Vec3; 3]; 8] {
    face_indices_to_vertices(FACE_INDICES, &vertices())
}
