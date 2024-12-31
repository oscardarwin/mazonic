use bevy::prelude::*;

use crate::{
    game_state::GameState,
    shape::loader::{LevelIndex, LevelType, Levels},
};

#[derive(Default)]
pub struct UiPlugin;

impl Plugin for UiPlugin {
    fn build(&self, app: &mut App) {
        #[cfg(feature = "bevy_dev_tools")]
        {
            app.add_plugins(bevy::dev_tools::ui_debug_overlay::DebugUiPlugin)
                .add_systems(Update, toggle_overlay);
        }
    }
}

#[derive(Component)]
pub struct PreviousLevelButton;

#[derive(Component)]
pub struct ReplayLevelButton;

#[derive(Component)]
pub struct NextLevelButton;

const FONT_PATH: &str = "fonts/Slimamifbold.ttf";
const NORMAL_BUTTON: Color = Color::srgb(0.15, 0.15, 0.15);
const HOVERED_BUTTON: Color = Color::srgb(0.25, 0.25, 0.25);
const PRESSED_BUTTON: Color = Color::srgb(0.65, 0.65, 0.65);

pub fn spawn_level_complete_ui(mut commands: Commands, asset_server: Res<AssetServer>) {
    commands
        .spawn(Node {
            width: Val::Percent(100.0),
            height: Val::Percent(100.0),
            justify_content: JustifyContent::Center,
            ..default()
        })
        .insert(PickingBehavior::IGNORE)
        .with_children(|parent| {
            parent
                .spawn(Node {
                    width: Val::Px(256.),
                    height: Val::Percent(100.),
                    flex_direction: FlexDirection::Column,
                    justify_content: JustifyContent::End,
                    padding: UiRect::all(Val::Px(5.)),
                    row_gap: Val::Px(5.),
                    ..default()
                })
                .with_children(|parent| {
                    parent
                        .spawn((
                            Node {
                                width: Val::Percent(100.),
                                height: Val::Px(64.),
                                flex_direction: FlexDirection::Row,
                                padding: UiRect::all(Val::Px(5.)),
                                row_gap: Val::Px(5.),
                                ..default()
                            },
                            BackgroundColor(Color::srgb(0.95, 0.85, 0.85)),
                        ))
                        .with_children(|parent| {
                            parent
                                .spawn((
                                    Button,
                                    Node {
                                        width: Val::Percent(25.),
                                        height: Val::Percent(100.),
                                        border: UiRect::all(Val::Px(5.0)),
                                        // horizontally center child text
                                        justify_content: JustifyContent::Center,
                                        // vertically center child text
                                        align_items: AlignItems::Center,
                                        ..default()
                                    },
                                    BorderColor(Color::BLACK),
                                    BorderRadius::MAX,
                                    BackgroundColor(NORMAL_BUTTON),
                                ))
                                .insert(PreviousLevelButton)
                                .with_child((
                                    Text::new("←"),
                                    TextFont {
                                        font: asset_server.load(FONT_PATH),
                                        font_size: 33.0,
                                        ..default()
                                    },
                                    TextColor(Color::srgb(0.9, 0.9, 0.9)),
                                ));
                            parent.spawn((
                                Node {
                                    width: Val::Percent(25.),
                                    height: Val::Percent(100.),
                                    border: UiRect::all(Val::Px(5.0)),
                                    // horizontally center child text
                                    justify_content: JustifyContent::Center,
                                    // vertically center child text
                                    align_items: AlignItems::Center,
                                    ..default()
                                },
                                BackgroundColor(NORMAL_BUTTON),
                            ));

                            parent
                                .spawn((
                                    Button,
                                    Node {
                                        width: Val::Percent(25.),
                                        height: Val::Percent(100.),
                                        border: UiRect::all(Val::Px(5.0)),
                                        // horizontally center child text
                                        justify_content: JustifyContent::Center,
                                        // vertically center child text
                                        align_items: AlignItems::Center,
                                        ..default()
                                    },
                                    BorderColor(Color::BLACK),
                                    BorderRadius::MAX,
                                    BackgroundColor(NORMAL_BUTTON),
                                ))
                                .insert(ReplayLevelButton)
                                .with_child((
                                    Text::new("↻"),
                                    TextFont {
                                        font: asset_server.load(FONT_PATH),
                                        font_size: 33.0,
                                        ..default()
                                    },
                                    TextColor(Color::srgb(0.9, 0.9, 0.9)),
                                ));

                            parent
                                .spawn((
                                    Button,
                                    Node {
                                        width: Val::Percent(25.),
                                        height: Val::Percent(100.),
                                        border: UiRect::all(Val::Px(5.0)),
                                        // horizontally center child text
                                        justify_content: JustifyContent::Center,
                                        // vertically center child text
                                        align_items: AlignItems::Center,
                                        ..default()
                                    },
                                    BorderColor(Color::BLACK),
                                    BorderRadius::MAX,
                                    BackgroundColor(NORMAL_BUTTON),
                                ))
                                .insert(NextLevelButton)
                                .with_child((
                                    Text::new("→"),
                                    TextFont {
                                        font: asset_server.load(FONT_PATH),
                                        font_size: 33.0,
                                        ..default()
                                    },
                                    TextColor(Color::srgb(0.9, 0.9, 0.9)),
                                ));
                        });
                });
        });
}

