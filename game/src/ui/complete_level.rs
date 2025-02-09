use bevy::{
    pbr::ExtendedMaterial,
    prelude::*,
    ui::widget::{ImageNodeSize, NodeImageMode},
};

use crate::{
    assets::shaders::{FlashUiMaterial, PulsingShader},
    constants::SYMBOL_TEXTURE_DIMENSIONS,
    game_settings::GameSettings,
    level_selector::coordinate_to_symbol_mesh,
    levels::{GameLevel, LevelData, Shape},
    statistics::PlayerPath,
};

#[derive(Component, Debug, Clone)]
pub struct LevelCompleteBadge;

#[derive(Component, Debug, Clone)]
pub struct FadeOut {
    timer: Timer,
}
impl FadeOut {
    fn new() -> Self {
        Self {
            timer: Timer::from_seconds(1.2, TimerMode::Once),
        }
    }
}

#[derive(Component, Debug, Clone)]
pub struct FadeIn {
    timer: Timer,
}
impl FadeIn {
    fn new() -> Self {
        Self {
            timer: Timer::from_seconds(0.3, TimerMode::Once),
        }
    }
}

#[derive(Component, Debug, Clone)]
pub struct Fadeable {
    pub max_alpha: f32,
}

#[derive(Component, Debug, Clone)]
pub struct RootNode(pub Entity);

#[derive(Component, Debug, Clone)]
pub struct ExpandEffect {
    pub delay: f32,
    pub timer: Timer,
}

impl ExpandEffect {
    pub fn new(delay: f32) -> Self {
        Self {
            delay,
            timer: Timer::from_seconds(5.0, TimerMode::Once),
        }
    }
}

const FONT_PATH: &str = "fonts/Slimamifbold.ttf";

