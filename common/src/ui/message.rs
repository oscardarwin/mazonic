use bevy::{ecs::query::QueryData, prelude::*};

use crate::constants::{FONT_PATH, TEXT_COLOR, TRANSPARENCY};


const FADE_START_TIME_SECONDS: f32 = 3.0;
const FADE_END_TIME_SECONDS: f32 = 5.0;
const FADE_DURATION_SECONDS: f32 = FADE_END_TIME_SECONDS - FADE_START_TIME_SECONDS;
const FONT_SIZE: f32 = 30.0;

#[derive(Resource, Default)]
pub struct MessagePopup(pub String);

#[derive(Component)]
pub struct MessagePopupUI;

pub fn update(
    time: Res<Time>,
    mut reset_time: Local<f32>,
    message_popup: Res<MessagePopup>,
    mut popup_ui_query: Query<(&mut Text, &mut TextColor), With<MessagePopupUI>>
) {
    let (mut text, mut text_color) = popup_ui_query.single_mut();

    if message_popup.is_changed() {
        *reset_time = time.elapsed_secs();

        text.0 = message_popup.0.clone();
        text_color.0.set_alpha(TRANSPARENCY);
    }

    let delta = time.elapsed_secs() - *reset_time;

    if delta > FADE_START_TIME_SECONDS && delta < FADE_END_TIME_SECONDS {
        let fade_proportion = (delta - FADE_START_TIME_SECONDS) / FADE_DURATION_SECONDS;
        text_color.0.set_alpha(1.0 - fade_proportion);
    }
}

pub fn spawn(
    mut commands: Commands,
    asset_server: Res<AssetServer>
) {
    commands.insert_resource(MessagePopup::default());

    let font = asset_server.load(FONT_PATH);
    commands
        .spawn(Node {
            width: Val::Percent(100.0),
            height: Val::Percent(100.0),
            justify_content: JustifyContent::Center,
            border: UiRect::all(Val::Px(10.)),
            ..default()
        })
        .insert(PickingBehavior::IGNORE)
        .with_child(
            (
                Text::new("".to_string()),
                TextFont {
                    font: font.clone(),
                    font_size: FONT_SIZE,
                    ..default()
                },
                TextColor(TEXT_COLOR),
                MessagePopupUI,
            )
        );


}

