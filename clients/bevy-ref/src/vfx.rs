//! GPU particle VFX for Civis game events (wildfire, splash, dust, bless, disaster).
//!
//! WRAP > HANDROLL: this module wraps [`bevy_hanabi`] (the de-facto
//! Niagara-equivalent GPU particle system for Bevy) rather than hand-rolling a
//! particle simulator. Compatibility was verified on 2026-05-30 via crates.io
//! and the upstream README compat table:
//!
//! | bevy_hanabi | bevy |
//! |-------------|------|
//! | 0.18.0 (2026-02-01) | 0.18 |
//!
//! bevy_hanabi 0.18.0 targets Bevy 0.18, matching this client's `bevy = "0.18"`
//! pin, so the real GPU path is used (no billboard/quad fallback was needed).
//!
//! # Usage
//!
//! The whole module is gated behind the `vfx` cargo feature (which implies
//! `bevy`). Register the plugin and fire effects via an event so any system can
//! trigger VFX without touching [`Assets`] at the call site:
//!
//! ```ignore
//! # use bevy::prelude::*;
//! # use civ_bevy_ref::vfx::{VfxPlugin, SpawnEffect, EffectKind};
//! app.add_plugins(VfxPlugin);
//!
//! // From any system:
//! fn on_meteor(mut ev: EventWriter<SpawnEffect>) {
//!     ev.write(SpawnEffect { kind: EffectKind::Explosion, pos: Vec3::ZERO });
//! }
//! ```
//!
//! A free helper [`spawn_effect`] is also exposed for systems that already hold
//! an [`EventWriter<SpawnEffect>`].

#![cfg(feature = "vfx")]

use bevy::prelude::*;
use bevy_hanabi::prelude::*;

/// Kinds of one-shot particle effects keyed to Civis game events.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum EffectKind {
    /// Fire + smoke for wildfire / combustion.
    Fire,
    /// Water splash for entities entering water / rain impacts.
    Splash,
    /// Dust burst for construction and earthquakes.
    Dust,
    /// Sparkle / shimmer for a bless / blessing.
    Sparkle,
    /// Bright explosion burst for meteors and disasters.
    Explosion,
}

impl EffectKind {
    /// All kinds, used to pre-bake one [`EffectAsset`] per kind at startup.
    const ALL: [EffectKind; 5] = [
        EffectKind::Fire,
        EffectKind::Splash,
        EffectKind::Dust,
        EffectKind::Sparkle,
        EffectKind::Explosion,
    ];
}

/// Event requesting a one-shot particle effect of `kind` at world `pos`.
#[derive(Event, Debug, Clone, Copy)]
pub struct SpawnEffect {
    /// Which effect to play.
    pub kind: EffectKind,
    /// World-space position to play it at.
    pub pos: Vec3,
}

/// Convenience helper: enqueue a one-shot effect from any system that holds an
/// [`EventWriter<SpawnEffect>`].
///
/// ```ignore
/// fn sys(mut w: EventWriter<SpawnEffect>) {
///     spawn_effect(&mut w, EffectKind::Splash, Vec3::new(10.0, 0.0, 4.0));
/// }
/// ```
pub fn spawn_effect(writer: &mut EventWriter<SpawnEffect>, kind: EffectKind, pos: Vec3) {
    writer.write(SpawnEffect { kind, pos });
}

/// Handles for the pre-baked effect assets, one per [`EffectKind`].
#[derive(Resource, Default)]
struct VfxAssets {
    fire: Handle<EffectAsset>,
    splash: Handle<EffectAsset>,
    dust: Handle<EffectAsset>,
    sparkle: Handle<EffectAsset>,
    explosion: Handle<EffectAsset>,
}

impl VfxAssets {
    fn handle(&self, kind: EffectKind) -> Handle<EffectAsset> {
        match kind {
            EffectKind::Fire => self.fire.clone(),
            EffectKind::Splash => self.splash.clone(),
            EffectKind::Dust => self.dust.clone(),
            EffectKind::Sparkle => self.sparkle.clone(),
            EffectKind::Explosion => self.explosion.clone(),
        }
    }
}

/// Lifetime tracker so one-shot (burst) effect entities despawn after they have
/// finished emitting + their particles have died.
#[derive(Component)]
struct VfxLifetime(Timer);

/// Plugin registering reusable GPU particle effects for game events.
///
/// Adds [`bevy_hanabi::HanabiPlugin`], the [`SpawnEffect`] event, and the
/// systems that (a) bake one effect asset per kind at startup and (b) spawn +
/// reap one-shot effect entities in response to events.
pub struct VfxPlugin;

