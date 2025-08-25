use bevy::prelude::*;
use bevy_enoki::prelude::*;

fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        .add_plugins(EnokiPlugin)
        .add_systems(Startup, setup)
        .run();
}

fn setup(mut cmds: Commands) {
    cmds.spawn(Camera2d);
    cmds.spawn(ParticleSpawner::default());
}