pub fn despawn_level_complete_ui(mut commands: Commands, ui_entities: Query<Entity, With<Node>>) {
    println!("despawn_level_complete_ui");
    for entity in ui_entities.iter() {
        commands.entity(entity).despawn();
    }
}

#[cfg(feature = "bevy_dev_tools")]
fn toggle_overlay(
    input: Res<ButtonInput<KeyCode>>,
    mut options: ResMut<bevy::dev_tools::ui_debug_overlay::UiDebugOptions>,
) {
    info_once!("The debug outlines are enabled, press Space to turn them on/off");
    if input.just_pressed(KeyCode::Space) {
        // The toggle method will enable the debug_overlay if disabled and disable if enabled
        options.toggle();
    }
}

pub fn update_level_complete_ui(
    mut interaction_query: Query<
        (&Interaction, &mut BackgroundColor, &mut BorderColor),
        (Changed<Interaction>, With<Button>),
    >,
) {
    for (interaction, mut color, mut border_color) in &mut interaction_query {
        match *interaction {
            Interaction::Pressed => {
                *color = PRESSED_BUTTON.into();
                border_color.0 = Color::WHITE;
            }
            Interaction::Hovered => {
                *color = HOVERED_BUTTON.into();
                border_color.0 = Color::WHITE;
            }
            Interaction::None => {
                *color = NORMAL_BUTTON.into();
                border_color.0 = Color::BLACK;
            }
        }
    }
}

pub fn previous_level(
    interaction_query: Query<
        &Interaction,
        (
            Changed<Interaction>,
            With<Button>,
            With<PreviousLevelButton>,
        ),
    >,
    mut level_index: ResMut<LevelIndex>,
    mut game_state: ResMut<NextState<GameState>>,
) {
    let Ok(interaction) = interaction_query.get_single() else {
        return;
    };

    if *interaction == Interaction::Pressed && level_index.0 > 0 {
        println!("previous level");

        let LevelIndex(index) = level_index.clone();
        level_index.0 -= 1;
        game_state.set(GameState::Loading);
    }
}

pub fn replay_level(
    interaction_query: Query<
        &Interaction,
        (Changed<Interaction>, With<Button>, With<ReplayLevelButton>),
    >,
    mut game_state: ResMut<NextState<GameState>>,
) {
    let Ok(interaction) = interaction_query.get_single() else {
        return;
    };

    if *interaction == Interaction::Pressed {
        println!("replay level");
        game_state.set(GameState::Loading);
    }
}

pub fn next_level(
    interaction_query: Query<
        &Interaction,
        (Changed<Interaction>, With<Button>, With<NextLevelButton>),
    >,
    mut level_index: ResMut<LevelIndex>,
    mut game_state: ResMut<NextState<GameState>>,
) {
    let Ok(interaction) = interaction_query.get_single() else {
        return;
    };

    if *interaction == Interaction::Pressed {
        println!("next level");

        let LevelIndex(index) = level_index.clone();
        level_index.0 += 1;
        game_state.set(GameState::Loading);
    }
}