impl Plugin for VfxPlugin {
    fn build(&self, app: &mut App) {
        if !app.is_plugin_added::<HanabiPlugin>() {
            app.add_plugins(HanabiPlugin);
        }
        app.init_resource::<VfxAssets>()
            .add_event::<SpawnEffect>()
            .add_systems(Startup, bake_effects)
            .add_systems(Update, (handle_spawn_events, reap_finished));
    }
}

/// Bake one [`EffectAsset`] per [`EffectKind`] once, at startup.
fn bake_effects(mut assets: ResMut<VfxAssets>, mut effects: ResMut<Assets<EffectAsset>>) {
    for kind in EffectKind::ALL {
        let asset = effects.add(build_effect(kind));
        match kind {
            EffectKind::Fire => assets.fire = asset,
            EffectKind::Splash => assets.splash = asset,
            EffectKind::Dust => assets.dust = asset,
            EffectKind::Sparkle => assets.sparkle = asset,
            EffectKind::Explosion => assets.explosion = asset,
        }
    }
}

/// Spawn a fresh particle-effect entity for each requested event.
fn handle_spawn_events(
    mut commands: Commands,
    mut events: EventReader<SpawnEffect>,
    assets: Res<VfxAssets>,
) {
    for ev in events.read() {
        commands.spawn((
            Name::new("vfx-oneshot"),
            ParticleEffect::new(assets.handle(ev.kind)),
            Transform::from_translation(ev.pos),
            VfxLifetime(Timer::from_seconds(
                effect_ttl_secs(ev.kind),
                TimerMode::Once,
            )),
        ));
    }
}

/// Despawn one-shot effect entities once their lifetime elapses.
fn reap_finished(
    mut commands: Commands,
    time: Res<Time>,
    mut q: Query<(Entity, &mut VfxLifetime)>,
) {
    for (e, mut life) in &mut q {
        if life.0.tick(time.delta()).finished() {
            commands.entity(e).despawn();
        }
    }
}

/// How long to keep a one-shot effect entity alive (emit window + max particle
/// lifetime + slack) before despawning it.
fn effect_ttl_secs(kind: EffectKind) -> f32 {
    match kind {
        EffectKind::Fire => 3.5,
        EffectKind::Splash => 1.5,
        EffectKind::Dust => 2.0,
        EffectKind::Sparkle => 2.5,
        EffectKind::Explosion => 2.5,
    }
}

// ---------------------------------------------------------------------------
// Effect definitions. Each returns an EffectAsset tuned for its game event.
// ---------------------------------------------------------------------------

/// Dispatch to the per-kind effect builder.
fn build_effect(kind: EffectKind) -> EffectAsset {
    match kind {
        EffectKind::Fire => fire_effect(),
        EffectKind::Splash => splash_effect(),
        EffectKind::Dust => dust_effect(),
        EffectKind::Sparkle => sparkle_effect(),
        EffectKind::Explosion => explosion_effect(),
    }
}

/// Small helper: a constant-color gradient that fades alpha 1 -> 0.
fn fade_gradient(rgb: Vec3) -> Gradient<Vec4> {
    let mut g = Gradient::new();
    g.add_key(0.0, rgb.extend(1.0));
    g.add_key(0.7, rgb.extend(0.8));
    g.add_key(1.0, rgb.extend(0.0));
    g
}

/// Fire + smoke: warm particles rising with buoyant (upward) acceleration.
fn fire_effect() -> EffectAsset {
    let writer = ExprWriter::new();

    let init_pos = SetPositionSphereModifier {
        center: writer.lit(Vec3::ZERO).expr(),
        radius: writer.lit(0.6).expr(),
        dimension: ShapeDimension::Volume,
    };

    // Mostly-upward velocity with some spread.
    let init_vel = SetVelocitySphereModifier {
        center: writer.lit(Vec3::new(0.0, -2.0, 0.0)).expr(),
        speed: writer.lit(2.0).uniform(writer.lit(4.0)).expr(),
    };

    let init_age = SetAttributeModifier::new(Attribute::AGE, writer.lit(0.).expr());
    let init_lifetime = SetAttributeModifier::new(
        Attribute::LIFETIME,
        writer.lit(0.8).uniform(writer.lit(1.6)).expr(),
    );

    // Buoyancy: hot air rises.
    let update_accel = AccelModifier::new(writer.lit(Vec3::new(0.0, 6.0, 0.0)).expr());
    let update_drag = LinearDragModifier::new(writer.lit(1.5).expr());

    let mut color = Gradient::new();
    color.add_key(0.0, Vec4::new(6.0, 4.0, 1.0, 1.0)); // bright orange (HDR for bloom)
    color.add_key(0.4, Vec4::new(3.0, 1.0, 0.2, 0.9)); // deep red
    color.add_key(0.8, Vec4::new(0.3, 0.3, 0.3, 0.4)); // smoke grey
    color.add_key(1.0, Vec4::new(0.1, 0.1, 0.1, 0.0)); // fade out

    let mut size = Gradient::new();
    size.add_key(0.0, Vec3::splat(0.3));
    size.add_key(1.0, Vec3::splat(0.9));

    EffectAsset::new(2048, SpawnerSettings::rate(80.0.into()), writer.finish())
        .with_name("vfx_fire")
        .init(init_pos)
        .init(init_vel)
        .init(init_age)
        .init(init_lifetime)
        .update(update_accel)
        .update(update_drag)
        .render(ColorOverLifetimeModifier {
            gradient: color,
            blend: ColorBlendMode::Overwrite,
            mask: ColorBlendMask::RGBA,
        })
        .render(SizeOverLifetimeModifier {
            gradient: size,
            screen_space_size: false,
        })
}

