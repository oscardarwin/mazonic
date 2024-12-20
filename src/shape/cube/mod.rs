pub mod maze;
mod mesh;

use std::f32::consts::FRAC_PI_2;

use bevy::{
    asset::Assets,
    color::Color,
    ecs::system::{Commands, ResMut},
    math::{primitives::Cylinder, vec2, NormedVectorSpace},
    pbr::{PbrBundle, StandardMaterial},
    prelude::*,
    render::mesh::Mesh,
    transform::components::Transform,
};
use bevy_vector_shapes::{
    painter::{ShapeCommands, ShapeConfig, ShapeSpawner},
    render::ShapePipelineType,
    shapes::{LineBundle, LineComponent, LineSpawner, ShapeAlphaMode, ShapeBundle, TriangleBundle},
};

use self::{
    maze::{BorderType, CubeMaze, CubeNode},
    mesh::EdgeMeshBuilder,
};

pub fn spawn(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    cube_maze: Res<CubeMaze>,
    mut shape_commands: ShapeCommands,
) {
    let cyan = Color::srgb_u8(247, 247, 0);
    let beige = Color::srgb_u8(242, 231, 213);
    let green = Color::srgb_u8(109, 152, 134);

    let cyan_material = materials.add(StandardMaterial::from_color(cyan));
    let beige_material = materials.add(StandardMaterial::from_color(beige));
    let green_material = materials.add(StandardMaterial::from_color(green));

    let face_angle = FRAC_PI_2;
    let edge_mesh_builder = EdgeMeshBuilder::new();

    let face_connection_mesh = meshes.add(edge_mesh_builder.line(cube_maze.distance_between_nodes));
    let face_arrow_mesh =
        meshes.add(edge_mesh_builder.dashed_arrow(cube_maze.distance_between_nodes));

    let edge_mesh =
        meshes.add(edge_mesh_builder.edge_line(cube_maze.distance_between_nodes / 2.0, face_angle));
    let edge_arrow_mesh = meshes.add(
        edge_mesh_builder.dashed_arrow_edge(cube_maze.distance_between_nodes / 2.0, face_angle),
    );

    for (source_node, target_node, _) in cube_maze.maze.graph.all_edges() {
        let bidirectional = cube_maze.maze.graph.contains_edge(target_node, source_node);

        if bidirectional && source_node.cmp(&target_node).is_lt() {
            continue;
        }

        let Some(border_type) = BorderType::from_faces(&source_node.face, &target_node.face) else {
            panic!["unknown edge type"];
        };

        let mesh_handle = match (&border_type, bidirectional) {
            (BorderType::SameFace, true) => face_connection_mesh.clone(),
            (BorderType::SameFace, false) => face_arrow_mesh.clone(),
            (BorderType::Connected, true) => edge_mesh.clone(),
            (BorderType::Connected, false) => edge_arrow_mesh.clone(),
        };

        let transform = get_connection_transform(source_node, target_node, &border_type);

        commands.spawn(PbrBundle {
            mesh: Mesh3d(mesh_handle),
            material: MeshMaterial3d(beige_material.clone()),
            transform,
            ..default()
        });
    }

    let last_node = cube_maze.maze.solution.last().unwrap();
    commands.spawn(PbrBundle {
        mesh: Mesh3d(meshes.add(Circle::new(0.1))),
        material: MeshMaterial3d(cyan_material.clone()),
        transform: Transform::IDENTITY
            .looking_at(
                -last_node.face.normal(),
                last_node.face.normal().any_orthogonal_vector(),
            )
            .with_translation(last_node.position + last_node.face.normal() * 0.002),
        ..default()
    });

    let cuboid = meshes.add(Cuboid::from_length(1.5));
    commands.spawn(PbrBundle {
        mesh: Mesh3d(cuboid),
        material: MeshMaterial3d(green_material.clone()),
        transform: Transform::IDENTITY,
        ..default()
    });
}

fn get_connection_transform(from: CubeNode, to: CubeNode, border_type: &BorderType) -> Transform {
    match border_type {
        BorderType::SameFace => {
            let forward = from.position - to.position;
            Transform::IDENTITY
                .looking_to(forward, from.face.normal())
                .with_translation(from.position + from.face.normal() * 0.001)
        }
        BorderType::Connected => {
            let from_normal = from.face.normal();
            let to_normal = to.face.normal();

            let half_angle = from_normal.angle_between(to_normal) / 2.0;

            let average_normal = from_normal.lerp(to_normal, 0.5).normalize();

            let edge_vec = to.position - from.position;

            let intersection_point = from.position
                + (edge_vec + edge_vec.norm() * half_angle.tan() * average_normal) / 2.0;

            Transform::IDENTITY
                .looking_to(intersection_point - to.position, to.face.normal())
                .with_translation(intersection_point + average_normal * 0.001)
        }
    }
}
