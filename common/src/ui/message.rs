use std::ops::AddAssign;

use bevy::{ecs::query::QueryData, prelude::*, time::Stopwatch};

use crate::{constants::{FONT_PATH, TEXT_COLOR, TRANSPARENCY}, game_save::CurrentPuzzle, play_statistics::{PlayStatistics, PuzzleStatistics, SolveTime}};


const FADE_START_TIME_SECONDS: f32 = 3.0;
const FADE_END_TIME_SECONDS: f32 = 5.0;
const FADE_DURATION_SECONDS: f32 = FADE_END_TIME_SECONDS - FADE_START_TIME_SECONDS;
const FONT_SIZE: f32 = 30.0;

#[derive(Component, Debug, Default)]
pub struct MessagePopup(pub String);

#[derive(Component)]
pub struct MessagePopupStopwatch(pub Stopwatch);

#[derive(Component)]
pub struct MessagePopupUpperMarker;

#[derive(Component)]
pub struct MessagePopupLowerMarker;

pub fn on_change(
    mut popup_ui_query: Query<(&mut Text, &mut TextColor, &MessagePopup, &mut MessagePopupStopwatch), Changed<MessagePopup>> 
) {
    for (mut text, mut text_color, popup, mut stopwatch) in popup_ui_query.iter_mut() {
        stopwatch.0.reset();
        text.0 = popup.0.clone();
        text_color.0.set_alpha(TRANSPARENCY);
    }
}

pub fn update_upper(
    time: Res<Time>,
    mut reset_time: Local<f32>,
    mut popup_ui_query: Query<(&mut Text, &mut TextColor, &MessagePopup, &mut MessagePopupStopwatch)> 
) {
    for (mut text, mut text_color, popup, mut stopwatch) in popup_ui_query.iter_mut() {
        stopwatch.0.tick(time.delta());
        let elapsed = stopwatch.0.elapsed().as_secs_f32();
        
        if elapsed > FADE_START_TIME_SECONDS && elapsed < FADE_END_TIME_SECONDS {
            let fade_proportion = (elapsed - FADE_START_TIME_SECONDS) / FADE_DURATION_SECONDS;
            text_color.0.set_alpha(1.0 - fade_proportion);
        }
    }
}

pub fn spawn(
    mut commands: Commands,
    asset_server: Res<AssetServer>
) {
    let font = asset_server.load(FONT_PATH);
    commands
        .spawn(Node {
            width: Val::Percent(100.0),
            height: Val::Percent(100.0),
            justify_content: JustifyContent::SpaceBetween,
            flex_direction: FlexDirection::Column,
            align_items: AlignItems::Center,
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
                MessagePopupUpperMarker,
                MessagePopup("HELLO UPPER".to_string()),
                MessagePopupStopwatch(Stopwatch::new()),
            )
        )
        .with_child(
            (
                Text::new("".to_string()),
                TextFont {
                    font: font.clone(),
                    font_size: FONT_SIZE,
                    ..default()
                },
                TextColor(TEXT_COLOR),
                MessagePopupLowerMarker,
                MessagePopup("HELLO LOWER".to_string()),
            )
        );
}

pub fn update_lower_during_puzzle_state(
    solve_time: Res<SolveTime>,
    mut popup_ui_query: Query<&mut Text, With<MessagePopupLowerMarker>>,
) {
    if solve_time.is_changed() {
        let mut text = popup_ui_query.single_mut();
        text.0 = format!("{:.1}s", solve_time.stopwatch.elapsed().as_secs_f32());
    }

}

pub fn exit_puzzle_state(mut popup_ui_query: Query<&mut Text, With<MessagePopupLowerMarker>>) {
    let mut text = popup_ui_query.single_mut();
    text.0 = "".to_string();
}
