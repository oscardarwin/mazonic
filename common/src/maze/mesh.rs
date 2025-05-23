use bevy::{
    math::NormedVectorSpace,
    pbr::ExtendedMaterial,
    prelude::*,
    utils::{hashbrown::HashMap, HashSet},
};
use bevy_hanabi::prelude::*;
use rand::{seq::IteratorRandom, SeedableRng};
use rand_chacha::ChaCha8Rng;

use crate::{
    assets::{
        material_handles::MaterialHandles,
        mesh_handles::MeshHandles,
        shaders::{DashedArrowShader, PulsingShader},
    }, effects::musical_notes::{MusicalNoteEffectColor, MusicalNoteEffectHandle, MusicalNoteImageHandles, MusicalNoteMarker}, game_save::{CurrentPuzzle, DiscoveredMelody, PuzzleIdentifier}, game_systems::SystemHandles, is_room_junction::is_junction, levels::{GameLevel, PuzzleEntityMarker, Shape}, maze::maze_mesh_builder::MazeMeshBuilder, play_statistics::PlayStatistics, room::Room, shape::loader::{GraphComponent, SolutionComponent}
};

use super::border_type::BorderType;

const ROOM_HEIGHT: f32 = 0.002;
const SAME_FACE_EDGE_HEIGHT: f32 = 0.001;
const CROSS_FACE_EDGE_HEIGHT: f32 = 0.001;

#[derive(Component, Debug, Clone)]
pub struct MazeMarker;

pub fn spawn(
    mut commands: Commands,
    level_query: Query<&GameLevel>,
    maze_query: Query<(&GraphComponent, &SolutionComponent)>,
    mesh_handles: Res<MeshHandles>,
    material_handles: Res<MaterialHandles>,
    play_statistics: Res<PlayStatistics>,
    current_puzzle_query: Query<&CurrentPuzzle>,
    musical_note_effect_handle: Query<&MusicalNoteEffectHandle>,
    musical_note_image_handle_query: Query<&MusicalNoteImageHandles>,
) {
    let Ok(level) = level_query.get_single() else {
        return;
    };

    let Ok((GraphComponent(graph), SolutionComponent(solution))) = maze_query.get_single() else {
        return;
    };

    let Ok(CurrentPuzzle(puzzle_identifier)) = current_puzzle_query.get_single() else {
        return;
    };

    let discovered_melody_room_ids = play_statistics.get_melody_room_ids(puzzle_identifier);

    let discovered_melody_room_ids_to_melody_index = discovered_melody_room_ids
        .iter()
        .enumerate()
        .map(|(melody_index, room_id)| (room_id, melody_index))
        .collect::<HashMap<_, _>>();

    let distance_between_nodes = level.node_distance();

    let goal_node = solution.last().unwrap();
    for room in graph.nodes().filter(|room| is_junction(room, &graph)) {
        let is_goal_node = room == *goal_node;

        let transform = Transform::IDENTITY
            .looking_at(
                -room.face().normal(),
                room.face().normal().any_orthogonal_vector(),
            )
            .with_translation(room.position() + room.face().normal() * ROOM_HEIGHT);

        let mut entity_commands =
            commands.spawn((transform, PuzzleEntityMarker, room, Visibility::default()));

        let discovered_melody_room = discovered_melody_room_ids_to_melody_index.get(&room.id);

        let mesh_handle = if room == *goal_node {
            mesh_handles.goal_room.clone()
        } else {
            mesh_handles.junction_room.clone()
        };

        entity_commands.with_children(|parent| {
            let mut child_entity_commands = parent.spawn((
                Mesh3d(mesh_handle),
                Transform::IDENTITY.with_scale(Vec3::splat(distance_between_nodes)),
                MazeMarker,
            ));

            let material_handle = match (is_goal_node, discovered_melody_room) {
                (true, _) => child_entity_commands
                    .insert(MeshMaterial3d(material_handles.goal_handle.clone())),
                (false, Some(melody_index)) => child_entity_commands.insert((MeshMaterial3d(
                    material_handles.bright_line_handle.clone(),
                ), MusicalNoteMarker(*melody_index, MusicalNoteEffectColor::Line))),
                (false, None) => child_entity_commands
                    .insert(MeshMaterial3d(material_handles.line_handle.clone())),
            };
        });
    }

    let discovered_melody_room_pairs =
        make_room_pairs_from_discovered_melodies(puzzle_identifier, &discovered_melody_room_ids);

    let maze_mesh_handles = match &level.shape {
        Shape::Tetrahedron(_) => &mesh_handles.shape_maze_edge_mesh_handles.tetrahedron,
        Shape::Cube(_) => &mesh_handles.shape_maze_edge_mesh_handles.cube,
        Shape::Octahedron(_) => &mesh_handles.shape_maze_edge_mesh_handles.octahedron,
        Shape::Dodecahedron(_) => &mesh_handles.shape_maze_edge_mesh_handles.dodecahedron,
        Shape::Icosahedron(_) => &mesh_handles.shape_maze_edge_mesh_handles.icosahedron,
    };

    for (source_node, target_node, _) in graph.all_edges() {
        let bidirectional = graph.contains_edge(target_node, source_node);

        if bidirectional && source_node.cmp(&target_node).is_lt() {
            continue;
        }

        let Some(border_type) = level.border_type(&source_node.face(), &target_node.face()) else {
            panic!["unknown edge type"];
        };

        let mesh_handle = match (&border_type, bidirectional) {
            (BorderType::SameFace, true) => maze_mesh_handles.same_face_edge.clone(),
            (BorderType::SameFace, false) => maze_mesh_handles.one_way_same_face_edge.clone(),
            (BorderType::Connected, true) => maze_mesh_handles.cross_face_edge.clone(),
            (BorderType::Connected, false) => maze_mesh_handles.one_way_cross_face_edge.clone(),
        };

        let transform = get_connection_transform(source_node, target_node, &border_type);

        let is_discovered = discovered_melody_room_pairs
            .contains(&(source_node.id, target_node.id))
            || discovered_melody_room_pairs.contains(&(target_node.id, source_node.id));

        let mut entity_commands = commands
            .spawn((transform.clone(), PuzzleEntityMarker, Visibility::default()))
            .with_children(|parent| {
                let mut entity_commands = parent.spawn((
                    Mesh3d(mesh_handle),
                    Transform::IDENTITY.with_scale(Vec3::splat(distance_between_nodes)),
                    MazeMarker,
                ));

                match (bidirectional, is_discovered) {
                    (false, true) => entity_commands.insert(MeshMaterial3d(
                        material_handles.bright_dashed_arrow_handle.clone(),
                    )),
                    (false, false) => entity_commands
                        .insert(MeshMaterial3d(material_handles.dashed_arrow_handle.clone())),
                    (true, true) => entity_commands.insert(MeshMaterial3d(
                        material_handles.bright_line_handle.clone(),
                    )),
                    (true, false) => {
                        entity_commands.insert(MeshMaterial3d(material_handles.line_handle.clone()))
                    }
                };
            });
    }
}

