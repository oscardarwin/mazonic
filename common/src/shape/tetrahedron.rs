use std::{f32::consts::SQRT_2, fmt::Debug};

use bevy::{
    ecs::{component::Component, system::Resource},
    math::Vec3,
    render::mesh::Mesh,
};
use serde::{Deserialize, Serialize};

use crate::{
    constants::SQRT_3,
    room::{Face, Room},
};

use super::shape_utils::face_indices_to_vertices;

const VERTEX_SCALING_FACTOR: f32 = 1.0 / SQRT_2 / 2.0;

const VERTICES: [Vec3; 4] = [
    Vec3::new(1.0, 1.0, 1.0),
    Vec3::new(-1.0, 1.0, -1.0),
    Vec3::new(-1.0, -1.0, 1.0),
    Vec3::new(1.0, -1.0, -1.0),
];

pub const FACE_INDICES: [[usize; 3]; 4] = [[3, 2, 1], [0, 2, 3], [3, 1, 0], [0, 1, 2]];

fn vertices() -> [Vec3; 4] {
    VERTICES.map(|position| position * VERTEX_SCALING_FACTOR)
}

pub fn faces() -> [[Vec3; 3]; 4] {
    face_indices_to_vertices(FACE_INDICES, &vertices())
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum Coloring {
    Full([u8; 4]),
    Dual([u8; 2]),
    Mono(u8),
}
