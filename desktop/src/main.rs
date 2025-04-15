use bevy::prelude::*;

use mazonic;

fn main() {
    let mut app = App::new();
    mazonic::add_common_plugins(&mut app);
    app.run();
}
