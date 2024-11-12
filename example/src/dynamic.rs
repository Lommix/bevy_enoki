/// ----------------------------------------------
/// dynamic example
/// how to update effect behavior dynamiclly
/// ----------------------------------------------
use bevy::{
    core_pipeline::bloom::{Bloom, BloomSettings},
    diagnostic::DiagnosticsStore,
    prelude::*,
};
use bevy_enoki::{prelude::*, EnokiPlugin};
use std::time::Duration;

fn main() {
    App::new()
        .add_plugins(DefaultPlugins.set(ImagePlugin {
            default_sampler: bevy::render::texture::ImageSamplerDescriptor::nearest(),
        }))
        .add_plugins(bevy::diagnostic::FrameTimeDiagnosticsPlugin)
        .add_plugins(EnokiPlugin)
        .add_systems(Startup, setup)
        .add_systems(Update, (show_fps, change_dynamic, move_camera))
        .run();
}

#[derive(Component)]
pub struct FpsText;

#[derive(Deref, Component, DerefMut)]
pub struct ChangeTimer(Timer);

#[derive(Deref, Component, DerefMut)]
pub struct Pcindex(f32);

#[derive(Deref, Resource, DerefMut)]
pub struct ParticleMaterialAsset(Handle<SpriteParticle2dMaterial>);

fn setup(
    mut cmd: Commands,
    mut materials: ResMut<Assets<SpriteParticle2dMaterial>>,
    server: Res<AssetServer>,
) {
    cmd.spawn((
        Camera2d,
        Camera {
            clear_color: ClearColorConfig::Custom(Color::BLACK),
            hdr: true,
            ..default()
        },
        Bloom {
            intensity: 0.1,
            ..default()
        },
    ));

    cmd.spawn((
        ChangeTimer(Timer::new(Duration::from_millis(300), TimerMode::Repeating)),
        Pcindex(0.),
    ));

    cmd.spawn((
        Text::default(),
        TextFont {
            font_size: 42.,
            ..default()
        },
        FpsText,
    ));

    cmd.spawn((ParticleSpawnerBundle {
        effect: server.load("base.particle.ron").into(),
        material: materials
            .add(SpriteParticle2dMaterial::new(
                server.load("enoki.png"),
                1,
                1,
            ))
            .into(),
        ..default()
    },));
}

fn change_dynamic(
    mut elapsed: Local<f32>,
    mut query: Query<&mut ParticleEffectInstance>,
    time: Res<Time>,
) {
    *elapsed += time.delta_secs();

    let Ok(mut maybe_effect) = query.get_single_mut() else {
        return;
    };

    if let Some(effect) = maybe_effect.0.as_mut() {
        effect.linear_speed = Some(Rval::new(1000. * elapsed.sin().abs(), 0.1));
        effect.spawn_amount = 100;
    }
}

fn show_fps(
    diagnostics: Res<DiagnosticsStore>,
    particles: Query<&ParticleStore>,
    mut texts: Query<&mut Text, With<FpsText>>,
) {
    let Some(fps) = diagnostics
        .get(&bevy::diagnostic::FrameTimeDiagnosticsPlugin::FPS)
        .map(|v| v.value())
        .flatten()
    else {
        return;
    };

    let particle_count: usize = particles.iter().map(|store| store.len()).sum();

    let Ok(mut text) = texts.get_single_mut() else {
        return;
    };

    text.0 = format!("FPS: {:.1} Particles: {}", fps, particle_count);
}

fn move_camera(
    mut cam: Query<&mut Transform, With<Camera>>,
    inputs: Res<ButtonInput<KeyCode>>,
    time: Res<Time>,
) {
    let x = inputs.pressed(KeyCode::ArrowLeft) as i32 - inputs.pressed(KeyCode::ArrowRight) as i32;
    let y = inputs.pressed(KeyCode::ArrowUp) as i32 - inputs.pressed(KeyCode::ArrowDown) as i32;

    let zoom = inputs.pressed(KeyCode::KeyO) as i32 - inputs.pressed(KeyCode::KeyI) as i32;

    cam.iter_mut().for_each(|mut t| {
        t.translation.x += x as f32 * 300. * time.delta_secs();
        t.translation.y += y as f32 * 300. * time.delta_secs();
        t.scale = (t.scale + (zoom as f32) * 0.1).max(Vec3::splat(0.1));
    });
}
