/// ----------------------------------------------
/// sprite example
/// how to display a sprite animation/texture
/// ----------------------------------------------
use bevy::{core_pipeline::bloom::Bloom, diagnostic::DiagnosticsStore, prelude::*};
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
        .add_systems(Update, (show_fps, spawn_particles, move_camera))
        .run();
}

#[derive(Component)]
pub struct FpsText;

#[derive(Deref, Component, DerefMut)]
pub struct MoveTimer(Timer);

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
            ..default()
        },
        Bloom {
            intensity: 0.1,
            ..default()
        },
    ));

    cmd.spawn((
        MoveTimer(Timer::new(Duration::from_millis(300), TimerMode::Repeating)),
        Pcindex(0.),
    ));

    cmd.spawn((
        Text::default(),
        TextFont {
            font_size: 24.,
            ..default()
        },
        FpsText,
    ));
    cmd.insert_resource(ParticleMaterialAsset(materials.add(
        SpriteParticle2dMaterial::new(server.load("particle.png"), 6, 1),
    )));
}

fn spawn_particles(
    mut cmd: Commands,
    mut query: Query<(&mut MoveTimer, &mut Pcindex)>,
    time: Res<Time>,
    material: Res<ParticleMaterialAsset>,
    server: Res<AssetServer>,
) {
    let Ok((mut timer, mut index)) = query.get_single_mut() else {
        return;
    };

    timer.tick(time.delta());
    if !timer.finished() {
        return;
    }

    for _ in 0..3 {
        let x = (rand::random::<f32>() - 0.5) * 500.;
        let y = (rand::random::<f32>() - 0.5) * 500.;

        cmd.spawn((
            EffectHandle(server.load("firework.particle.ron")),
            MaterialHandle(material.0.clone()),
            ParticleSpawnerState::default(),
            Transform::from_xyz(x, y, index.0),
            OneShot::Despawn,
        ));

        index.0 += 1.;
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

    text.0 = format!(
        "O:ZoomOut I:ZoomIn Arrow:Move\nFPS: {:.1}\nParticles: {}",
        fps, particle_count
    );
}

fn move_camera(
    mut cam: Query<&mut Transform, With<Camera>>,
    inputs: Res<ButtonInput<KeyCode>>,
    time: Res<Time>,
) {
    let x = inputs.pressed(KeyCode::ArrowRight) as i32 - inputs.pressed(KeyCode::ArrowLeft) as i32;
    let y = inputs.pressed(KeyCode::ArrowUp) as i32 - inputs.pressed(KeyCode::ArrowDown) as i32;

    let zoom = inputs.pressed(KeyCode::KeyO) as i32 - inputs.pressed(KeyCode::KeyI) as i32;

    cam.iter_mut().for_each(|mut t| {
        t.translation.x += x as f32 * 300. * time.delta_secs();
        t.translation.y += y as f32 * 300. * time.delta_secs();
        t.scale = (t.scale + (zoom as f32) * 0.1).max(Vec3::splat(0.1));
    });
}
