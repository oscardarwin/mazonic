use bevy::{
    prelude::*,
    render::{mesh::Mesh, render_resource::encase::rts_array::Length},
};

use std::f32::consts::{FRAC_PI_2, PI};

use bevy::render::{
    mesh::{Indices, PrimitiveTopology},
    render_asset::RenderAssetUsages,
};

use crate::constants::SQRT_3;

#[derive(Component)]
pub struct MazeMeshBuilder {
    dash_width: f32,
    arrow_head_length: f32,
    arrow_head_width: f32,
    face_angle: f32,
    distance_between_nodes: f32,
}

impl MazeMeshBuilder {
    pub fn new(face_angle: f32) -> Self {
        let dash_width = 0.16;
        let arrow_head_width = dash_width * 1.7;
        let arrow_head_length = 0.2;

        MazeMeshBuilder {
            dash_width,
            arrow_head_length,
            arrow_head_width,
            face_angle,
            distance_between_nodes: 1.0,
        }
    }

    pub fn tetrahedron() -> Self {
        Self::new((1.0_f32 / 3.0).acos())
    }

    pub fn cube() -> Self {
        Self::new(FRAC_PI_2)
    }

    pub fn octahedron() -> Self {
        Self::new((-1.0_f32 / 3.0).acos())
    }

    pub fn dodecahedron() -> Self {
        Self::new((-5.0_f32.sqrt() / 5.0).acos())
    }

    pub fn icosahedron() -> Self {
        Self::new((-5.0_f32.sqrt() / 3.0).acos())
    }

    pub fn level_selector() -> Self {
        Self {
            dash_width: 0.05,
            arrow_head_length: 0.1,
            arrow_head_width: 0.1,
            face_angle: (-5.0_f32.sqrt() / 3.0).acos(),
            distance_between_nodes: 1.0 / SQRT_3 / 3.0,
        }
    }

    fn line(&self, length: f32, uv_start: f32, uv_end: f32) -> Mesh {
        let mut line = Rectangle::new(self.dash_width, length)
            .mesh()
            .build()
            .rotated_by(Quat::from_rotation_x(-FRAC_PI_2))
            .translated_by(Vec3::Z * length / 2.0);

        line.insert_attribute(
            Mesh::ATTRIBUTE_UV_0,
            vec![
                [0.0, uv_start],
                [1.0, uv_start],
                [1.0, uv_end],
                [0.0, uv_end],
            ],
        );

        line
    }

    fn arrow_head(&self) -> Mesh {
        let arrow_side_vertex = Vec3::new(self.arrow_head_width / 2.0, 0.0, 0.0);
        let arrow_tip_vertex = Vec3::new(0.0, 0.0, self.arrow_head_length / 2.0);

        let mut mesh = Triangle3d::new(arrow_tip_vertex, arrow_side_vertex, -arrow_side_vertex)
            .mesh()
            .build();

        mesh.insert_attribute(
            Mesh::ATTRIBUTE_UV_0,
            vec![[0.0, 0.5], [1.0, 0.5], [1.0, 1.0]],
        );

        mesh
    }

    pub fn same_face_edge(&self) -> Mesh {
        self.line(self.distance_between_nodes, 0., 0.5)
    }

    pub fn one_way_same_face_edge(&self) -> Mesh {
        let rectangle_section_length =
            0.9 * self.distance_between_nodes - self.arrow_head_length * 0.5;
        let mut rectangle_mesh = self.line(rectangle_section_length, 0.0, 0.5);

        let arrow_head_mesh = self
            .arrow_head()
            .translated_by(Vec3::Z * rectangle_section_length);

        rectangle_mesh.merge(&arrow_head_mesh);
        rectangle_mesh
    }

    pub fn one_way_cross_face_edge(&self) -> Mesh {
        let first_length = self.distance_between_nodes / 2.0;
        let second_length = 0.4 * self.distance_between_nodes - self.arrow_head_length * 0.5;
        let uv_mid_point = 0.5 * first_length / (first_length + second_length);

        let mut second_line = self.line(second_length, uv_mid_point, 0.5);

        let arrow_head_mesh = self.arrow_head().translated_by(Vec3::Z * second_length);
        second_line.merge(&arrow_head_mesh);

        let first_line = self
            .line(first_length, uv_mid_point, 0.0)
            .rotated_by(Quat::from_rotation_z(PI))
            .rotated_by(Quat::from_rotation_x(self.face_angle));

        second_line.merge(&first_line);
        second_line
    }

    pub fn cross_face_edge(&self) -> Mesh {
        let half_length = self.distance_between_nodes / 2.0;

        let mut first_line = self.line(half_length, 0.0, 0.5);
        let second_line = self
            .line(half_length, 0.0, 0.5)
            .rotated_by(Quat::from_rotation_z(PI))
            .rotated_by(Quat::from_rotation_x(self.face_angle));

        first_line.merge(&second_line);
        first_line
    }
}
