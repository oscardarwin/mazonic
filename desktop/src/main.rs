use bevy::{prelude::*, window::PrimaryWindow};

use mazonic::{self, camera::CameraTarget, controller_screen_position::ControllerScreenPosition};

fn main() {
    let mut app = App::new();
    mazonic::add_common_plugins(&mut app);

    app.add_systems(Update, update_controller_position);
    app.add_systems(Update, update_zoom);
    app.run();
}

fn update_controller_position(
    mouse_button_input: Res<ButtonInput<MouseButton>>,
    window_query: Query<&Window, With<PrimaryWindow>>,
    mut controller_screen_position_query: Query<&mut ControllerScreenPosition>,
) {
    let Ok(mut controller_screen_position) = controller_screen_position_query.get_single_mut()
    else {
        return;
    };

    let Ok(window) = window_query.get_single() else {
        return;
    };
    let is_pressed = mouse_button_input.pressed(MouseButton::Left);
    
    *controller_screen_position = match window.cursor_position() {
        Some(position) if is_pressed => ControllerScreenPosition::Position(position),
        _ => ControllerScreenPosition::None,
    };
}

fn update_zoom(
    keys: Res<ButtonInput<KeyCode>>,
    camera_target_query: Query<&mut CameraTarget>,
    ) {
    let zoom_out = keys.pressed(KeyCode::Minus);
    let zoom_in = keys.pressed(KeyCode::Equal);

    match (zoom_out, zoom_in) {
        (false, false) | (true, true) => return,
        (true, false) => zoom(camera_target_query, 0.1),
        (false, true) => zoom(camera_target_query, -0.1),

    }
}

fn zoom(mut camera_target_query: Query<&mut CameraTarget>, amount: f32) {
    let Ok(mut camera_target) = camera_target_query.get_single_mut() else {
        return;
    };

    let target_zoom = camera_target.translation_norm + amount;

    camera_target.set_zoom(target_zoom);
}

