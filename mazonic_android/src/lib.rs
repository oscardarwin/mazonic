use bevy::{input::touch::Touch, prelude::*};
use mazonic::{self, camera::CameraTarget, controller_screen_position::ControllerScreenPosition};

#[bevy_main]
fn main() {
    let mut app = App::new();

    let android_app = bevy::window::ANDROID_APP
        .get()
        .expect("Bevy must be set up with the #[bevy_main] macro on Android");

    let internal_storage_path = android_app.internal_data_path().unwrap();

    let save_location = mazonic::game_save::SaveLocation(internal_storage_path.clone());

    app.insert_resource(save_location);

    println!("Internal storage path: {:?}", internal_storage_path);

    mazonic::add_common_plugins(&mut app);

    app.add_systems(Update, update_controller_position);

    app.run();
}

fn update_controller_position(
    touches: Res<Touches>,
    mut camera_target_query: Query<&mut CameraTarget>,
    mut local_start_camera_norm: Local<Option<f32>>,
    mut controller_screen_position_query: Query<&mut ControllerScreenPosition>,
) {
    let Ok(mut controller_screen_position) = controller_screen_position_query.get_single_mut()
    else {
        return;
    };

    let Ok(mut camera_target) = camera_target_query.get_single_mut() else {
        return;
    };

    let touches_vec = touches.iter().collect::<Vec<_>>();
    let touches_slice = touches_vec.as_slice();

    *controller_screen_position = match touches_slice {
        [touch] => ControllerScreenPosition::Position(touch.position()),
        _ => ControllerScreenPosition::None,
    };

    if let [touch_1, touch_2] = touches_slice {
        let start_camera_norm = match *local_start_camera_norm {
            Some(norm) => norm,
            None => {
                *local_start_camera_norm = Some(camera_target.translation_norm);
                camera_target.translation_norm
            }
        };

        let zoom_coefficient = compute_target_zoom_level(touch_1, touch_2);

        camera_target.set_zoom(start_camera_norm * zoom_coefficient);
    } else {
        *local_start_camera_norm = None;
    }
}

fn compute_target_zoom_level(touch_1: &Touch, touch_2: &Touch) -> f32 {
    let current_width = touch_1.position().distance(touch_2.position());
    let starting_width = touch_1.start_position().distance(touch_2.start_position());

    starting_width / f32::max(current_width, 1.0)
}