/// Water splash: a quick upward+outward burst that arcs back down under gravity.
fn splash_effect() -> EffectAsset {
    let writer = ExprWriter::new();

    let init_pos = SetPositionSphereModifier {
        center: writer.lit(Vec3::ZERO).expr(),
        radius: writer.lit(0.3).expr(),
        dimension: ShapeDimension::Surface,
    };

    let init_vel = SetVelocitySphereModifier {
        center: writer.lit(Vec3::new(0.0, -4.0, 0.0)).expr(),
        speed: writer.lit(4.0).uniform(writer.lit(7.0)).expr(),
    };

    let init_age = SetAttributeModifier::new(Attribute::AGE, writer.lit(0.).expr());
    let init_lifetime = SetAttributeModifier::new(
        Attribute::LIFETIME,
        writer.lit(0.5).uniform(writer.lit(1.0)).expr(),
    );

    // Real gravity pulls droplets back down.
    let update_accel = AccelModifier::new(writer.lit(Vec3::new(0.0, -16.0, 0.0)).expr());

    let mut size = Gradient::new();
    size.add_key(0.0, Vec3::splat(0.15));
    size.add_key(1.0, Vec3::splat(0.05));

    EffectAsset::new(
        1024,
        // Burst: ~120 particles up front, then stop.
        SpawnerSettings::once(120.0.into()),
        writer.finish(),
    )
    .with_name("vfx_splash")
    .init(init_pos)
    .init(init_vel)
    .init(init_age)
    .init(init_lifetime)
    .update(update_accel)
    .render(ColorOverLifetimeModifier {
        gradient: fade_gradient(Vec3::new(0.6, 0.8, 1.0)),
        blend: ColorBlendMode::Overwrite,
        mask: ColorBlendMask::RGBA,
    })
    .render(SizeOverLifetimeModifier {
        gradient: size,
        screen_space_size: false,
    })
}

/// Dust: a slow, low, spreading puff for building placement and quakes.
fn dust_effect() -> EffectAsset {
    let writer = ExprWriter::new();

    let init_pos = SetPositionSphereModifier {
        center: writer.lit(Vec3::ZERO).expr(),
        radius: writer.lit(0.8).expr(),
        dimension: ShapeDimension::Volume,
    };

    let init_vel = SetVelocitySphereModifier {
        center: writer.lit(Vec3::new(0.0, -0.5, 0.0)).expr(),
        speed: writer.lit(1.0).uniform(writer.lit(2.5)).expr(),
    };

    let init_age = SetAttributeModifier::new(Attribute::AGE, writer.lit(0.).expr());
    let init_lifetime = SetAttributeModifier::new(
        Attribute::LIFETIME,
        writer.lit(1.0).uniform(writer.lit(1.8)).expr(),
    );

    let update_drag = LinearDragModifier::new(writer.lit(2.5).expr());

    let mut size = Gradient::new();
    size.add_key(0.0, Vec3::splat(0.4));
    size.add_key(1.0, Vec3::splat(1.2));

    EffectAsset::new(1024, SpawnerSettings::once(150.0.into()), writer.finish())
        .with_name("vfx_dust")
        .init(init_pos)
        .init(init_vel)
        .init(init_age)
        .init(init_lifetime)
        .update(update_drag)
        .render(ColorOverLifetimeModifier {
            gradient: fade_gradient(Vec3::new(0.7, 0.62, 0.5)),
            blend: ColorBlendMode::Overwrite,
            mask: ColorBlendMask::RGBA,
        })
        .render(SizeOverLifetimeModifier {
            gradient: size,
            screen_space_size: false,
        })
}

