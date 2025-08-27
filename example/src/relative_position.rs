use bevy::prelude::*;
use bevy_enoki::prelude::*;

const RADIUS: f32 = 200.0;
const SPEED: f32 = 1.5;

fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        .add_plugins(EnokiPlugin)
        .add_systems(Startup, setup)
        .add_systems(Update, move_spawners)
        .run();
}

#[derive(Component)]
struct MovingSpawner {
    phase_offset: f32,
}

fn setup(mut cmds: Commands, server: Res<AssetServer>) {
    cmds.spawn(Camera2d);

    // Spawn particle system with relative positioning - starts on right side
    cmds.spawn((
        ParticleSpawner::default(),
        ParticleEffectHandle(server.load("relative_pos.particle.ron")),
        MovingSpawner {
            phase_offset: 0.0, // Starts at 0 degrees (right side)
        },
        Transform::from_translation(Vec3::new(RADIUS, 0.0, 0.0)),
    ));

    // Spawn particle system with non-relative positioning - starts on left side
    cmds.spawn((
        ParticleSpawner::default(),
        ParticleEffectHandle(server.load("non-relative_pos.particle.ron")),
        MovingSpawner {
            phase_offset: std::f32::consts::PI, // Starts at 180 degrees (left side)
        },
        Transform::from_translation(Vec3::new(-RADIUS, 0.0, 0.0)),
    ));
}

fn move_spawners(time: Res<Time>, mut query: Query<(&mut Transform, &MovingSpawner)>) {
    for (mut transform, moving_spawner) in query.iter_mut() {
        // Move both spawners around the same circle, but with different phase offsets
        let t = time.elapsed_secs();
        let angle = t * SPEED + moving_spawner.phase_offset;

        transform.translation.x = angle.cos() * RADIUS;
        transform.translation.y = angle.sin() * RADIUS;
        transform.translation.z = 0.0;
    }
}
