use bevy::prelude::*;
use bevy::render::mesh::Indices;
use bevy::{asset::RenderAssetUsages, render::mesh::PrimitiveTopology};

use itertools::repeat_n;

pub trait FaceMeshGenerator<const NUM_VERTICES_PER_FACE: usize> {
    fn load_mesh_asset<const NUM_FACES: usize>(
        mut mesh_assets: &mut Assets<Mesh>,
        vertices: [[Vec3; NUM_VERTICES_PER_FACE]; NUM_FACES],
    ) -> [Handle<Mesh>; NUM_FACES] {
        vertices
            .map(|face_vertices| Self::get_face_mesh(face_vertices))
            .map(|mesh| mesh_assets.add(mesh))
    }

    fn get_face_mesh(face_vertices: [Vec3; NUM_VERTICES_PER_FACE]) -> Mesh;
}

pub struct TriangleFaceMeshGenerator;

impl FaceMeshGenerator<3> for TriangleFaceMeshGenerator {
    fn get_face_mesh(face_vertices: [Vec3; 3]) -> Mesh {
        let uvs = vec![[0.0_f32, 0.0], [1.0, 0.0], [0.0, 1.0]];
        let defining_vector_1 = face_vertices[1] - face_vertices[0];
        let defining_vector_2 = face_vertices[2] - face_vertices[0];
        let normal = defining_vector_1.cross(defining_vector_2).normalize();
        let normals = repeat_n(normal.to_array(), 3).collect::<Vec<[f32; 3]>>();

        let face_indices = vec![0_u16, 1, 2];

        Mesh::new(
            PrimitiveTopology::TriangleList,
            RenderAssetUsages::RENDER_WORLD,
        )
        .with_inserted_attribute(
            Mesh::ATTRIBUTE_POSITION,
            face_vertices.into_iter().collect::<Vec<Vec3>>(),
        )
        .with_inserted_attribute(Mesh::ATTRIBUTE_UV_0, uvs)
        .with_inserted_attribute(Mesh::ATTRIBUTE_NORMAL, normals)
        .with_inserted_indices(Indices::U16(face_indices))
    }
}

pub struct SquareFaceMeshGenerator;

impl FaceMeshGenerator<4> for SquareFaceMeshGenerator {
    fn get_face_mesh(face_vertices: [Vec3; 4]) -> Mesh {
        // let scaling_factor = 0.5;

        let uvs = vec![[0.0_f32, 0.0], [1.0, 0.0], [1.0, 1.0], [0.0, 1.0]];

        let defining_vector_1 = face_vertices[1] - face_vertices[0];
        let defining_vector_2 = face_vertices[2] - face_vertices[0];
        let normal = defining_vector_1.cross(defining_vector_2).normalize();
        let normals = repeat_n(normal.to_array(), 4).collect::<Vec<[f32; 3]>>();

        let face_indices = vec![0_u16, 1, 2, 0, 2, 3];

        Mesh::new(
            PrimitiveTopology::TriangleList,
            RenderAssetUsages::RENDER_WORLD,
        )
        .with_inserted_attribute(
            Mesh::ATTRIBUTE_POSITION,
            face_vertices.into_iter().collect::<Vec<Vec3>>(),
        )
        .with_inserted_attribute(Mesh::ATTRIBUTE_UV_0, uvs)
        .with_inserted_attribute(Mesh::ATTRIBUTE_NORMAL, normals)
        .with_inserted_indices(Indices::U16(face_indices))
    }
}

pub struct PentagonFaceMeshGenerator;

impl FaceMeshGenerator<5> for PentagonFaceMeshGenerator {
    fn get_face_mesh(vertices: [Vec3; 5]) -> Mesh {
        //let scaling_factor = PHI / 2.0;
        let uvs = vec![
            [0.0_f32, 0.0],
            [1.0, 0.0],
            [0.0, 1.0],
            [0.5, 1.0],
            [0.0, 0.5],
        ];

        let defining_vector_1 = vertices[1] - vertices[0];
        let defining_vector_2 = vertices[2] - vertices[0];
        let normal = defining_vector_1.cross(defining_vector_2).normalize();
        let normals = repeat_n(normal.to_array(), 5).collect::<Vec<[f32; 3]>>();

        let face_indices = vec![0_u16, 1, 2, 0, 2, 3, 0, 3, 4];

        Mesh::new(
            PrimitiveTopology::TriangleList,
            RenderAssetUsages::RENDER_WORLD,
        )
        .with_inserted_attribute(
            Mesh::ATTRIBUTE_POSITION,
            vertices.into_iter().collect::<Vec<Vec3>>(),
        )
        .with_inserted_attribute(Mesh::ATTRIBUTE_UV_0, uvs)
        .with_inserted_attribute(Mesh::ATTRIBUTE_NORMAL, normals)
        .with_inserted_indices(Indices::U16(face_indices))
    }
}
