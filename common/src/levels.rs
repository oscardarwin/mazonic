use bevy::{prelude::*, utils::HashSet};
use serde::{Deserialize, Serialize};

use crate::{
    constants::{SQRT_3, TAN_27},
    maze::border_type::BorderType,
    room::Face,
    shape::{cube, dodecahedron, icosahedron, octahedron, tetrahedron},
};

#[derive(Component)]
pub struct LevelData;

#[derive(Clone, Debug, Serialize, Deserialize)]
pub enum Shape {
    Cube,
    Tetrahedron,
    Icosahedron,
    Octahedron,
    Dodecahedron,
}

#[derive(Component, Clone, Debug)]
pub struct GameLevel {
    pub shape: Shape,
    pub nodes_per_edge: u8,
}

impl GameLevel {
    pub const fn new(shape: Shape, nodes_per_edge: u8) -> Self {
        GameLevel {
            shape,
            nodes_per_edge,
        }
    }

    pub fn border_type(&self, from: &Face, to: &Face) -> Option<BorderType> {
        let from_vertex_set = self.get_face_indices(from);
        let to_vertex_set = self.get_face_indices(to);

        match from_vertex_set.intersection(&to_vertex_set).count() {
            0 | 1 => None,
            2 => Some(BorderType::Connected),
            _ => Some(BorderType::SameFace),
        }
    }

    fn get_face_indices(&self, face: &Face) -> HashSet<usize> {
        let indices = match self.shape {
            Shape::Tetrahedron => tetrahedron::FACE_INDICES[face.id()].to_vec(),
            Shape::Cube => cube::FACE_INDICES[face.id()].to_vec(),
            Shape::Octahedron => octahedron::FACE_INDICES[face.id()].to_vec(),
            Shape::Dodecahedron => dodecahedron::FACE_INDICES[face.id()].to_vec(),
            Shape::Icosahedron => icosahedron::FACE_INDICES[face.id()].to_vec(),
        };

        indices.into_iter().collect()
    }

    pub fn node_distance(&self) -> f32 {
        match &self.shape {
            Shape::Tetrahedron | Shape::Octahedron | Shape::Icosahedron => {
                1.0 / (self.nodes_per_edge as f32 - 1.0 + SQRT_3)
            }
            Shape::Cube => 1.0 / (self.nodes_per_edge as f32),
            Shape::Dodecahedron => TAN_27,
        }
    }

 pub const fn tetrahedron(nodes_per_edge: u8) -> GameLevel {
        let shape = Shape::Tetrahedron;
        GameLevel::new(shape, nodes_per_edge)
    }

    pub const fn cube(nodes_per_edge: u8) -> GameLevel {
        let shape = Shape::Cube;
        GameLevel::new(shape, nodes_per_edge)
    }

    pub const fn octahedron(nodes_per_edge: u8) -> GameLevel {
        let shape = Shape::Octahedron;
        GameLevel::new(shape, nodes_per_edge)
    }

    pub const fn dodecahedron() -> GameLevel {
        let shape = Shape::Dodecahedron;
        GameLevel::new(shape, 1)
    }

    pub const fn icosahedron(nodes_per_edge: u8) -> GameLevel {
        let shape = Shape::Icosahedron;
        GameLevel::new(shape, nodes_per_edge)
    }
}

pub const LEVELS: [GameLevel; 18] = [
    GameLevel::tetrahedron(1),
    GameLevel::cube(2),
    GameLevel::octahedron(3),
    GameLevel::dodecahedron(),
    GameLevel::icosahedron(2),
    GameLevel::octahedron(4),
    GameLevel::tetrahedron(6),
    GameLevel::cube(4),
    GameLevel::tetrahedron(7),
    GameLevel::octahedron(5),
    GameLevel::icosahedron(3),
    GameLevel::tetrahedron(8),
    GameLevel::cube(5),
    GameLevel::octahedron(6),
    GameLevel::tetrahedron(9),
    GameLevel::icosahedron(4),
    GameLevel::cube(6),
    GameLevel::icosahedron(5),
];
