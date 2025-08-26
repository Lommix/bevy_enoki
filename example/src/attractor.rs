use bevy::prelude::*;
use bevy_enoki::prelude::*;

fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        .add_plugins(EnokiPlugin)
        .add_systems(Startup, setup)
        .run();
}

fn setup(mut cmds: Commands, server: Res<AssetServer>) {
    cmds.spawn(Camera2d);

    // Spawn particle spawner with attractors defined in the effect file
    cmds.spawn((
        ParticleSpawner::default(),
        ParticleEffectHandle(server.load("attractor.particle.ron")),
    ));
}
