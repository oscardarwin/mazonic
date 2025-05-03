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
use serde::{Deserialize, Serialize};

use crate::room::{Face, Room};

use super::shape_utils::face_indices_to_vertices;

const VERTEX_SCALING_FACTOR: f32 = 0.5;

const VERTICES: [Vec3; 8] = [
    Vec3::new(-1.0, -1.0, -1.0),
    Vec3::new(-1.0, -1.0, 1.0),
    Vec3::new(-1.0, 1.0, -1.0),
    Vec3::new(-1.0, 1.0, 1.0),
    Vec3::new(1.0, -1.0, -1.0),
    Vec3::new(1.0, -1.0, 1.0),
    Vec3::new(1.0, 1.0, -1.0),
    Vec3::new(1.0, 1.0, 1.0),
];

pub const FACE_INDICES: [[usize; 4]; 6] = [
    [0, 2, 6, 4],
    [0, 1, 3, 2],
    [6, 7, 5, 4],
    [2, 3, 7, 6],
    [4, 5, 1, 0],
    [5, 7, 3, 1],
];

fn vertices() -> [Vec3; 8] {
    VERTICES.map(|position| position * VERTEX_SCALING_FACTOR)
}

pub fn faces() -> [[Vec3; 4]; 6] {
    face_indices_to_vertices(FACE_INDICES, &vertices())
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum Coloring {
    Full([u8; 3]),
    Dual([u8; 2]),
    Mono(u8),
}
