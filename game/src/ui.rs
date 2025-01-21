use bevy::{
    prelude::*,
    ui::widget::{ImageNodeSize, NodeImageMode},
};

use crate::{
    game_state::{GameState, PlayState},
    level_selector::SaveData,
    levels::LEVELS,
    shape::loader::{GraphComponent, SolutionComponent},
    statistics::PlayerPath,
};

#[derive(Component)]
pub struct PreviousLevelButton;

#[derive(Component)]
pub struct ReplayLevelButton;

#[derive(Component)]
pub struct NextLevelButton;

#[derive(Component)]
pub struct LevelSelectorButton;

const FONT_PATH: &str = "fonts/Slimamifbold.ttf";
const NORMAL_BUTTON: Color = Color::srgb(0.15, 0.15, 0.15);
const HOVERED_BUTTON: Color = Color::srgb(0.25, 0.25, 0.25);
const PRESSED_BUTTON: Color = Color::srgb(0.65, 0.65, 0.65);

pub fn spawn_navigation_ui(mut commands: Commands, asset_server: Res<AssetServer>) {
    let font = asset_server.load(FONT_PATH);
    let font_size = 50.0;

    let get_text_node = |text: String| {
        (
            Text::new(text),
            TextFont {
                font: font.clone(),
                font_size: font_size.clone(),
                ..default()
            },
            TextColor(Color::srgb(0.9, 0.9, 0.9)),
        )
    };

    let level_selector_rect = Rect::new(0., 0., 128., 128.);

    let button = (
        Button,
        Node {
            width: Val::Px(96.),
            height: Val::Px(96.),
            border: UiRect::all(Val::Px(5.0)),
            // horizontally center child text
            justify_content: JustifyContent::Center,
            // vertically center child text
            align_items: AlignItems::Center,
            padding: UiRect::all(Val::Px(5.)),
            ..default()
        },
        BorderColor(Color::BLACK),
        BorderRadius::MAX,
        BackgroundColor(NORMAL_BUTTON),
    );

    let side_bar_node = Node {
        width: Val::Px(96.),
        height: Val::Percent(100.),
        flex_direction: FlexDirection::Column,
        justify_content: JustifyContent::SpaceBetween,
        ..default()
    };

    let level_selector_node = (ImageNode::new(
        asset_server.load("sprites/symbols_sprite_sheet.png"),
    )
    .with_rect(level_selector_rect),);

    commands
        .spawn(Node {
            width: Val::Percent(100.0),
            height: Val::Percent(100.0),
            justify_content: JustifyContent::SpaceBetween,
            ..default()
        })
        .insert(PickingBehavior::IGNORE)
        .with_children(|parent| {
            parent.spawn(side_bar_node.clone()).with_children(|parent| {
                parent
                    .spawn(button.clone())
                    .insert(ReplayLevelButton)
                    .with_child(get_text_node("↻".to_string()));

                parent
                    .spawn(button.clone())
                    .insert(PreviousLevelButton)
                    .with_child(get_text_node("←".to_string()));
            });

            parent.spawn(side_bar_node).with_children(|parent| {
                parent
                    .spawn(button.clone())
                    .insert(LevelSelectorButton)
                    .with_child((
                        Node {
                            width: Val::Percent(85.),
                            height: Val::Percent(85.),
                            ..default()
                        },
                        level_selector_node,
                    ));

                parent
                    .spawn(button)
                    .insert(NextLevelButton)
                    .with_child(get_text_node("→".to_string()));
            });
        });
}

pub fn spawn_level_complete_ui(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    save_data_query: Query<&SaveData>,
    player_path_resource: Res<PlayerPath>,
    solution_component: Query<&SolutionComponent>,
) {
    let Ok(SolutionComponent(solution)) = solution_component.get_single() else {
        return;
    };

    let max_level = LEVELS.len();
    let save_data = save_data_query.single();
    let PlayerPath(path) = player_path_resource.into_inner();
    let path_length = path.len();

    let solution_length = solution.len();
    let solution_text = format!(
        "Level {}\nScore {}\nSolution {}",
        save_data.current_index + 1,
        path_length,
        solution_length
    );

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
                    width: Val::Px(384.),
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
                                height: Val::Px(96.),
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
                                ))
                                .with_child((
                                    Text::new(solution_text),
                                    TextFont {
                                        font: asset_server.load(FONT_PATH),
                                        font_size: 12.0,
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
    mut save_data_query: Query<&mut SaveData>,
    mut play_state: ResMut<NextState<PlayState>>,
) {
    let Ok(mut save_data) = save_data_query.get_single_mut() else {
        return;
    };

    let Ok(interaction) = interaction_query.get_single() else {
        return;
    };

    if *interaction == Interaction::Pressed && save_data.current_index > 0 {
        println!("previous level");
        save_data.current_index -= 1;
        play_state.set(PlayState::Loading);
    }
}

pub fn replay_level(
    interaction_query: Query<
        &Interaction,
        (Changed<Interaction>, With<Button>, With<ReplayLevelButton>),
    >,
    mut play_state: ResMut<NextState<PlayState>>,
) {
    let Ok(interaction) = interaction_query.get_single() else {
        return;
    };

    if *interaction == Interaction::Pressed {
        println!("replay level");
        play_state.set(PlayState::Loading);
    }
}

pub fn next_level(
    interaction_query: Query<
        &Interaction,
        (Changed<Interaction>, With<Button>, With<NextLevelButton>),
    >,
    mut save_data_query: Query<&mut SaveData>,
    mut play_state: ResMut<NextState<PlayState>>,
) {
    let Ok(mut save_data) = save_data_query.get_single_mut() else {
        return;
    };

    let Ok(interaction) = interaction_query.get_single() else {
        return;
    };

    if *interaction == Interaction::Pressed && save_data.current_index < LEVELS.len() - 1 {
        println!("next level");

        save_data.current_index += 1;
        play_state.set(PlayState::Loading);
    }
}

pub fn level_selector(
    interaction_query: Query<
        &Interaction,
        (
            Changed<Interaction>,
            With<Button>,
            With<LevelSelectorButton>,
        ),
    >,
    mut game_state: ResMut<NextState<GameState>>,
) {
    let Ok(interaction) = interaction_query.get_single() else {
        return;
    };

    if *interaction == Interaction::Pressed {
        println!("next level");

        game_state.set(GameState::Selector);
    }
}