/// Sparkle: bright golden shimmer rising gently for a bless.
fn sparkle_effect() -> EffectAsset {
    let writer = ExprWriter::new();

    let init_pos = SetPositionSphereModifier {
        center: writer.lit(Vec3::ZERO).expr(),
        radius: writer.lit(1.0).expr(),
        dimension: ShapeDimension::Volume,
    };

    let init_vel = SetVelocitySphereModifier {
        center: writer.lit(Vec3::new(0.0, -3.0, 0.0)).expr(),
        speed: writer.lit(1.0).uniform(writer.lit(2.0)).expr(),
    };

    let init_age = SetAttributeModifier::new(Attribute::AGE, writer.lit(0.).expr());
    let init_lifetime = SetAttributeModifier::new(
        Attribute::LIFETIME,
        writer.lit(1.2).uniform(writer.lit(2.0)).expr(),
    );

    // Gentle upward float.
    let update_accel = AccelModifier::new(writer.lit(Vec3::new(0.0, 2.0, 0.0)).expr());

    let mut color = Gradient::new();
    color.add_key(0.0, Vec4::new(5.0, 4.0, 1.5, 0.0)); // fade in (HDR gold)
    color.add_key(0.2, Vec4::new(5.0, 4.0, 1.5, 1.0));
    color.add_key(0.8, Vec4::new(4.0, 3.0, 1.0, 0.8));
    color.add_key(1.0, Vec4::new(2.0, 1.5, 0.5, 0.0));

    let mut size = Gradient::new();
    size.add_key(0.0, Vec3::splat(0.05));
    size.add_key(0.5, Vec3::splat(0.18));
    size.add_key(1.0, Vec3::splat(0.02));

    EffectAsset::new(1024, SpawnerSettings::rate(60.0.into()), writer.finish())
        .with_name("vfx_sparkle")
        .init(init_pos)
        .init(init_vel)
        .init(init_age)
        .init(init_lifetime)
        .update(update_accel)
        .render(ColorOverLifetimeModifier {
            gradient: color,
            blend: ColorBlendMode::Overwrite,
            mask: ColorBlendMask::RGBA,
        })
        .render(SizeOverLifetimeModifier {
            gradient: size,
            screen_space_size: false,
        })
}

/// Explosion: a single bright omnidirectional burst with drag + gravity, for
/// meteors and disasters.
fn explosion_effect() -> EffectAsset {
    let writer = ExprWriter::new();

    let init_pos = SetPositionSphereModifier {
        center: writer.lit(Vec3::ZERO).expr(),
        radius: writer.lit(0.5).expr(),
        dimension: ShapeDimension::Volume,
    };

    let init_vel = SetVelocitySphereModifier {
        center: writer.lit(Vec3::ZERO).expr(),
        speed: writer.lit(20.0).uniform(writer.lit(45.0)).expr(),
    };

    let init_age = SetAttributeModifier::new(Attribute::AGE, writer.lit(0.).expr());
    let init_lifetime = SetAttributeModifier::new(
        Attribute::LIFETIME,
        writer.lit(0.6).uniform(writer.lit(1.2)).expr(),
    );

    let update_drag = LinearDragModifier::new(writer.lit(5.0).expr());
    let update_accel = AccelModifier::new(writer.lit(Vec3::new(0.0, -12.0, 0.0)).expr());

    let mut color = Gradient::new();
    color.add_key(0.0, Vec4::new(8.0, 6.0, 2.0, 1.0)); // white-hot (HDR)
    color.add_key(0.3, Vec4::new(6.0, 2.0, 0.4, 1.0)); // orange
    color.add_key(0.7, Vec4::new(1.5, 0.4, 0.2, 0.7)); // red embers
    color.add_key(1.0, Vec4::new(0.2, 0.2, 0.2, 0.0)); // smoke fade

    let mut size = Gradient::new();
    size.add_key(0.0, Vec3::splat(0.5));
    size.add_key(1.0, Vec3::splat(0.1));

    EffectAsset::new(8192, SpawnerSettings::once(800.0.into()), writer.finish())
        .with_name("vfx_explosion")
        .init(init_pos)
        .init(init_vel)
        .init(init_age)
        .init(init_lifetime)
        .update(update_drag)
        .update(update_accel)
        .render(ColorOverLifetimeModifier {
            gradient: color,
            blend: ColorBlendMode::Overwrite,
            mask: ColorBlendMask::RGBA,
        })
        .render(SizeOverLifetimeModifier {
            gradient: size,
            screen_space_size: false,
        })
}
