use bevy::{
    asset::Assets,
    color::Color,
    ecs::system::{Commands, ResMut},
    math::NormedVectorSpace,
    pbr::{PbrBundle, StandardMaterial},
    prelude::*,
    render::mesh::Mesh,
    transform::components::Transform,
};

use std::f32::consts::{FRAC_PI_2, PI};

use bevy::{
    prelude::*,
    render::{
        mesh::{Indices, PrimitiveTopology},
        render_asset::RenderAssetUsages,
    },
};
use petgraph::Direction;

use crate::Level;

use itertools::Itertools;

use super::platonic_solid::{BorderType, HasFace, IsRoom, PlatonicSolid};

pub fn spawn_shape_meshes<P: PlatonicSolid>(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    level: Res<Level<P>>,
) {
    let cyan = Color::srgb_u8(247, 247, 0);
    let beige = Color::srgb_u8(242, 231, 213);
    let green = Color::srgb_u8(109, 152, 134);

    let cyan_material = materials.add(StandardMaterial::from_color(cyan));
    let beige_material = materials.add(StandardMaterial::from_color(beige));
    let green_material = materials.add(StandardMaterial::from_color(green));

    let goal_node = level.maze.solution.last().unwrap();
    for node in level.maze.graph.nodes().filter(|node| {
        let incoming_neighbors = level
            .maze
            .graph
            .neighbors_directed(*node, Direction::Incoming);
        let outgoing_neighbors = level
            .maze
            .graph
            .neighbors_directed(*node, Direction::Outgoing);

        let neighbors = incoming_neighbors
            .chain(outgoing_neighbors)
            .unique()
            .collect::<Vec<P::Room>>();

        neighbors.len() != 2 || {
            let first_neighbor = neighbors[0];
            let second_neighbor = neighbors[1];

            let node_to_first_vec = node.position() - first_neighbor.position();
            let node_to_second_vec = node.position() - second_neighbor.position();

            node_to_first_vec.dot(node_to_second_vec).abs() < 0.1
        }
    }) {
        let material_handle = if node == *goal_node {
            cyan_material.clone()
        } else {
            beige_material.clone()
        };

        let transform = Transform::IDENTITY
            .looking_at(
                -node.face().normal(),
                node.face().normal().any_orthogonal_vector(),
            )
            .with_translation(node.position() + node.face().normal() * 0.002);

        let radius = if node == *goal_node { 0.1 } else { 0.06 };

        commands.spawn(PbrBundle {
            mesh: Mesh3d(meshes.add(Circle::new(radius))),
            material: MeshMaterial3d(material_handle),
            transform,
            ..default()
        });
    }

    let face_angle = FRAC_PI_2;
    let edge_mesh_builder =
        EdgeMeshBuilder::new(level.platonic_solid.distance_between_nodes(), face_angle);

    let face_connection_mesh = meshes.add(edge_mesh_builder.line());
    let face_arrow_mesh = meshes.add(edge_mesh_builder.dashed_arrow());

    let edge_mesh = meshes.add(edge_mesh_builder.edge_line());
    let edge_arrow_mesh = meshes.add(edge_mesh_builder.dashed_arrow_edge());

    for (source_node, target_node, _) in level.maze.graph.all_edges() {
        let bidirectional = level.maze.graph.contains_edge(target_node, source_node);

        if bidirectional && source_node.cmp(&target_node).is_lt() {
            continue;
        }

        let Some(border_type) = source_node.face().border_type(&target_node.face()) else {
            panic!["unknown edge type"];
        };

        let mesh_handle = match (&border_type, bidirectional) {
            (BorderType::SameFace, true) => face_connection_mesh.clone(),
            (BorderType::SameFace, false) => face_arrow_mesh.clone(),
            (BorderType::Connected, true) => edge_mesh.clone(),
            (BorderType::Connected, false) => edge_arrow_mesh.clone(),
        };

        let transform = get_connection_transform::<P>(source_node, target_node, &border_type);

        commands.spawn(PbrBundle {
            mesh: Mesh3d(mesh_handle),
            material: MeshMaterial3d(beige_material.clone()),
            transform,
            ..default()
        });
    }

    let cuboid = meshes.add(Cuboid::from_length(1.5));
    commands.spawn(PbrBundle {
        mesh: Mesh3d(cuboid),
        material: MeshMaterial3d(green_material.clone()),
        transform: Transform::IDENTITY,
        ..default()
    });
}

fn get_connection_transform<P: PlatonicSolid>(
    from: P::Room,
    to: P::Room,
    border_type: &BorderType,
) -> Transform {
    match border_type {
        BorderType::SameFace => {
            let forward = from.position() - to.position();
            Transform::IDENTITY
                .looking_to(forward, from.face().normal())
                .with_translation(from.position() + from.face().normal() * 0.001)
        }
        BorderType::Connected => {
            let from_normal = from.face().normal();
            let to_normal = to.face().normal();

            let half_angle = from_normal.angle_between(to_normal) / 2.0;

            let average_normal = from_normal.lerp(to_normal, 0.5).normalize();

            let edge_vec = to.position() - from.position();

            let intersection_point = from.position()
                + (edge_vec + edge_vec.norm() * half_angle.tan() * average_normal) / 2.0;

            Transform::IDENTITY
                .looking_to(intersection_point - to.position(), to.face().normal())
                .with_translation(intersection_point + average_normal * 0.001)
        }
    }
}

struct EdgeMeshBuilder {
    dash_width: f32,
    dash_length: f32,
    min_spacing: f32,
    arrow_head_width: f32,
    face_angle: f32,
    distance_between_nodes: f32,
    empty_mesh: Mesh,
}

impl EdgeMeshBuilder {
    pub fn new(distance_between_nodes: f32, face_angle: f32) -> Self {
        let empty_mesh = Mesh::new(
            PrimitiveTopology::TriangleList,
            RenderAssetUsages::default(),
        )
        .with_inserted_attribute(Mesh::ATTRIBUTE_POSITION, Vec::<Vec3>::new())
        .with_inserted_attribute(Mesh::ATTRIBUTE_UV_0, Vec::<Vec2>::new())
        .with_inserted_attribute(Mesh::ATTRIBUTE_NORMAL, Vec::<Vec3>::new())
        .with_inserted_indices(Indices::U32(vec![]));

        EdgeMeshBuilder {
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
