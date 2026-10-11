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
    cmd.spawn_scene(bsn! {
        ParticleSpawner::<ColorParticle2dMaterial>
        ParticleEffectHandle("firework.particle.ron")
    });
}
