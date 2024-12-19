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
    let white = Color::srgb_u8(247, 247, 0);
    let beige = Color::srgb_u8(242, 231, 213);
    let green = Color::srgb_u8(109, 152, 134);
    let charcoal = Color::srgb_u8(57, 62, 70);

    let white_material = materials.add(StandardMaterial::from_color(white));
    let red_material = materials.add(StandardMaterial::from_color(Color::srgb_u8(0, 130, 140)));
    let beige_material = materials.add(StandardMaterial::from_color(beige));
    let green_material = materials.add(StandardMaterial::from_color(green));
    let charcoal_material = materials.add(StandardMaterial::from_color(charcoal));

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

        let mesh_handle = match BorderType::from_faces(&source_node.face, &target_node.face) {
            BorderType::SameFace if bidirectional => face_connection_mesh.clone(),
            BorderType::SameFace if !bidirectional => face_arrow_mesh.clone(),
            BorderType::Connected if bidirectional => edge_mesh.clone(),
            BorderType::Connected if !bidirectional => edge_arrow_mesh.clone(),
            _ => panic!["unknown edge types"],
        };

        let old_transform = get_connection_transform(source_node, target_node);

        commands.spawn(PbrBundle {
            mesh: Mesh3d(mesh_handle),
            material: MeshMaterial3d(beige_material.clone()),
            transform: old_transform,
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

fn get_connection_transform(from: CubeNode, to: CubeNode) -> Transform {
    match BorderType::from_faces(&from.face, &to.face) {
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
        _ => panic!["unknown edge types"],
    }
}