pub fn spawn(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    game_settings: Res<GameSettings>,
    level_query: Query<&GameLevel>,
    player_path_query: Query<&PlayerPath>,
    mut ui_materials: ResMut<Assets<FlashUiMaterial>>,
) {
    let level = level_query.single();

    let symbol_pixel_width = 512.;
    let symbol_rect_position = match level.shape {
        Shape::Tetrahedron => 4,
        Shape::Cube => 3,
        Shape::Octahedron => 2,
        Shape::Dodecahedron => 1,
        Shape::Icosahedron => 0,
    } as f32;

    let symbol_rect_slice = Rect::new(
        symbol_rect_position * symbol_pixel_width,
        symbol_pixel_width,
        (symbol_rect_position + 1.0) * symbol_pixel_width,
        2.0 * symbol_pixel_width,
    );

    let start_uv = Vec2::new(symbol_rect_position, 1.0) / SYMBOL_TEXTURE_DIMENSIONS;
    let end_uv = Vec2::new(symbol_rect_position + 1.0, 2.0) / SYMBOL_TEXTURE_DIMENSIONS;

    let symbol_background_rect_slice = Rect::new(
        symbol_rect_position * symbol_pixel_width,
        0.,
        (symbol_rect_position + 1.0) * symbol_pixel_width,
        symbol_pixel_width,
    );

    let font = asset_server.load(FONT_PATH);
    let font_size = 28.0;

    let mut root_node_commands = commands.spawn((
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
    ));

    let bright_line_color = game_settings.palette.line_color.to_linear().to_vec3() * 100.0;
    let text_node = commands
        .spawn((
            Text::new("Score"),
            TextFont {
                font: font.clone(),
                font_size: font_size.clone(),
                ..default()
            },
            TextColor(Color::LinearRgba(LinearRgba::from_vec3(bright_line_color))),
            FadeIn::new(),
            Fadeable { max_alpha: 1.0 },
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

    let image = asset_server.load("sprites/symbols_sprite_sheet.png");
    let symbol_node = commands
        .spawn((
            MaterialNode(ui_materials.add(FlashUiMaterial {
                color: game_settings.palette.line_color.to_linear().to_vec4(),
                color_texture: image.clone(),
                start_uv,
                end_uv,
            })),
            Node {
                width: Val::Percent(100.0),
                height: Val::Percent(100.0),
                justify_content: JustifyContent::Center,
                align_items: AlignItems::Center,
                ..default()
            },
            FadeIn::new(),
            Fadeable { max_alpha: 1.0 },
        ))
        .add_child(text_container_node)
        .id();

    let symbol_background_node = commands
        .spawn((
            ImageNode {
                image: image.clone(),
                color: game_settings.palette.line_color,
                rect: Some(symbol_background_rect_slice),
                ..default()
            },
            Node {
                width: Val::Px(512.),
                aspect_ratio: Some(1.0),
                justify_content: JustifyContent::Center,
                align_items: AlignItems::Center,
                ..default()
            },
            FadeIn::new(),
            Fadeable { max_alpha: 0.4 },
        ))
        .add_child(symbol_node)
        .id();

    let root_node = commands
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
        .add_child(symbol_background_node)
        .id();

    let text_entity = commands.entity(text_node).insert(RootNode(root_node));

    spawn_background_effect(
        &mut commands,
        game_settings.palette.line_color.clone(),
        0.3,
        image.clone(),
        symbol_background_rect_slice.clone(),
    );
    spawn_background_effect(
        &mut commands,
        game_settings.palette.line_color.clone(),
        0.2,
        image,
        symbol_background_rect_slice,
    );
}

fn spawn_background_effect(
    mut commands: &mut Commands,
    color: Color,
    delay: f32,
    image: Handle<Image>,
    rect_slice: Rect,
) {
    let symbol_background_expand_effect = commands
        .spawn((
            ImageNode {
                image,
                color,
                rect: Some(rect_slice),
                ..default()
            },
            Node {
                width: Val::Px(512.),
                aspect_ratio: Some(1.),
                justify_content: JustifyContent::Center,
                align_items: AlignItems::Center,
                ..default()
            },
            ExpandEffect::new(delay),
        ))
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
        .add_child(symbol_background_expand_effect);
}

pub fn fade_out_system(
    mut commands: Commands,
    time: Res<Time>,
    mut background_image_node_query: Query<(&mut ImageNode, &mut FadeOut, &Fadeable)>,
    mut text_color_query: Query<
        (&mut TextColor, &mut FadeOut, &Fadeable, &RootNode),
        Without<ImageNode>,
    >,
    mut symbol_node_query: Query<
        (&MaterialNode<FlashUiMaterial>, &mut FadeOut, &Fadeable),
        (Without<ImageNode>, Without<TextColor>),
    >,
    mut flash_ui_materials: ResMut<Assets<FlashUiMaterial>>,
) {
    for (mut image_node, mut fade, fadeable) in background_image_node_query.iter_mut() {
        fade.timer.tick(time.delta());
        let progress = fade.timer.fraction();
        let alpha = fadeable.max_alpha * (1.0 - progress);
        image_node.color.set_alpha(alpha);
    }

    for (MaterialNode(symbol_node_material_handle), mut fade, fadeable) in
        symbol_node_query.iter_mut()
    {
        fade.timer.tick(time.delta());
        let progress = fade.timer.fraction();
        let alpha = fadeable.max_alpha * (1.0 - progress);

        let mut symbol_node_material = flash_ui_materials
            .get_mut(symbol_node_material_handle)
            .unwrap();

        symbol_node_material.color = symbol_node_material.color.with_w(alpha);
    }

    for (mut text_color_node, mut fade, fadeable, RootNode(root_node)) in
        text_color_query.iter_mut()
    {
        fade.timer.tick(time.delta());
        let progress = fade.timer.fraction();

        let alpha = fadeable.max_alpha * (1.0 - progress);
        text_color_node.0.set_alpha(alpha);

        if progress > 0.99 {
            commands.entity(*root_node).despawn_recursive();
        }
    }
}
pub fn fade_in_system(
    mut commands: Commands,
    time: Res<Time>,
    mut image_node_query: Query<(Entity, &mut ImageNode, &mut FadeIn, &Fadeable)>,
    mut text_color_query: Query<
        (Entity, &mut TextColor, &mut FadeIn, &Fadeable),
        Without<ImageNode>,
    >,
) {
    for (entity, mut image_node, mut fade, fadeable) in image_node_query.iter_mut() {
        fade.timer.tick(time.delta());
        let progress = fade.timer.fraction();
        image_node.color.set_alpha(progress * fadeable.max_alpha);

        if progress > 0.99 {
            commands.entity(entity).remove::<FadeIn>();
        }
    }

    for (entity, mut text_color_node, mut fade, fadeable) in text_color_query.iter_mut() {
        fade.timer.tick(time.delta());
        let progress = fade.timer.fraction();
        text_color_node.0.set_alpha(progress * fadeable.max_alpha);

        if progress > 0.99 {
            commands.entity(entity).remove::<FadeIn>();
        }
    }
}

pub fn trigger_fade_out(mut commands: Commands, fade_query: Query<Entity, With<Fadeable>>) {
    for fade in fade_query.iter() {
        commands.entity(fade).insert(FadeOut::new());
    }
}

pub fn update_expand_effect(
    mut commands: Commands,
    mut expand_effect_query: Query<(Entity, &mut Node, &mut ImageNode, &mut ExpandEffect)>,
    time: Res<Time>,
) {
    for (entity, mut node, mut image_node, mut expand_effect) in expand_effect_query.iter_mut() {
        expand_effect.timer.tick(time.delta());
        let progress = expand_effect.timer.elapsed_secs() + expand_effect.delay;

        let alpha = progress * (-5.0 * progress).exp();
        image_node.color.set_alpha(alpha);

        let scaling_factor = (2.4 * progress).exp();
        node.width = Val::Px(512. * scaling_factor);
        node.height = Val::Px(512. * scaling_factor);

        if expand_effect.timer.finished() {
            commands.entity(entity).despawn();
        }
    }
}
