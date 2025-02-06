use bevy::{
    pbr::ExtendedMaterial,
    prelude::*,
    ui::widget::{ImageNodeSize, NodeImageMode},
};

use crate::{
    assets::shaders::PulsingShader,
    game_settings::GameSettings,
    level_selector::coordinate_to_symbol_mesh,
    levels::{GameLevel, LevelData, Shape},
};

#[derive(Component, Debug, Clone)]
pub struct LevelCompleteBadge;

#[derive(Component, Debug, Clone)]
pub struct Fade {
    timer: Timer,
    fading_in: bool,
}

impl Fade {
    fn fade_in() -> Self {
        Self {
            timer: Timer::from_seconds(0.3, TimerMode::Once),
            fading_in: true,
        }
    }

    fn fade_out() -> Self {
        Self {
            timer: Timer::from_seconds(1.0, TimerMode::Once),
            fading_in: false,
        }
    }
}

const FONT_PATH: &str = "fonts/Slimamifbold.ttf";

pub fn spawn(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    game_settings: Res<GameSettings>,
    level_query: Query<&GameLevel>,
) {
    let level = level_query.single();

    let symbol_rect_slice = match level.shape {
        Shape::Tetrahedron => Rect::new(512., 0., 640., 128.),
        Shape::Cube => Rect::new(384., 0., 512., 128.),
        Shape::Octahedron => Rect::new(256., 0., 384., 128.),
        Shape::Dodecahedron => Rect::new(128., 0., 256., 128.),
        Shape::Icosahedron => Rect::new(0., 0., 128., 128.),
    };

    let font = asset_server.load(FONT_PATH);
    let font_size = 28.0;

    let text_node = commands
        .spawn((
            Text::new("Score"),
            TextFont {
                font: font.clone(),
                font_size: font_size.clone(),
                ..default()
            },
            TextColor(game_settings.palette.line_color.clone()),
            Fade::fade_in(),
        ))
        .id();

    let text_container_node = commands
        .spawn(Node {
            width: Val::Percent(80.0),
            height: Val::Percent(80.0),
            justify_content: JustifyContent::Center,
            align_items: AlignItems::Center,
            border: UiRect::all(Val::Px(10.)),
            ..default()
        })
        .add_child(text_node)
        .id();

    let symbol_node = commands
        .spawn((
            ImageNode {
                image: asset_server.load("sprites/symbols_sprite_sheet.png"),
                color: game_settings.palette.line_color.clone(),
                rect: Some(symbol_rect_slice),
                ..default()
            },
            Node {
                width: Val::Percent(30.0),
                min_width: Val::Px(256.0),
                aspect_ratio: Some(1.0),
                justify_content: JustifyContent::Center,
                align_items: AlignItems::Center,
                ..default()
            },
            Fade::fade_in(),
        ))
        .add_child(text_container_node)
        .id();

    commands
        .spawn((
            Node {
                width: Val::Percent(100.0),
                height: Val::Percent(100.0),
                justify_content: JustifyContent::Center,
                align_items: AlignItems::Center,
                border: UiRect::all(Val::Px(10.)),
                ..default()
            },
            PickingBehavior::IGNORE,
            LevelData,
        ))
        .add_child(symbol_node);
}

pub fn fade_system(
    mut commands: Commands,
    time: Res<Time>,
    mut image_node_query: Query<(&mut ImageNode, &mut Fade, &Parent)>,
    mut text_color_query: Query<(&mut TextColor, &mut Fade), Without<ImageNode>>,
) {
    for (mut image_node, mut fade, parent) in image_node_query.iter_mut() {
        fade.timer.tick(time.delta());
        let progress = fade.timer.fraction();

        let alpha = if fade.fading_in {
            progress // Fade-in increases alpha
        } else {
            1.0 - progress // Fade-out decreases alpha
        };

        image_node.color.set_alpha(alpha);

        if alpha < 0.01 {
            println!("despawn");
            let parent_entity = parent.get();
            commands.entity(parent_entity).despawn_recursive();
        }
    }

    for (mut text_color_node, mut fade) in text_color_query.iter_mut() {
        fade.timer.tick(time.delta());
        let progress = fade.timer.fraction();

        let alpha = if fade.fading_in {
            progress // Fade-in increases alpha
        } else {
            1.0 - progress // Fade-out decreases alpha
        };

        text_color_node.0.set_alpha(alpha);
    }
}

pub fn trigger_fade_out(mut commands: Commands, fade_query: Query<Entity, With<Fade>>) {
    for fade in fade_query.iter() {
        commands.entity(fade).insert(Fade::fade_out());
    }
}
