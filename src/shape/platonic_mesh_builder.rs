use bevy::{
    prelude::*,
    render::mesh::Mesh,
};

use std::f32::consts::{FRAC_PI_2, PI};

use bevy::render::{
        mesh::{Indices, PrimitiveTopology},
        render_asset::RenderAssetUsages,
    };


use itertools::Itertools;

pub struct PlatonicMeshBuilder {
    dash_width: f32,
    dash_length: f32,
    min_spacing: f32,
    arrow_head_width: f32,
    face_angle: f32,
    distance_between_nodes: f32,
    empty_mesh: Mesh,
}

impl PlatonicMeshBuilder {
    pub fn new(distance_between_nodes: f32, face_angle: f32) -> Self {
        let empty_mesh = Mesh::new(
            PrimitiveTopology::TriangleList,
            RenderAssetUsages::default(),
        )
        .with_inserted_attribute(Mesh::ATTRIBUTE_POSITION, Vec::<Vec3>::new())
        .with_inserted_attribute(Mesh::ATTRIBUTE_UV_0, Vec::<Vec2>::new())
        .with_inserted_attribute(Mesh::ATTRIBUTE_NORMAL, Vec::<Vec3>::new())
        .with_inserted_indices(Indices::U32(vec![]));

        PlatonicMeshBuilder {
            dash_width: 0.06,
            dash_length: 0.09,
            min_spacing: 0.07,
            arrow_head_width: 0.12,
            face_angle,
            distance_between_nodes,
            empty_mesh,
        }
    }

    pub fn line(&self) -> Mesh {
        self.make_line(self.distance_between_nodes)
    }

    fn make_line(&self, length: f32) -> Mesh {
        Rectangle::new(self.dash_width, length)
            .mesh()
            .build()
            .rotated_by(Quat::from_rotation_x(-FRAC_PI_2))
            .translated_by(Vec3::Z * length / 2.0)
    }

    fn make_dashed_line(&self, length: f32) -> Mesh {
        let total_min_segment_length = self.dash_length + self.min_spacing;

        let num_dashes = (length / total_min_segment_length).floor();

        let dash_and_space_length = length / num_dashes;
        let dash_and_space_half_length = dash_and_space_length / 2.0;

        let mut mesh = self.empty_mesh.clone();
        let line_direction = Vec3::Z;
        for dash_index in 0..num_dashes as u8 {
            let z_coord = dash_and_space_half_length + dash_index as f32 * dash_and_space_length;
            let position = z_coord * line_direction;
            let dash_mesh = self.make_dash(position);

            mesh.merge(&dash_mesh);
        }
        mesh
    }

    pub fn dashed_arrow(&self) -> Mesh {
        self.make_dashed_arrow(self.distance_between_nodes)
    }

    fn make_dashed_arrow(&self, length: f32) -> Mesh {
        let total_min_segment_length = self.dash_length + self.min_spacing;

        let num_dashes = (length / total_min_segment_length).floor();

        let dash_and_space_length = length / num_dashes;
        let dash_and_space_half_length = dash_and_space_length / 2.0;

        let mut mesh = self.empty_mesh.clone();
        for dash_index in 0..(num_dashes as u8 - 1) {
            let z_coord = dash_and_space_half_length + dash_index as f32 * dash_and_space_length;
            let position = z_coord * Vec3::Z;
            let dash_mesh = self.make_dash(position);

            mesh.merge(&dash_mesh);
        }

        let arrow_z_coord =
            dash_and_space_half_length + (num_dashes as f32 - 1.0) * dash_and_space_length;
        let position = Vec3::Z * arrow_z_coord;
        mesh.merge(&self.make_arrow(position));

        mesh
    }

    pub fn dashed_arrow_edge(&self) -> Mesh {
        let half_length = self.distance_between_nodes / 2.0;

        let mut first_dashed_line = self.make_dashed_arrow(half_length);
        let second_dashed_line = self
            .make_dashed_line(half_length)
            .rotated_by(Quat::from_rotation_z(PI))
            .rotated_by(Quat::from_rotation_x(PI - self.face_angle));

        first_dashed_line.merge(&second_dashed_line);
        first_dashed_line
    }

    pub fn edge_line(&self) -> Mesh {
        let half_length = self.distance_between_nodes / 2.0;

        let mut first_line = self.make_line(half_length);
        let second_line = self
            .make_line(half_length)
            .rotated_by(Quat::from_rotation_z(PI))
            .rotated_by(Quat::from_rotation_x(PI - self.face_angle));

        first_line.merge(&second_line);
        first_line
    }

    fn make_dash(&self, position: Vec3) -> Mesh {
        Rectangle::new(self.dash_width, self.dash_length)
            .mesh()
            .build()
            .rotated_by(Quat::from_rotation_x(-FRAC_PI_2))
            .translated_by(position)
    }

    fn make_arrow(&self, position: Vec3) -> Mesh {
        let arrow_side_vertex = Vec3::new(self.arrow_head_width / 2.0, 0.0, 0.0);
        let arrow_tip_vertex = Vec3::new(0.0, 0.0, self.dash_length / 2.0);

        let mut arrow = Triangle3d::new(arrow_tip_vertex, arrow_side_vertex, -arrow_side_vertex)
            .mesh()
            .build()
            .translated_by(position);

        let arrow_base = Rectangle::new(self.dash_width, self.dash_length / 2.0)
            .mesh()
            .build()
            .rotated_by(Quat::from_rotation_x(-FRAC_PI_2))
            .translated_by(position - Vec3::Z * self.dash_length / 4.0);

        arrow.merge(&arrow_base);
        arrow
    }
}
