use bevy::{prelude::*, utils::HashSet};
use serde::{Deserialize, Serialize};

use crate::{
    constants::{SQRT_3, TAN_27},
    maze::border_type::BorderType,
    room::Face,
    shape::{cube, dodecahedron, icosahedron, octahedron, tetrahedron},
};

#[derive(Component)]
pub struct PuzzleEntityMarker;

pub fn despawn_puzzle_entities(mut commands: Commands, level_entities: Query<Entity, With<PuzzleEntityMarker>>) {
    for entity in level_entities.iter() {
        commands.entity(entity).despawn_recursive();
    }
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub enum Shape {
    Tetrahedron(tetrahedron::Coloring),
    Cube(cube::Coloring),
    Octahedron(octahedron::Coloring),
    Dodecahedron(dodecahedron::Coloring),
    Icosahedron(icosahedron::Coloring),
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
            Shape::Tetrahedron(_) => tetrahedron::FACE_INDICES[face.id()].to_vec(),
            Shape::Cube(_) => cube::FACE_INDICES[face.id()].to_vec(),
            Shape::Octahedron(_) => octahedron::FACE_INDICES[face.id()].to_vec(),
            Shape::Dodecahedron(_) => dodecahedron::FACE_INDICES[face.id()].to_vec(),
            Shape::Icosahedron(_) => icosahedron::FACE_INDICES[face.id()].to_vec(),
        };

        indices.into_iter().collect()
    }

    pub fn node_distance(&self) -> f32 {
        match &self.shape {
            Shape::Tetrahedron(_) | Shape::Octahedron(_) | Shape::Icosahedron(_) => {
                1.0 / (self.nodes_per_edge as f32 - 1.0 + SQRT_3)
            }
            Shape::Cube(_) => 1.0 / (self.nodes_per_edge as f32),
            Shape::Dodecahedron(_) => TAN_27,
        }
    }

    pub const fn tetrahedron(nodes_per_edge: u8, coloring: tetrahedron::Coloring) -> GameLevel {
        let shape = Shape::Tetrahedron(coloring);
        GameLevel::new(shape, nodes_per_edge)
    }

    pub const fn cube(nodes_per_edge: u8, coloring: cube::Coloring) -> GameLevel {
        let shape = Shape::Cube(coloring);
        GameLevel::new(shape, nodes_per_edge)
    }

    pub const fn octahedron(nodes_per_edge: u8, coloring: octahedron::Coloring) -> GameLevel {
        let shape = Shape::Octahedron(coloring);
        GameLevel::new(shape, nodes_per_edge)
    }

    pub const fn dodecahedron(coloring: dodecahedron::Coloring) -> GameLevel {
        let shape = Shape::Dodecahedron(coloring);
        GameLevel::new(shape, 1)
    }

    pub const fn icosahedron(nodes_per_edge: u8, coloring: icosahedron::Coloring) -> GameLevel {
        let shape = Shape::Icosahedron(coloring);
        GameLevel::new(shape, nodes_per_edge)
    }
}

pub const LEVELS: [GameLevel; 18] = [
    GameLevel::tetrahedron(1, tetrahedron::Coloring::Full([0, 1, 2, 3])),
    GameLevel::cube(2, cube::Coloring::Full([1, 2, 3])),
    GameLevel::octahedron(3, octahedron::Coloring::Full([0, 1, 2, 4])),
    GameLevel::dodecahedron(dodecahedron::Coloring::Full([1, 2, 3, 4])),
    GameLevel::icosahedron(2, icosahedron::Coloring::Full([0, 1, 2, 3, 4])),
    GameLevel::octahedron(4, octahedron::Coloring::Stripes([0, 1, 2, 4])),
    GameLevel::tetrahedron(6, tetrahedron::Coloring::Full([0, 2, 3, 4])),
    GameLevel::cube(4, cube::Coloring::Full([1, 2, 3])),
    GameLevel::tetrahedron(7, tetrahedron::Coloring::Full([0, 1, 3, 4])),
    GameLevel::octahedron(5, octahedron::Coloring::CrissCross([2, 4])),
    GameLevel::icosahedron(3, icosahedron::Coloring::Tri([0, 1, 2])),
    GameLevel::tetrahedron(8, tetrahedron::Coloring::Dual([1, 2])),
    GameLevel::cube(5, cube::Coloring::Dual([0, 3])),
    GameLevel::octahedron(6, octahedron::Coloring::Dual([0, 4])),
    GameLevel::tetrahedron(9, tetrahedron::Coloring::Mono(1)),
    GameLevel::icosahedron(4, icosahedron::Coloring::Dual([0, 3])),
    GameLevel::cube(6, cube::Coloring::Mono(2)),
    GameLevel::icosahedron(5, icosahedron::Coloring::Mono(4)),
];
