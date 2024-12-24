use std::{
    cmp::Ordering,
    f32::consts::{FRAC_PI_2, FRAC_PI_3, SQRT_2},
    fmt::Debug,
    hash::{Hash, Hasher},
    ops::Not,
};

use bevy::{
    ecs::system::Resource,
    math::{primitives::Cuboid, NormedVectorSpace, Vec3},
};

use bevy::math::primitives::Tetrahedron as BevyTetrahedron;
use itertools::iproduct;
use maze_generator::{model::TraversalGraph, traversal_graph_generator::TraversalGraphGenerator};
use strum_macros::EnumIter;

use crate::shape::platonic_solid::{BorderType, Edge, HasFace, IsRoom, PlatonicSolid};

use super::platonic_mesh_builder::PlatonicMeshBuilder;

const VERTICES: [Vec3; 4] = [
    Vec3::new(0.5, 0.5, 0.5),
    Vec3::new(-0.5, 0.5, -0.5), // 3 / 36
    Vec3::new(-0.5, -0.5, 0.5),
    Vec3::new(0.5, -0.5, -0.5),
];

#[derive(EnumIter, Debug, Clone, Hash, Eq, PartialEq, Copy, PartialOrd, Ord)]
pub enum TetrahedronFace {
    ABD,
    BCD,
    CBA,
    DCA,
}

impl TetrahedronFace {
    fn defining_vectors(&self) -> (Vec3, Vec3) {
        let (vec_1, vec_2) = match self {
            TetrahedronFace::ABD => (VERTICES[3] - VERTICES[0], VERTICES[1] - VERTICES[0]),
            TetrahedronFace::BCD => (VERTICES[3] - VERTICES[1], VERTICES[2] - VERTICES[1]),
            TetrahedronFace::CBA => (VERTICES[0] - VERTICES[2], VERTICES[1] - VERTICES[2]),
            TetrahedronFace::DCA => (VERTICES[0] - VERTICES[3], VERTICES[2] - VERTICES[3]),
        };
        (vec_1.normalize(), vec_2.normalize())
    }

    fn is_disconnected_from(&self, other: &TetrahedronFace) -> bool {
        false
    }
}

impl HasFace for TetrahedronFace {
    fn normal(&self) -> Vec3 {
        let (vec_1, vec_2) = self.defining_vectors();

        vec_1.cross(vec_2).normalize()
    }

    fn border_type(&self, other: &TetrahedronFace) -> Option<BorderType> {
        let border_type = if self == other {
            BorderType::SameFace
        } else {
            BorderType::Connected
        };
        Some(border_type)
    }
}

#[derive(Debug, Clone, Copy)]
pub struct TetrahedronRoom {
    pub position: Vec3,
    pub face_position: (u8, u8),
    pub face: TetrahedronFace,
}

impl IsRoom<TetrahedronFace> for TetrahedronRoom {
    fn position(&self) -> Vec3 {
        self.position
    }

    fn face(&self) -> TetrahedronFace {
        self.face
    }
}

impl Ord for TetrahedronRoom {
    fn cmp(&self, other: &TetrahedronRoom) -> Ordering {
        match self.face.cmp(&other.face) {
            Ordering::Equal => self.face_position.cmp(&other.face_position),
            ordering => ordering,
        }
    }
}

impl PartialOrd for TetrahedronRoom {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl Hash for TetrahedronRoom {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.face_position.hash(state);
        self.face.hash(state);
    }
}

impl PartialEq for TetrahedronRoom {
    fn eq(&self, other: &Self) -> bool {
        self.position.distance(other.position) < 0.01
    }
}

impl Eq for TetrahedronRoom {}

#[derive(Resource, Clone)]
pub struct Tetrahedron {
    nodes_per_edge: u8,
    distance_between_nodes: f32,
    face_size: f32,
    tetrahedron: BevyTetrahedron,
}

impl Tetrahedron {
    pub fn new(nodes_per_edge: u8, face_size: f32) -> Self {
        let distance_between_nodes = face_size / (nodes_per_edge as f32 - 1.0 + 3.0_f32.sqrt());

        let face_size_ratio = face_size / SQRT_2;
        let tetrahedron = BevyTetrahedron::new(
            VERTICES[0] * face_size_ratio,
            VERTICES[1] * face_size_ratio,
            VERTICES[2] * face_size_ratio,
            VERTICES[3] * face_size_ratio,
        );

        Self {
            nodes_per_edge,
            distance_between_nodes,
            face_size,
            tetrahedron,
        }
    }
}

impl PlatonicSolid for Tetrahedron {
    type Face = TetrahedronFace;
    type Room = TetrahedronRoom;

    fn make_nodes_from_face(&self, face: TetrahedronFace) -> Vec<TetrahedronRoom> {
        let (vec_i, vec_j) = face.defining_vectors();
        let normal = face.normal();

        let nodes_per_edge_float = self.nodes_per_edge as f32;
        let face_height_from_origin = 3.0_f32.sqrt() * self.face_size / 6.0 / SQRT_2;

        let max_abs_face_coord = (nodes_per_edge_float - 1.0) / 3.0;

        iproduct!(0..self.nodes_per_edge, 0..self.nodes_per_edge)
            .filter(|(i, j)| i + j <= self.nodes_per_edge - 1)
            .map(|(i, j)| {
                let face_x = i as f32;
                let face_y = j as f32;

                let face_coord_x = (face_x - max_abs_face_coord) * vec_i;
                let face_coord_y = (face_y - max_abs_face_coord) * vec_j;

                let face_coord = (face_coord_x + face_coord_y) * self.distance_between_nodes
                    + normal * face_height_from_origin;
                let position = face_coord;

                TetrahedronRoom {
                    position,
                    face_position: (i, j),
                    face: face.clone(),
                }
            })
            .collect::<Vec<TetrahedronRoom>>()
    }

    fn generate_traversal_graph(
        &self,
        nodes: Vec<TetrahedronRoom>,
    ) -> TraversalGraph<TetrahedronRoom, Edge> {
        let traversal_graph_generator = TetrahedronTraversalGraphGenerator {
            distance_between_nodes: self.distance_between_nodes,
        };

        let traversal_graph = traversal_graph_generator.generate(nodes.clone());

        println!(
            "Produced traversal graph with {:?} edges",
            traversal_graph.all_edges().count()
        );

        traversal_graph
    }

    fn get_mesh_builder(&self) -> PlatonicMeshBuilder {
        PlatonicMeshBuilder::new(
            self.distance_between_nodes,
            (1.0_f32 / 3.0).acos(),
            self.tetrahedron.into(),
        )
    }
}

struct TetrahedronTraversalGraphGenerator {
    pub distance_between_nodes: f32,
}

impl TraversalGraphGenerator<TetrahedronRoom, Edge> for TetrahedronTraversalGraphGenerator {
    fn can_connect(&self, from: &TetrahedronRoom, to: &TetrahedronRoom) -> bool {
        let distance = from.position.distance(to.position);

        match from.face.border_type(&to.face) {
            Some(BorderType::SameFace) => distance - 0.1 <= self.distance_between_nodes,
            Some(BorderType::Connected) => {
                let connected_edge_factor = 1.0 / 3.0_f32.sqrt();
                distance - 0.1 <= self.distance_between_nodes * connected_edge_factor
            }
            _ => false,
        }
    }
}
