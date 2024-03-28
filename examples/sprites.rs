use std::time::Duration;
use bevy::{core_pipeline::bloom::BloomSettings, diagnostic::DiagnosticsStore, prelude::*};
use bevy_enoki::{prelude::*, EnokiPlugin};

fn main() {
    App::new()
        .add_plugins(DefaultPlugins.set(ImagePlugin {
            default_sampler: bevy::render::texture::ImageSamplerDescriptor::nearest(),
        }))
        .add_plugins(bevy::diagnostic::FrameTimeDiagnosticsPlugin)
        .add_plugins(EnokiPlugin)
        .add_systems(Startup, setup)
        .add_systems(Update, (show_fps, spawn_spawn))
        .run()
}

#[derive(Component)]
pub struct FpsText;

#[derive(Deref, Component, DerefMut)]
pub struct MoveTimer(Timer);

#[derive(Deref, Component, DerefMut)]
pub struct Pcindex(f32);

#[derive(Deref, Resource, DerefMut)]
pub struct ParticleMaterialAsset(Handle<SpriteParticleMaterial>);

fn setup(
    mut cmd: Commands,
    mut materials: ResMut<Assets<SpriteParticleMaterial>>,
    server: Res<AssetServer>,
) {
    cmd.spawn((
        Camera2dBundle {
            camera: Camera {
                clear_color: ClearColorConfig::Custom(Color::BLACK),
                hdr: true,
                ..default()
            },

            ..default()
        },
        BloomSettings {
            intensity: 0.3,
            ..default()
        },
    ));

    cmd.spawn((
        MoveTimer(Timer::new(Duration::from_millis(300), TimerMode::Repeating)),
        Pcindex(0.),
    ));

    cmd.spawn((TextBundle::default(), FpsText));
    cmd.insert_resource(ParticleMaterialAsset(
        materials.add(SpriteParticleMaterial::new(server.load("particle.png"), 5)),
    ));
}

fn spawn_spawn(
    mut cmd: Commands,
    mut query: Query<(&mut MoveTimer, &mut Pcindex)>,
    time: Res<Time>,
    server: Res<AssetServer>,
    particle_material: Res<ParticleMaterialAsset>,
) {
    let Ok((mut timer, mut index)) = query.get_single_mut() else {
        return;
    };

    timer.tick(time.delta());
    if !timer.finished() {
        return;
    }

    for _ in 0..1 {
        // let x = (rand::random::<f32>() - 0.5) * 500.;
        // let y = (rand::random::<f32>() - 0.5) * 500.;

        cmd.spawn((
            ParticleSpawnerBundle {
                transform: Transform::from_xyz(0., 0., index.0),
                effect: server.load("test.particle.ron"),
                material: particle_material.clone(),
                ..default()
            },
            OneShot,
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

    text.sections = vec![TextSection::new(
        format!("FPS: {:.1} Particles: {}", fps, particle_count),
        TextStyle {
            font_size: 45.,
            color: Color::WHITE,
            ..default()
        },
    )]
    // info!(fps);
}
