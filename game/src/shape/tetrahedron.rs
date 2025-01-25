use std::{f32::consts::SQRT_2, fmt::Debug};

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

const VERTEX_SCALING_FACTOR: f32 = 1.0 / SQRT_2 / 2.0;

const TETRAHEDRON_VERTICES: [Vec3; 4] = [
    Vec3::new(1.0, 1.0, 1.0),
    Vec3::new(-1.0, 1.0, -1.0),
    Vec3::new(-1.0, -1.0, 1.0),
    Vec3::new(1.0, -1.0, -1.0),
];

pub const TETRAHEDRON_FACES: [[usize; 3]; 4] = [[3, 2, 1], [0, 2, 3], [3, 1, 0], [0, 1, 2]];

#[derive(Resource, Component, Clone, Debug)]
pub struct Tetrahedron;

impl Tetrahedron {
    pub fn get_faces() -> [[Vec3; 3]; 4] {
        face_indices_to_vertices(TETRAHEDRON_FACES, &TETRAHEDRON_VERTICES)
            .map(|face_vertices| face_vertices.map(|vertex| vertex * VERTEX_SCALING_FACTOR))
    }
}
