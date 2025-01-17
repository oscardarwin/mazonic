use std::{fmt::Debug, hash::Hash};

use bevy::{ecs::system::Resource, math::Vec3, render::mesh::Mesh, utils::HashSet};
use serde::{Deserialize, Serialize};

use crate::room::{Face, SolidRoom};

#[derive(Debug, Eq, PartialEq)]
pub enum BorderType {
    SameFace,
    Connected,
}

#[derive(Debug, Clone, Hash, Eq, PartialEq, PartialOrd, Default, Serialize, Deserialize)]
pub struct Edge;

pub trait ShapeMeshLoader<
    const NUM_VERTICES: usize,
    const NUM_FACES: usize,
    const NUM_VERTICES_PER_FACE: usize,
>: Resource + Sized + Clone + Debug
{
    const VERTICES: [[f32; 3]; NUM_VERTICES];
    const FACES: [[usize; NUM_VERTICES_PER_FACE]; NUM_FACES];

    fn vertices(vertex_indices: &[usize; NUM_VERTICES_PER_FACE]) -> [Vec3; NUM_VERTICES_PER_FACE] {
        vertex_indices.map(|index| Vec3::from_array(Self::VERTICES[index]))
    }

    fn make_nodes_from_face(&self, face: &Face) -> Vec<SolidRoom>;

    fn border_type(from: &Face, to: &Face) -> Option<BorderType> {
        let from_vertex_set = Self::FACES[from.id()]
            .into_iter()
            .collect::<HashSet<usize>>();
        let to_vertex_set = Self::FACES[to.id()].into_iter().collect::<HashSet<usize>>();

        match from_vertex_set.intersection(&to_vertex_set).count() {
            0 | 1 => None,
            2 => Some(BorderType::Connected),
            _ => Some(BorderType::SameFace),
        }
    }

    fn generate_nodes(&self) -> Vec<SolidRoom> {
        Self::FACES
            .into_iter()
            .enumerate()
            .map(|(id, face_indices)| {
                let normal = Self::face_normal(&face_indices);

                Face { id, normal }
            })
            .flat_map(|face| self.make_nodes_from_face(&face))
            .collect()
    }

    fn get_face_meshes(&self) -> Vec<Mesh> {
        Self::FACES
            .map(|vertex_indices| Self::vertices(&vertex_indices))
            .map(|vertices| self.get_face_mesh(vertices))
            .into_iter()
            .collect::<Vec<Mesh>>()
    }

    fn face_normal(face_indices: &[usize; NUM_VERTICES_PER_FACE]) -> Vec3 {
        let vertices = Self::vertices(face_indices);
        let vec_1 = vertices[1] - vertices[0];
        let vec_2 = vertices[2] - vertices[0];
        vec_1.cross(vec_2).normalize()
    }

    fn get_face_mesh(&self, face_vertices: [Vec3; NUM_VERTICES_PER_FACE]) -> Mesh;
}
