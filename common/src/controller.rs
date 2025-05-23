use crate::{
    camera::MainCamera,
    controller_screen_position::ControllerScreenPosition,
    game_settings::GameSettings,
    game_state::PuzzleState,
    levels::GameLevel,
    maze::border_type::BorderType,
    player::{Player, PlayerMazeState},
    room::{Edge, Room},
    shape::loader::GraphComponent,
};
use bevy::{
    ecs::system::{Query, ResMut},
    input::{
        mouse::{MouseButton, MouseButtonInput},
        ButtonInput, ButtonState,
    },
    math::{primitives::InfinitePlane3d, NormedVectorSpace, Ray3d, Vec3},
    prelude::*,
    render::camera::Camera,
    state::state::NextState,
    transform::components::GlobalTransform,
    window::PrimaryWindow,
};
use bevy_rapier3d::{pipeline::QueryFilter, plugin::RapierContext};
use petgraph::{graphmap::GraphMap, Directed};

#[derive(SubStates, Default, Debug, Clone, PartialEq, Eq, Hash)]
#[source(PuzzleState = PuzzleState::Playing)]
pub enum ControllerState {
    #[default]
    IdlePostSolve,
    IdlePostView,
    Solving,
    Viewing,
}

#[derive(Default)]
pub struct Controller;

impl Plugin for Controller {
    fn build(&self, app: &mut App) {
        app.add_sub_state::<ControllerState>();
    }
}

pub fn idle(
    camera_query: Query<(&GlobalTransform, &Camera), With<MainCamera>>,
    primary_window: Query<&Window, With<PrimaryWindow>>,
    rapier_context_query: Query<&RapierContext>,
    mut next_controller_state: ResMut<NextState<ControllerState>>,
    mut mouse_button_event_reader: EventReader<MouseButtonInput>,
    controller_screen_position_query: Query<&ControllerScreenPosition>,
    mut local_previous_cursor_position: Local<Option<ControllerScreenPosition>>,
) {
    let Ok(controller_screen_position) = controller_screen_position_query.get_single() else {
        return;
    };

    let Some(previous_cursor_position) = &*local_previous_cursor_position else {
        *local_previous_cursor_position = Some(controller_screen_position.clone());
        return;
    };

    let (ControllerScreenPosition::None, ControllerScreenPosition::Position(cursor_position)) = (previous_cursor_position, controller_screen_position) else {
        *local_previous_cursor_position = Some(controller_screen_position.clone());
        return;
    };
    
    let (camera_global_transform, camera) = camera_query.single();

    let Some(ray) = camera
        .viewport_to_world(camera_global_transform, *cursor_position)
        .ok()
    else {
        return;
    };

    *local_previous_cursor_position = None;

    if rapier_context_query
        .single()
        .cast_ray(
            ray.origin,
            ray.direction.into(),
            30.,
            true,
            QueryFilter::default(),
        )
        .is_some()
    {
        next_controller_state.set(ControllerState::Solving);
    } else {
        next_controller_state.set(ControllerState::Viewing);
    }
}

pub fn view(
    mut next_controller_state: ResMut<NextState<ControllerState>>,
    controller_screen_position_query: Query<&ControllerScreenPosition>,
) {
    let Ok(controller_screen_position) = controller_screen_position_query.get_single() else {
        return;
    };

    match controller_screen_position {
        ControllerScreenPosition::None => {
            next_controller_state.set(ControllerState::IdlePostView);
        }
        _ => {}
    };
}

pub fn solve(
    controller_screen_position_query: Query<&ControllerScreenPosition>,
    camera_query: Query<(&GlobalTransform, &Camera)>,
    primary_window: Query<&Window, With<PrimaryWindow>>,
    mut player_query: Query<(&mut PlayerMazeState, &Player)>,
    mut mouse_button_event_reader: EventReader<MouseButtonInput>,
    level: Query<&GameLevel>,
    graph_query: Query<&GraphComponent>,
    mut next_controller_state: ResMut<NextState<ControllerState>>,
    game_settings: Res<GameSettings>,
    mut previous_cursor_position: Local<Option<Vec2>>,
) {
    let Ok(shape) = level.get_single() else {
        return;
    };

    let Ok(GraphComponent(graph)) = graph_query.get_single() else {
        return;
    };

    let Ok(controller_screen_position) = controller_screen_position_query.get_single() else {
        return;
    };

    let ControllerScreenPosition::Position(cursor_position) = controller_screen_position else {
        next_controller_state.set(ControllerState::IdlePostSolve);
        return;
    };

    if previous_cursor_position
        .filter(|position| position.distance(*cursor_position) < 2.0)
        .is_some()
    {
        return;
    } else {
        *previous_cursor_position = Some(*cursor_position);
    }

    let (camera_global_transform, camera) = camera_query.single();

    let Some(ray) = camera
        .viewport_to_world(camera_global_transform, *cursor_position)
        .ok()
    else {
        // if it was impossible to compute for whatever reason; we can't do anything
        return;
    };

    let (mut player_maze_state, Player { radius }) = player_query.single_mut();
    let player_elevation = game_settings.player_elevation + radius;
    let node_snap_threshold = shape.node_distance() * 0.1;

    if let Some(new_player_maze_state) = match player_maze_state.as_ref() {
        PlayerMazeState::Node(node) => {
            move_player_on_node(&node, &graph, player_elevation, node_snap_threshold, ray)
        }
        PlayerMazeState::Edge(from_node, to_node, _) => move_player_on_edge(
            &from_node,
            &to_node,
            ray,
            player_elevation,
            node_snap_threshold,
            &shape,
        ),
    } {
        *player_maze_state = new_player_maze_state;
    }
}

