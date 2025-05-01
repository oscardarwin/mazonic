use bevy::{
    prelude::*,
    ui::widget::{ImageNodeSize, NodeImageMode},
};

use crate::{
    constants::{FONT_PATH, TEXT_COLOR, TRANSPARENCY}, game_save::{CurrentPuzzle, GameSave, PuzzleIdentifier, WorkingLevelIndex}, game_state::{GameState, PlayState}, levels::LEVELS, shape::loader::{GraphComponent, SolutionComponent}, statistics::PlayerPath
};

#[derive(Component)]
pub struct NavigationUI;

#[derive(Component)]
pub struct PreviousLevelButton;

#[derive(Component)]
pub struct ReplayLevelButton;

#[derive(Component)]
pub struct NextLevelButton;

#[derive(Component)]
pub struct LevelSelectorButton;

const NORMAL_BUTTON: Color = Color::srgba(0.15, 0.15, 0.15, TRANSPARENCY);
const HOVERED_BUTTON: Color = Color::srgba(0.25, 0.25, 0.25, TRANSPARENCY);
const PRESSED_BUTTON: Color = Color::srgba(0.65, 0.65, 0.65, TRANSPARENCY);
const BUTTON_BACKGROUND_COLOR: Color = Color::srgba(0.1, 0.1, 0.1, TRANSPARENCY);
const PRESSED_BUTTON_BORDER_COLOR: Color = Color::srgba(0.9, 0.9, 0.9, TRANSPARENCY);
const FONT_SIZE: f32 = 50.0;


pub fn spawn(mut commands: Commands, asset_server: Res<AssetServer>) {
    let font = asset_server.load(FONT_PATH);

    let get_text_node = |text: &str| {
        (
            Text::new(text),
            TextFont {
                font: font.clone(),
                font_size: FONT_SIZE,
                ..default()
            },
            TextColor(TEXT_COLOR),
        )
    };

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
        BorderColor(BUTTON_BACKGROUND_COLOR),
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

    let selector_symbol_pixel_width = 512.;
    let level_selector_rect = Rect::new(
        0.,
        selector_symbol_pixel_width,
        selector_symbol_pixel_width,
        2.0 * selector_symbol_pixel_width,
    );
    let level_selector_node = (ImageNode {
        image: asset_server.load("sprites/symbols_sprite_sheet.png"),
        color: PRESSED_BUTTON_BORDER_COLOR,
        ..Default::default()
    }
    .with_rect(level_selector_rect),);

    commands
        .spawn(Node {
            width: Val::Percent(100.0),
            height: Val::Percent(100.0),
            justify_content: JustifyContent::SpaceBetween,
            border: UiRect::all(Val::Px(10.)),
            ..default()
        })
        .insert(NavigationUI)
        .insert(PickingBehavior::IGNORE)
        .with_children(|parent| {
            parent.spawn(side_bar_node.clone()).with_children(|parent| {
                parent
                    .spawn(button.clone())
                    .insert(ReplayLevelButton)
                    .with_child(get_text_node("↻"));

                parent
                    .spawn(button.clone())
                    .insert(PreviousLevelButton)
                    .with_child(get_text_node("←"));
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
                    .with_child(get_text_node("→"));
            });
        });
}

pub fn despawn_level_navigation_ui(mut commands: Commands, ui_entities: Query<Entity, With<NavigationUI>>) {
    for entity in ui_entities.iter() {
        commands.entity(entity).despawn_recursive();
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
                border_color.0 = PRESSED_BUTTON_BORDER_COLOR.into();
            }
            Interaction::Hovered => {
                *color = HOVERED_BUTTON.into();
                border_color.0 = PRESSED_BUTTON_BORDER_COLOR.into();
            }
            Interaction::None => {
                *color = NORMAL_BUTTON.into();
                border_color.0 = PRESSED_BUTTON_BORDER_COLOR.into();
            }
        }
    }
}

pub fn update_previous_level_button_visibility(
    mut previous_level_button_query: Query<&mut Visibility, With<PreviousLevelButton>>,
    current_level_index_query: Query<&CurrentPuzzle>,
) {
    let Ok(CurrentPuzzle(puzzle_identifier)) = current_level_index_query.get_single() else {
        return;
    };

    let Ok(mut previous_level_button_visibility) = previous_level_button_query.get_single_mut()
    else {
        return;
    };

    *previous_level_button_visibility = match puzzle_identifier {
        PuzzleIdentifier::Level(level_index) if *level_index > 0 => Visibility::Visible,
        _ => Visibility::Hidden,
        
    }
}

pub fn update_next_level_button_visibility(
    mut next_level_button_query: Query<&mut Visibility, With<NextLevelButton>>,
    current_level_index_query: Query<&CurrentPuzzle>,
    working_level_index_query: Query<&WorkingLevelIndex>,
) {
    let Ok(CurrentPuzzle(puzzle_identifier)) = current_level_index_query.get_single() else {
        return;
    };

    let Ok(mut next_level_button_visibility) = next_level_button_query.get_single_mut() else {
        return;
    };

    let Ok(WorkingLevelIndex(working_level_index)) = working_level_index_query.get_single() else {
        return;
    };

    let max_level_index = LEVELS.len() - 1;

    *next_level_button_visibility = match puzzle_identifier {
        PuzzleIdentifier::Level(level_index) if level_index < working_level_index && *level_index < max_level_index => Visibility::Visible,
        _ => Visibility::Hidden,
        
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
    mut current_level_index_query: Query<&mut CurrentPuzzle>,
    mut play_state: ResMut<NextState<PlayState>>,
) {
    let Ok(mut current_puzzle) = current_level_index_query.get_single_mut() else {
        return;
    };

    let Ok(interaction) = interaction_query.get_single() else {
        return;
    };
    
    let CurrentPuzzle(PuzzleIdentifier::Level(mut current_level_index)) = *current_puzzle else {
        return;
    };


    if *interaction == Interaction::Pressed && current_level_index > 0 {
        println!("previous level");
        *current_puzzle = CurrentPuzzle(PuzzleIdentifier::Level(current_level_index - 1));

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
    mut current_level_index_query: Query<&mut CurrentPuzzle>,
    mut play_state: ResMut<NextState<PlayState>>,
) {
    let Ok(mut current_puzzle) = current_level_index_query.get_single_mut() else {
        return;
    };

    let Ok(interaction) = interaction_query.get_single() else {
        return;
    };

    let CurrentPuzzle(PuzzleIdentifier::Level(current_level_index)) = *current_puzzle else {
        return;
    };

    if *interaction == Interaction::Pressed && current_level_index < LEVELS.len() - 1 {
        println!("next level");

        *current_puzzle  = CurrentPuzzle(PuzzleIdentifier::Level(current_level_index + 1));
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
