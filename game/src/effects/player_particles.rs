use bevy::prelude::*;
use bevy_hanabi::prelude::*;

use crate::game_settings::GameSettings;

#[derive(Component, Clone, Debug)]
pub struct PlayerParticlesHandle(pub Handle<EffectAsset>);

#[derive(Component, Clone, Debug)]
pub struct PlayerParticleEffect;

pub fn setup(
    mut effects: ResMut<Assets<EffectAsset>>,
    mut commands: Commands,
    game_settings: Res<GameSettings>,
) {
    let mut gradient = Gradient::new();
    let player_color = game_settings
        .palette
        .player_color
        .to_linear()
        .with_alpha(0.4)
        .to_vec4();
    let start_color = game_settings
        .palette
        .line_color
        .to_linear()
        .with_alpha(0.01)
        .to_vec4();
    gradient.add_key(0.0, start_color);
    gradient.add_key(1.0, player_color);

    let mut module = Module::default();

    let init_pos = SetPositionSphereModifier {
        center: module.lit(Vec3::ZERO),
        radius: module.lit(0.3),
        dimension: ShapeDimension::Surface,
    };

    let orient = OrientModifier {
        mode: OrientMode::ParallelCameraDepthPlane,
        rotation: None,
    };

    let init_vel = SetVelocitySphereModifier {
        center: module.lit(Vec3::ZERO),
        speed: module.lit(-0.1),
    };

    let lifetime = module.lit(2.0);
    let init_lifetime = SetAttributeModifier::new(Attribute::LIFETIME, lifetime);
    let init_size = SetAttributeModifier::new(Attribute::SIZE, module.lit(0.02));

    let effect = EffectAsset::new(32, Spawner::rate(2.0.into()), module)
        .with_name("PlayerParticles")
        .init(init_pos)
        .init(init_size)
        .init(init_vel)
        .init(init_lifetime)
        .render(orient)
        .with_simulation_condition(SimulationCondition::Always)
        .render(ColorOverLifetimeModifier { gradient });

    // Insert into the asset system
    let effect_handle = effects.add(effect);
    commands.spawn(PlayerParticlesHandle(effect_handle));
}

#[derive(Component, Clone, Debug)]
pub struct VisibilityTimer {
    timer: Timer,
}

pub fn turn_on_player_particles(mut commands: Commands) {
    commands.spawn(VisibilityTimer {
        timer: Timer::from_seconds(1.8, TimerMode::Once),
    });
}

pub fn update_player_particles(
    mut player_halo_query: Query<&mut Visibility, With<PlayerParticleEffect>>,
    mut commands: Commands,
    mut timer_query: Query<(Entity, &mut VisibilityTimer)>,
    time: Res<Time>,
) {
    let Ok((entity, mut visibility_timer)) = timer_query.get_single_mut() else {
        return;
    };

    let Ok(mut visibility) = player_halo_query.get_single_mut() else {
        return;
    };

    visibility_timer.timer.tick(time.delta());

    if visibility_timer.timer.just_finished() {
        *visibility = Visibility::Visible;
        commands.entity(entity).despawn();
    }
}

pub fn turn_off_player_particles(
    mut player_halo_query: Query<&mut Visibility, With<PlayerParticleEffect>>,
) {
    if let Ok(mut visibility) = player_halo_query.get_single_mut() {
        *visibility = Visibility::Hidden;
    }
}
