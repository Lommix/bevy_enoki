use bevy::prelude::*;
use bevy_enoki::prelude::*;
mod utils;

fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        .add_plugins(EnokiPlugin)
        .add_systems(Startup, setup)
        .add_plugins(utils::camera_and_ui_plugin)
        .run();
}

fn setup(mut cmds: Commands, server: Res<AssetServer>) {
    // Spawn particle spawner with attractors defined in the effect file
    cmds.spawn((
        ParticleSpawner::default(),
        ParticleEffectHandle(server.load("attractor.particle.ron")),
    ));
}
