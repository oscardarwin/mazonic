use bevy::prelude::*;

pub fn face_indices_to_vertices<const NUM_FACES: usize, const VERTICES_PER_FACE: usize>(
    faces_indices: [[usize; VERTICES_PER_FACE]; NUM_FACES],
    vertices: &[Vec3],
) -> [[Vec3; VERTICES_PER_FACE]; NUM_FACES] {
    faces_indices.map(|face_indices| face_indices.map(|index| vertices[index]))
}

pub fn compute_face_normal<const VERTICES_PER_FACE: usize>(
    face: &[Vec3; VERTICES_PER_FACE],
) -> Vec3 {
    let vec_1 = face[1] - face[0];
    let vec_2 = face[2] - face[0];
    vec_1.cross(vec_2).normalize()
}
