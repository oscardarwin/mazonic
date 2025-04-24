use std::{
    f32::consts::PI,
    fmt::Debug,
    hash::{DefaultHasher, Hash, Hasher},
    usize,
};

use bevy::{
    asset::RenderAssetUsages,
    ecs::{component::Component, system::Resource},
    math::Vec3,
    render::mesh::{Indices, Mesh, PrimitiveTopology},
};

use itertools::repeat_n;

use crate::{
    constants::{PHI, TAN_27},
    room::{Face, Room},
};

use super::shape_utils::face_indices_to_vertices;

const VERTEX_SCALING_FACTOR: f32 = PHI / 2.0;

const VERTICES: [Vec3; 20] = [
    Vec3::new(1.0 / PHI, PHI, 0.0),
    Vec3::new(PHI, 0.0, 1.0 / PHI),
    Vec3::new(0.0, 1.0 / PHI, PHI),
    Vec3::new(-1.0 / PHI, -PHI, 0.0),
    Vec3::new(-PHI, 0.0, 1.0 / PHI),
    Vec3::new(0.0, -1.0 / PHI, PHI),
    Vec3::new(1.0 / PHI, -PHI, 0.0),
    Vec3::new(PHI, 0.0, -1.0 / PHI),
    Vec3::new(0.0, -1.0 / PHI, -PHI),
    Vec3::new(-1.0 / PHI, PHI, 0.0),
    Vec3::new(-PHI, 0.0, -1.0 / PHI),
    Vec3::new(0.0, 1.0 / PHI, -PHI),
    Vec3::new(1.0, 1.0, 1.0),
    Vec3::new(1.0, -1.0, 1.0),
    Vec3::new(-1.0, -1.0, 1.0),
    Vec3::new(-1.0, 1.0, 1.0),
    Vec3::new(1.0, -1.0, -1.0),
    Vec3::new(1.0, 1.0, -1.0),
    Vec3::new(-1.0, 1.0, -1.0),
    Vec3::new(-1.0, -1.0, -1.0),
];

pub const FACE_INDICES: [[usize; 5]; 12] = [
    [0, 9, 15, 2, 12],
    [0, 17, 11, 18, 9],
    [0, 12, 1, 7, 17],
    [1, 13, 6, 16, 7],
    [1, 12, 2, 5, 13],
    [2, 15, 4, 14, 5],
    [3, 6, 13, 5, 14],
    [3, 19, 8, 16, 6],
    [3, 14, 4, 10, 19],
    [4, 15, 9, 18, 10],
    [7, 16, 8, 11, 17],
    [8, 19, 10, 18, 11],
];

fn vertices() -> [Vec3; 20] {
    VERTICES.map(|position| position * VERTEX_SCALING_FACTOR)
}

pub fn faces() -> [[Vec3; 5]; 12] {
    face_indices_to_vertices(FACE_INDICES, &vertices())
}
