use crate::{
    game_settings::GameSettings,
    player::PlayerMazeState,
    shape::{
        cube::Cube,
        loader::PlatonicLevelData,
        platonic_solid::{BorderType, Edge, HasFace, IsRoom, PlatonicSolid},
        tetrahedron::Tetrahedron,
    },
};
use bevy::{
    ecs::system::{Query, ResMut},
    input::{mouse::MouseButton, ButtonInput},
    math::{primitives::InfinitePlane3d, NormedVectorSpace, Ray3d, Vec3},
    prelude::*,
    render::camera::Camera,
    state::state::NextState,
    transform::components::GlobalTransform,
    window::PrimaryWindow,
};
use bevy_rapier3d::{pipeline::QueryFilter, plugin::RapierContext};
use maze_generator::config::Maze;

#[derive(States, Default, Debug, Clone, PartialEq, Eq, Hash)]
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
        app.init_state::<ControllerState>()
            .add_systems(
                Update,
                idle.run_if(in_state(ControllerState::IdlePostSolve)),
            )
            .add_systems(Update, idle.run_if(in_state(ControllerState::IdlePostView)))
            .add_systems(Update, view.run_if(in_state(ControllerState::Viewing)));
    }
}

fn idle(
    camera_query: Query<(&GlobalTransform, &Camera)>,
    primary_window: Query<&Window, With<PrimaryWindow>>,
    mouse_buttons: Res<ButtonInput<MouseButton>>,
    rapier_context_query: Query<&RapierContext>,
    mut next_controller_state: ResMut<NextState<ControllerState>>,
) {
    if !mouse_buttons.pressed(MouseButton::Left) || mouse_buttons.just_pressed(MouseButton::Left) {
        return;
    }

    let Ok(window) = primary_window.get_single() else {
        return;
    };

    let Some(cursor_position) = window.cursor_position() else {
        // if the cursor is not inside the window, we can't do anything
        return;
    };

    let (camera_global_transform, camera) = camera_query.single();

    let Some(ray) = camera
        .viewport_to_world(camera_global_transform, cursor_position)
        .ok()
    else {
        // if it was impossible to compute for whatever reason; we can't do anything
        return;
    };

    if rapier_context_query
        .single()
        .cast_ray(
            ray.origin,
            ray.direction.into(),
            4.0,
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

fn view(
    mouse_buttons: Res<ButtonInput<MouseButton>>,
    mut next_controller_state: ResMut<NextState<ControllerState>>,
) {
    if !mouse_buttons.pressed(MouseButton::Left) || mouse_buttons.just_pressed(MouseButton::Left) {
        next_controller_state.set(ControllerState::IdlePostView);
        return;
    }
}

pub fn solve<P: PlatonicSolid>(
    camera_query: Query<(&GlobalTransform, &Camera)>,
    primary_window: Query<&Window, With<PrimaryWindow>>,
    mut player_query: Query<&mut PlayerMazeState<P>>,
    mouse_buttons: Res<ButtonInput<MouseButton>>,
    level: Res<PlatonicLevelData<P>>,
    mut next_controller_state: ResMut<NextState<ControllerState>>,
    game_settings: Res<GameSettings>,
) {
    if !mouse_buttons.pressed(MouseButton::Left) || mouse_buttons.just_pressed(MouseButton::Left) {
        next_controller_state.set(ControllerState::IdlePostSolve);
        return;
    }

    let Ok(window) = primary_window.get_single() else {
        return;
    };

    let Some(cursor_position) = window.cursor_position() else {
        // if the cursor is not inside the window, we can't do anything
        return;
    };

    let (camera_global_transform, camera) = camera_query.single();

    let Some(ray) = camera
        .viewport_to_world(camera_global_transform, cursor_position)
        .ok()
    else {
        // if it was impossible to compute for whatever reason; we can't do anything
        return;
    };

    // get plane for cuboid.
    let mut player_maze_state = player_query.single_mut();

    if let Some(new_player_maze_state) = match player_maze_state.as_ref() {
        PlayerMazeState::<P>::Node(node) => {
            move_player_on_node::<P>(&node, &level.maze, game_settings.player_elevation, ray)
        }
        PlayerMazeState::<P>::Edge(from_node, to_node, _) => {
            move_player_on_edge::<P>(&from_node, &to_node, ray, game_settings.player_elevation)
        }
    } {
        *player_maze_state = new_player_maze_state;
    }
}

fn project_ray_to_player_face<P: PlatonicSolid>(
    ray: Ray3d,
    cube_node: &P::Room,
    player_elevation: f32,
) -> Option<Vec3> {
    let plane_normal = cube_node.face().normal();
    let plane_point = cube_node.position() + player_elevation * plane_normal;

    ray.intersect_plane(plane_point, InfinitePlane3d::new(plane_normal))
        .map(|ray_distance| ray.origin + ray.direction.normalize() * ray_distance)
}

fn project_point_to_plane(point: &Vec3, plane_position: Vec3, plane_normal: &Vec3) -> Vec3 {
    *point - plane_normal.dot(*point - plane_position) * *plane_normal
}

fn move_player_on_node<P: PlatonicSolid>(
    node: &P::Room,
    maze: &Maze<P::Room, Edge>,
    player_elevation: f32,
    ray: Ray3d,
) -> Option<PlayerMazeState<P>> {
    let face_intersection_point = project_ray_to_player_face::<P>(ray, node, player_elevation)?;

    let node_player_position = node.position() + node.face().normal() * player_elevation;

    let face_intersection_from_player = face_intersection_point - node_player_position;

    if face_intersection_from_player.norm() <= 0.18 {
        return None;
    }

    let node_face_normal = node.face().normal();
    let node_player_plane_position = node.position() + player_elevation * node_face_normal;

    maze.graph
        .edges(node.clone())
        .map(|(_, to_node, _)| to_node)
        .min_by_key(|to_node| {
            let to_node_position = to_node.position();

            let to_node_player_plane_position =
                project_point_to_plane(&to_node_position, node_player_position, &node_face_normal);

            let edge_vec = to_node_player_plane_position - node_player_plane_position;

            (edge_vec.angle_between(face_intersection_from_player) * 50.0) as u16
        })
        .map(|to_node| {
            PlayerMazeState::<P>::Edge(node.clone(), to_node, node_player_plane_position)
        })
}

fn move_player_on_edge<P: PlatonicSolid>(
    from_node: &P::Room,
    to_node: &P::Room,
    ray: Ray3d,
    player_elevation: f32,
) -> Option<PlayerMazeState<P>> {
    let player_plane_edge_intersection =
        compute_player_plane_edge_intersection::<P>(ray, from_node, to_node, player_elevation)?;

    let to_node_distance = to_node.position() + to_node.face().normal() * player_elevation
        - player_plane_edge_intersection;

    let from_node_distance = from_node.position() + from_node.face().normal() * player_elevation
        - player_plane_edge_intersection;

    let new_player_state = if to_node_distance.norm() < 0.18 {
        PlayerMazeState::<P>::Node(to_node.clone())
    } else if from_node_distance.norm() < 0.18 {
        PlayerMazeState::<P>::Node(from_node.clone())
    } else {
        PlayerMazeState::<P>::Edge(
            from_node.clone(),
            to_node.clone(),
            player_plane_edge_intersection,
        )
    };

    Some(new_player_state)
}

fn compute_player_plane_edge_intersection<P: PlatonicSolid>(
    screen_ray: Ray3d,
    from_node: &P::Room,
    to_node: &P::Room,
    player_elevation: f32,
) -> Option<Vec3> {
    let from_plane_intersection =
        compute_intersection_point_of_edge::<P>(screen_ray, &from_node, player_elevation, &to_node);

    match &from_node.face().border_type(&to_node.face())? {
        BorderType::SameFace => from_plane_intersection,
        BorderType::Connected => {
            let to_plane_intersection = compute_intersection_point_of_edge::<P>(
                screen_ray,
                &to_node,
                player_elevation,
                &from_node,
            );

            std::cmp::min_by_key(
                from_plane_intersection,
                to_plane_intersection,
                |opt_intersection| opt_intersection.map(|x| (100.0 * x.norm()) as u16),
            )
        }
    }
}

fn compute_intersection_point_of_edge<P: PlatonicSolid>(
    ray: Ray3d,
    room: &P::Room,
    elevation: f32,
    other_edge_room: &P::Room,
) -> Option<Vec3> {
    let plane_normal = room.face().normal();

    if plane_normal.dot(Vec3::from(ray.direction)) > 0.0 {
        return None;
    }

    let plane_point = room.position() + elevation * plane_normal;
    let other_node_on_player_plane =
        project_point_to_plane(&other_edge_room.position(), plane_point, &plane_normal);

    let node_to_other_vec = other_node_on_player_plane - plane_point;
    let project_ray_on_face = project_ray_to_player_face::<P>(ray, room, elevation);

    project_ray_on_face
        .map(|intersection_point| intersection_point - plane_point)
        .map(|relative_intersection_point| {
            relative_intersection_point.dot(node_to_other_vec)
                / node_to_other_vec.dot(node_to_other_vec)
        })
        .map(|distance_along_node_other_vec| distance_along_node_other_vec.clamp(0.0, 1.0))
        .map(|clamped_distance_along_node_to_other_vec| {
            clamped_distance_along_node_to_other_vec * node_to_other_vec + plane_point
        })
}
