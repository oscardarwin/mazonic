use bevy::{input::touch::TouchPhase, prelude::*, utils::HashSet};
use mazonic::{self, controller_screen_position::ControllerScreenPosition};

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
    mut controller_screen_position_query: Query<&mut ControllerScreenPosition>,
) {
    let Ok(mut controller_screen_position) = controller_screen_position_query.get_single_mut()
    else {
        return;
    };

    println!("{:?}", controller_screen_position);
    *controller_screen_position = match touches.iter().next() {
        Some(touch) => ControllerScreenPosition::Position(touch.position()),
        None => ControllerScreenPosition::None,
    };
}