fn get_connection_transform(from: Room, to: Room, border_type: &BorderType) -> Transform {
    match border_type {
        BorderType::SameFace => {
            let forward = from.position() - to.position();
            Transform::IDENTITY
                .looking_to(forward, from.face().normal())
                .with_translation(from.position() + from.face().normal() * SAME_FACE_EDGE_HEIGHT)
        }
        BorderType::Connected => get_cross_face_edge_transform(
            from.position(),
            from.face().normal(),
            to.position(),
            to.face().normal(),
        ),
    }
}

pub fn get_cross_face_edge_transform(
    from_position: Vec3,
    from_normal: Vec3,
    to_position: Vec3,
    to_normal: Vec3,
) -> Transform {
    let half_angle = from_normal.angle_between(to_normal) / 2.0;

    let average_normal = from_normal.lerp(to_normal, 0.5).normalize();

    let edge_vec = to_position - from_position;

    let intersection_point =
        from_position + (edge_vec + edge_vec.norm() * half_angle.tan() * average_normal) / 2.0;

    Transform::IDENTITY
        .looking_to(intersection_point - to_position, to_normal)
        .with_translation(intersection_point + average_normal * CROSS_FACE_EDGE_HEIGHT)
}

pub fn make_room_pairs_from_discovered_melodies(
    current_puzzle_identifier: &PuzzleIdentifier,
    melody_room_ids: &Vec<u64>,
) -> HashSet<(u64, u64)> {
    let mut room_pairs = HashSet::new();

    for (from, to) in melody_room_ids.iter().zip(melody_room_ids.iter().skip(1)) {
        room_pairs.insert((*from, *to));
    }

    room_pairs
}

pub fn update_on_melody_discovered(
    mut commands: Commands,
    system_handles: Res<SystemHandles>,
    maze_entities_query: Query<Entity, With<MazeMarker>>,
) {
    commands.run_system(system_handles.spawn_maze);
    for maze_entity in maze_entities_query.iter() {
        commands.entity(maze_entity).despawn_recursive();
    }
}
