/// ----------------------------------------------
/// sprite example
/// how to display a sprite animation/texture
/// ----------------------------------------------
use bevy::{image::ImageSamplerDescriptor, prelude::*};
use bevy_enoki::{prelude::*, EnokiPlugin};
use std::time::Duration;

mod utils;

fn main() {
    App::new()
        .add_plugins(DefaultPlugins.set(ImagePlugin {
            default_sampler: ImageSamplerDescriptor::nearest(),
        }))
        .add_plugins(EnokiPlugin)
        .add_plugins(utils::camera_and_ui_plugin)
        .add_systems(Startup, setup)
        .add_systems(Update, spawn_particles)
        .run();
}

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
        MoveTimer(Timer::new(Duration::from_millis(300), TimerMode::Repeating)),
        Pcindex(0.),
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
    let Ok((mut timer, mut index)) = query.single_mut() else {
        return;
    };

    timer.tick(time.delta());
    if !timer.is_finished() {
        return;
    }

    for _ in 0..3 {
        let x = (rand::random::<f32>() - 0.5) * 500.;
        let y = (rand::random::<f32>() - 0.5) * 500.;

        cmd.spawn((
            ParticleEffectHandle(server.load("firework.particle.ron")),
            ParticleSpawner(material.0.clone()),
            Transform::from_xyz(x, y, index.0),
            OneShot::Despawn,
        ));

        index.0 += 1.;
    }
}
