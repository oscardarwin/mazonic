pub mod maze;
mod mesh;

use bevy::{
    asset::Assets,
    color::Color,
    ecs::system::{Commands, ResMut},
    math::primitives::Cylinder,
    pbr::{PbrBundle, StandardMaterial},
    prelude::*,
    render::mesh::Mesh,
    transform::components::Transform,
};

use self::{
    maze::{BorderType, CubeMaze, CubeNode},
    mesh::get_connection_mesh,
};

pub fn spawn(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    cube_maze: Res<CubeMaze>,
) {
    let white = Color::srgb_u8(247, 247, 0);
    let beige = Color::srgb_u8(242, 231, 213);
    let green = Color::srgb_u8(109, 152, 134);
    let charcoal = Color::srgb_u8(57, 62, 70);

    let white_material = materials.add(StandardMaterial::from_color(white));
    let beige_material = materials.add(StandardMaterial::from_color(beige));
    let green_material = materials.add(StandardMaterial::from_color(green));
    let charcoal_material = materials.add(StandardMaterial::from_color(charcoal));

    let cube_gen = CubeMaze::build(3, 2.0, 0.2);
    let connection_height = 0.04;

    for node in cube_maze.maze.graph.nodes() {
        let cylinder = Cylinder::new(0.09, connection_height + 0.005);

        let cylinder_mesh = meshes.add(cylinder);

        let face_normal = node.face.normal();

        let transform = Transform::IDENTITY
            .looking_at(face_normal.any_orthogonal_vector(), face_normal)
            .with_translation(node.position + 0.5 * connection_height * face_normal);

        let material = if *cube_maze.maze.solution.first().unwrap() == node {
            white_material.clone()
        } else if *cube_maze.maze.solution.last().unwrap() == node {
            white_material.clone()
        } else {
            beige_material.clone()
        };

        commands.spawn(PbrBundle {
            mesh: cylinder_mesh,
            material,
            transform,
            ..default()
        });
    }

    for (source_node, target_node, edge) in cube_maze.maze.graph.all_edges() {
        let mesh = get_connection_mesh(
            source_node,
            target_node,
            cube_maze.distance_between_nodes,
            connection_height,
        );
        let connecting_mesh = meshes.add(mesh);

        let transform = get_connection_transform(source_node, target_node, connection_height);
        commands.spawn(PbrBundle {
            mesh: connecting_mesh,
            material: beige_material.clone(),
            transform,
            ..default()
        });
    }

    let cuboid = meshes.add(Cuboid::from_length(1.5));
    commands.spawn(PbrBundle {
        mesh: cuboid,
        material: green_material.clone(),
        transform: Transform::IDENTITY,
        ..default()
    });
}

fn get_connection_transform(from: CubeNode, to: CubeNode, connection_height: f32) -> Transform {
    let border_type = BorderType::from_faces(&from.face, &to.face);
    match border_type {
        BorderType::SameFace => {
            let forward = from.position - to.position;
            let middle = (from.position + to.position) / 2.0;
            Transform::IDENTITY
                .looking_to(forward, from.face.normal())
                .with_translation(middle + from.face.normal() * connection_height / 2.0)
        }
        BorderType::Connected => {
            let forward = from.face.normal().cross(to.face.normal());
            let translation = from.position.abs().max(to.position.abs()) * from.position.signum();
            Transform::IDENTITY
                .looking_to(forward, from.face.normal())
                .with_translation(translation)
        }
        _ => panic!["unknown edge types"],
    }
}