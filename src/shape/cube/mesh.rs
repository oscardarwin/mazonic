use std::f32::consts::PI;

use bevy::{
    color::palettes::basic::SILVER,
    math::{vec2, NormedVectorSpace},
    prelude::*,
    render::{
        mesh::{Indices, PrimitiveTopology},
        render_asset::RenderAssetUsages,
        render_resource::{Extent3d, TextureDimension, TextureFormat},
    },
    window::PrimaryWindow,
};
use bevy_rapier3d::{
    geometry::Collider,
    pipeline::QueryFilter,
    plugin::{NoUserData, RapierContext, RapierPhysicsPlugin},
};

use super::maze::{BorderType, CubeNode};

pub fn get_connection_mesh(
    from: CubeNode,
    to: CubeNode,
    distance_between_nodes: f32,
    connection_height: f32,
) -> Mesh {
    let border_type = BorderType::get_from_faces(&from.face, &to.face);

    let width = 0.06;
    let node_size = 0.1;

    match border_type {
        BorderType::SameFace => {
            let length = (distance_between_nodes - node_size);
            let half_width = width / 2.0;
            let cube = Cuboid::new(width, connection_height, length).into();
            cube
        }
        BorderType::Connected => {
            create_edge_piece(width, connection_height, distance_between_nodes, node_size)
        }
        _ => panic!["stop"],
    }
}

fn create_edge_piece(width: f32, height: f32, distance_between_nodes: f32, node_size: f32) -> Mesh {
    let half_width = width / 2.0;
    let half_distance_to_node = (distance_between_nodes - node_size) / 2.0;
    let uv_mid_point = height / height + half_distance_to_node;

    let vertices = vec![
        //L +Z face
        [0.0, 0.0, half_width],
        [height, height, half_width],
        [-half_distance_to_node, 0.0, half_width],
        [-half_distance_to_node, height, half_width],
        [0.0, -half_distance_to_node, half_width],
        [height, -half_distance_to_node, half_width],
        //L -Z face
        [0.0, 0.0, -half_width],
        [height, height, -half_width],
        [-half_distance_to_node, 0.0, -half_width],
        [-half_distance_to_node, height, -half_width],
        [0.0, -half_distance_to_node, -half_width],
        [height, -half_distance_to_node, -half_width],
        //X normal face
        [height, height, half_width],
        [height, height, -half_width],
        [height, -half_distance_to_node, -half_width],
        [height, -half_distance_to_node, half_width],
        //Y normal face
        [height, height, half_width],
        [height, height, -half_width],
        [-half_distance_to_node, height, -half_width],
        [-half_distance_to_node, height, half_width],
    ]
    .into_iter()
    .map(|arr| Vec3::from_array(arr))
    .collect::<Vec<Vec3>>();

    let total_corner_piece_length = height + half_distance_to_node;
    let uv_coords = vertices
        .clone()
        .into_iter()
        .map(|vec| (vec + half_distance_to_node) / total_corner_piece_length)
        .map(|vec| Vec2::new(vec.x, vec.y))
        .collect::<Vec<Vec2>>();

    let normals = vertices
        .clone()
        .into_iter()
        .enumerate()
        .map(|(i, _)| match i {
            0..6 => Vec3::Z,
            6..12 => -Vec3::Z,
            12..16 => Vec3::X,
            16..20 => Vec3::Y,
            _ => panic!["stop"],
        })
        .collect::<Vec<Vec3>>();

    // Create a new mesh using a triangle list topology, where each set of 3 vertices composes a triangle.
    Mesh::new(
        PrimitiveTopology::TriangleList,
        RenderAssetUsages::default(),
    )
    .with_inserted_attribute(Mesh::ATTRIBUTE_POSITION, vertices)
    .with_inserted_attribute(Mesh::ATTRIBUTE_UV_0, uv_coords)
    // Assign normals (everything points outwards)
    .with_inserted_attribute(Mesh::ATTRIBUTE_NORMAL, normals)
    .with_inserted_indices(Indices::U32(vec![
        0, 1, 3, 0, 5, 1, 0, 3, 2, 0, 4, 5, // +Z face
        6, 9, 7, 6, 7, 11, 6, 8, 9, 6, 11, 10, // -Z face
        12, 14, 13, 12, 15, 14, // X face
        16, 17, 18, 16, 18, 19, // Y face
    ]))
}
