use std::{
    fmt::Debug,
    hash::{Hash, Hasher},
};

use bevy::{ecs::system::Resource, math::Vec3};
use maze_generator::{
    config::Maze,
    model::{Door, TraversalGraph},
};
use strum::IntoEnumIterator;

use super::platonic_mesh_builder::PlatonicMeshBuilder;

#[derive(Debug, Eq, PartialEq)]
pub enum BorderType {
    SameFace,
    Connected,
}

pub trait HasFace: IntoEnumIterator {
    fn normal(&self) -> Vec3;
    fn border_type(&self, other: &Self) -> Option<BorderType>;
}

pub trait IsRoom<F: HasFace> {
    fn position(&self) -> Vec3;
    fn face(&self) -> F;
}

#[derive(Debug, Clone, Hash, Eq, PartialEq, PartialOrd, Default)]
pub struct Edge;

impl<R> Door<R> for Edge {
    fn is_directed(&self) -> bool {
        false
    }

    fn door_path_weight(&self) -> u16 {
        1
    }

    fn get_all_doors() -> Vec<Self> {
        vec![Edge]
    }
}

pub trait PlatonicSolid: Resource + Sized {
    type Face: HasFace;
    type Room: Debug
        + Clone
        + Copy
        + Hash
        + Eq
        + Ord
        + PartialOrd
        + Send
        + Sync
        + IsRoom<Self::Face>;

    fn make_nodes_from_face(&self, face: Self::Face) -> Vec<Self::Room>;

    fn generate_traversal_graph(&self, nodes: Vec<Self::Room>) -> TraversalGraph<Self::Room, Edge>;

    fn build_maze(&self) -> Maze<Self::Room, Edge> {
        let nodes = self.make_nodes();
        let traversal_graph = self.generate_traversal_graph(nodes.clone());
        Maze::build(traversal_graph)
    }

    fn make_nodes(&self) -> Vec<Self::Room> {
        Self::Face::iter()
            .flat_map(|face| self.make_nodes_from_face(face))
            .collect()
    }

    fn get_mesh_builder(&self) -> PlatonicMeshBuilder;
}
