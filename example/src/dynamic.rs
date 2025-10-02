/// ----------------------------------------------
/// dynamic example
/// how to update effect behavior dynamiclly
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
        .add_systems(Startup, setup)
        .add_plugins(utils::camera_and_ui_plugin)
        .add_systems(Update, change_dynamic)
        .run();
}

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
        ChangeTimer(Timer::new(Duration::from_millis(300), TimerMode::Repeating)),
        Pcindex(0.),
    ));

    let material_handle = materials.add(SpriteParticle2dMaterial::from_texture(
        server.load("enoki.png"),
    ));

    cmd.spawn((
        ParticleSpawner(material_handle),
        ParticleEffectHandle(server.load("base.particle.ron")),
    ));
}

fn change_dynamic(
    mut elapsed: Local<f32>,
    mut query: Query<&mut ParticleEffectInstance>,
    time: Res<Time>,
) {
    *elapsed += time.delta_secs();

    let Ok(mut maybe_effect) = query.single_mut() else {
        return;
    };

    if let Some(effect) = maybe_effect.0.as_mut() {
        effect.linear_speed = Some(Rval::new(1000. * elapsed.sin().abs(), 0.1));
        effect.spawn_amount = 100;
    }
}
