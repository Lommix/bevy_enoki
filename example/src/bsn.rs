/// ----------------------------------------------
/// bsn example
/// how to spawn a particle effect with Bevy Scene Notation
/// ----------------------------------------------
use bevy::prelude::*;
use bevy_enoki::prelude::*;

mod utils;

fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        .add_plugins(EnokiPlugin)
        .add_plugins(utils::camera_and_ui_plugin)
        .add_systems(Startup, setup)
        .run();
}

fn setup(mut cmd: Commands) {
    // effect loaded from an asset path
    cmd.spawn_scene(bsn! {
        ParticleSpawner::<ColorParticle2dMaterial>
        ParticleEffectHandle("firework.particle.ron")
    });

    // full effect config defined inline
    cmd.spawn_scene(bsn! {
        Transform::from_xyz(-200., 0., 0.)
        ParticleSpawner::<ColorParticle2dMaterial>
        ParticleEffectHandle(asset_value(Particle2dEffect {
            spawn_rate: 0.01,
            spawn_amount: 5,
            emission_shape: EmissionShape::Circle(20.),
            lifetime: Rval::new(1.5, 0.5),
            linear_speed: Some(Rval::new(200., 0.3)),
            direction: Some(Rval::new(Vec2::Y, 0.8)),
            scale: Some(Rval::new(10., 1.)),
            color: Some(LinearRgba::rgb(1.0, 0.6, 0.1)),
            scale_curve: Some(MultiCurve::new().with_point(1., 0., None).with_point(0., 1., None)),
            ..default()
        }))
    });
}
