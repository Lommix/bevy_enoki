/// ----------------------------------------------
/// material example
/// how to add a custom material
/// ----------------------------------------------
use bevy::{
    core_pipeline::bloom::Bloom, diagnostic::DiagnosticsStore, image::ImageSamplerDescriptor,
    prelude::*, render::render_resource::AsBindGroup,
};
use bevy_enoki::{prelude::*, EnokiPlugin};

fn main() {
    App::new()
        .add_plugins(DefaultPlugins.set(ImagePlugin {
            default_sampler: ImageSamplerDescriptor::nearest(),
        }))
        .add_plugins(bevy::diagnostic::FrameTimeDiagnosticsPlugin::default())
        .add_plugins(EnokiPlugin)
        .add_plugins(Particle2dMaterialPlugin::<FireParticleMaterial>::default())
        .add_systems(Startup, setup)
        .add_systems(Update, (show_fps, move_camera))
        .run();
}

#[derive(Component)]
pub struct FpsText;

fn setup(
    mut cmd: Commands,
    mut materials: ResMut<Assets<FireParticleMaterial>>,
    server: Res<AssetServer>,
) {
    cmd.spawn((
        Camera2d,
        Camera {
            clear_color: ClearColorConfig::Custom(Color::BLACK),
            hdr: true,
            ..default()
        },
        Bloom {
            intensity: 0.1,
            ..default()
        },
    ));

    let material_handle = materials.add(FireParticleMaterial {
        texture: server.load("noise.png"),
    });

    cmd.spawn((
        ParticleSpawnerState::default(),
        ParticleEffectHandle(server.load("ice.particle.ron")),
        ParticleSpawner(material_handle),
    ));

    cmd.spawn((
        Text::default(),
        TextFont {
            font_size: 24.,
            ..default()
        },
        FpsText,
    ));
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

    let Ok(mut text) = texts.single_mut() else {
        return;
    };

    text.0 = format!(
        "O:ZoomOut I:ZoomIn Arrow:Move\nFPS: {:.1}\nParticles: {}",
        fps, particle_count
    );
}

#[derive(AsBindGroup, Asset, TypePath, Clone, Default)]
pub struct FireParticleMaterial {
    #[texture(0)]
    #[sampler(1)]
    texture: Handle<Image>,
}

impl Particle2dMaterial for FireParticleMaterial {
    fn fragment_shader() -> bevy::render::render_resource::ShaderRef {
        "custom_material.wgsl".into()
    }
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