fn project_ray_to_controller_face(
    ray: Ray3d,
    cube_node: &Room,
    player_elevation: f32,
) -> Option<Vec3> {
    let plane_normal = cube_node.face().normal();
    let plane_point = cube_node.position() + player_elevation * plane_normal;

    ray.intersect_plane(plane_point, InfinitePlane3d::new(plane_normal))
        .map(|ray_distance| ray.origin + ray.direction.normalize() * ray_distance)
}

fn project_point_to_plane(point: Vec3, plane_position: Vec3, plane_normal: Vec3) -> Vec3 {
    point - plane_normal.dot(point - plane_position) * plane_normal
}

fn move_player_on_node(
    node: &Room,
    graph: &GraphMap<Room, Edge, Directed>,
    player_elevation: f32,
    node_snap_threshold: f32,
    ray: Ray3d,
) -> Option<PlayerMazeState> {
    let face_intersection_point = project_ray_to_controller_face(ray, node, player_elevation)?;

    let node_player_position = node.position() + node.face().normal() * player_elevation;

    let face_intersection_from_player = face_intersection_point - node_player_position;

    if face_intersection_from_player.norm() <= node_snap_threshold {
        return None;
    }

    let node_face_normal = node.face().normal();
    let node_player_plane_position = node.position() + player_elevation * node_face_normal;

    graph
        .edges(node.clone())
        .map(|(_, to_node, _)| to_node)
        .min_by_key(|to_node| {
            let to_node_position = to_node.position();

            let to_node_player_plane_position =
                project_point_to_plane(to_node_position, node_player_position, node_face_normal);

            let edge_vec = to_node_player_plane_position - node_player_plane_position;

            (edge_vec.angle_between(face_intersection_from_player) * 50.0) as u16
        })
        .map(|to_node| PlayerMazeState::Edge(node.clone(), to_node, node_player_plane_position))
}

fn move_player_on_edge(
    from_node: &Room,
    to_node: &Room,
    ray: Ray3d,
    player_elevation: f32,
    node_snap_threshold: f32,
    level: &GameLevel,
) -> Option<PlayerMazeState> {
    let player_plane_edge_intersection =
        compute_player_plane_edge_intersection(ray, from_node, to_node, player_elevation, level)?;

    let to_node_to_intersection = to_node.position() + to_node.face().normal() * player_elevation
        - player_plane_edge_intersection;

    let from_node_to_intersection = from_node.position()
        + from_node.face().normal() * player_elevation
        - player_plane_edge_intersection;

    let new_player_state = if to_node_to_intersection.norm() < node_snap_threshold {
        PlayerMazeState::Node(to_node.clone())
    } else if from_node_to_intersection.norm() < node_snap_threshold {
        PlayerMazeState::Node(from_node.clone())
    } else {
        PlayerMazeState::Edge(
            from_node.clone(),
            to_node.clone(),
            player_plane_edge_intersection,
        )
    };

    Some(new_player_state)
}

fn compute_player_plane_edge_intersection(
    screen_ray: Ray3d,
    from_node: &Room,
    to_node: &Room,
    player_elevation: f32,
    level: &GameLevel,
) -> Option<Vec3> {
    let from_face = from_node.face();
    let to_face = to_node.face();

    let border_type = level.border_type(&from_face, &to_face)?;

    match border_type {
        BorderType::SameFace => {
            compute_intersection_point_of_edge(screen_ray, &from_node, player_elevation, &to_node)
        }
        BorderType::Connected => {
            let to_plane_intersection = compute_intersection_point_of_edge(
                screen_ray,
                &to_node,
                player_elevation,
                &from_node,
            );

            let from_plane_intersection = compute_intersection_point_of_edge(
                screen_ray,
                &from_node,
                player_elevation,
                &to_node,
            );
            
            std::cmp::max_by_key(
                from_plane_intersection,
                to_plane_intersection,
                |opt_intersection| opt_intersection.map(|x| (1000.0 / x.norm()) as u16),
            )
        }
    }
}

fn compute_intersection_point_of_edge(
    ray: Ray3d,
    from_room: &Room,
    elevation: f32,
    to_room: &Room,
) -> Option<Vec3> {
    let from_normal = from_room.face().normal();

    if from_normal.dot(Vec3::from(ray.direction)) > 0.0 {
        return None;
    }

    let room_controller_position = from_room.position() + elevation * from_normal;
    let to_room_controller_position = to_room.position() + elevation * to_room.face().normal();

    let other_node_on_from_controller_plane = project_point_to_plane(
        to_room_controller_position,
        room_controller_position,
        from_normal,
    );

    let from_controller_to = other_node_on_from_controller_plane - room_controller_position;
    let projected_ray_on_face = project_ray_to_controller_face(ray, from_room, elevation)?;
    
    let relative_intersection_point = projected_ray_on_face - room_controller_position;
    let distance_along_node_other_vec = relative_intersection_point.dot(from_controller_to) / from_controller_to.dot(from_controller_to);
    
    Some(distance_along_node_other_vec.clamp(0.0, 1.0) * from_controller_to + room_controller_position)
}
