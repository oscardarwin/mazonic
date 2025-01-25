use bevy::{prelude::*, utils::HashSet};

use crate::{
    constants::{SQRT_3, TAN_27},
    maze::border_type::BorderType,
    room::Face,
    shape::{cube, dodecahedron, icosahedron, octahedron, tetrahedron},
};

#[derive(Component)]
pub struct LevelData;

#[derive(Clone, Debug)]
pub enum Shape {
    Cube,
    Tetrahedron,
    Icosahedron,
    Octahedron,
    Dodecahedron,
}

#[derive(Component, Clone, Debug)]
pub struct GameLevel {
    pub seed: u64,
    pub shape: Shape,
    pub nodes_per_edge: u8,
}

impl GameLevel {
    const fn new(seed: u64, shape: Shape, nodes_per_edge: u8) -> Self {
        GameLevel {
            seed,
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

    pub const fn tetrahedron(nodes_per_edge: u8, seed: u64) -> GameLevel {
        let shape = Shape::Tetrahedron;
        GameLevel::new(seed, shape, nodes_per_edge)
    }

    pub const fn cube(nodes_per_edge: u8, seed: u64) -> GameLevel {
        let shape = Shape::Cube;
        GameLevel::new(seed, shape, nodes_per_edge)
    }

    pub const fn octahedron(nodes_per_edge: u8, seed: u64) -> GameLevel {
        let shape = Shape::Octahedron;
        GameLevel::new(seed, shape, nodes_per_edge)
    }

    pub const fn dodecahedron(seed: u64) -> GameLevel {
        let shape = Shape::Dodecahedron;
        GameLevel::new(seed, shape, 1)
    }

    pub const fn icosahedron(nodes_per_edge: u8, seed: u64) -> GameLevel {
        let shape = Shape::Icosahedron;
        GameLevel::new(seed, shape, nodes_per_edge)
    }

    pub fn filename(&self) -> String {
        let shape = match &self.shape {
            Shape::Cube => "cube",
            Shape::Tetrahedron => "tetrahedron",
            Shape::Octahedron => "octahedron",
            Shape::Dodecahedron => "dodecahedron",
            Shape::Icosahedron => "icosahedron",
        };

        format!(
            "levels/{}_s{:?}_n{:?}.json",
            shape, self.seed, self.nodes_per_edge
        )
    }
}

pub const LEVELS: [GameLevel; 20] = [
    GameLevel::tetrahedron(1, 1),
    GameLevel::cube(2, 2),
    GameLevel::octahedron(3, 3),
    GameLevel::dodecahedron(1),
    GameLevel::icosahedron(2, 2),
    GameLevel::octahedron(4, 4),
    GameLevel::tetrahedron(6, 0),
    GameLevel::cube(4, 3),
    GameLevel::tetrahedron(7, 0),
    GameLevel::octahedron(5, 0),
    GameLevel::icosahedron(3, 2),
    GameLevel::tetrahedron(8, 0),
    GameLevel::cube(5, 0),
    GameLevel::octahedron(6, 0),
    GameLevel::tetrahedron(9, 0),
    GameLevel::icosahedron(4, 2),
    GameLevel::cube(6, 1),
    GameLevel::octahedron(7, 0),
    GameLevel::cube(7, 0),
    GameLevel::icosahedron(5, 0),
];
