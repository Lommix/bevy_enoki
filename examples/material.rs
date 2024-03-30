/// ----------------------------------------------
/// material example
/// how to add a custom material
/// ----------------------------------------------
use bevy::{
    core_pipeline::bloom::BloomSettings, diagnostic::DiagnosticsStore, prelude::*,
    render::render_resource::AsBindGroup,
};
use bevy_enoki::{prelude::*, EnokiPlugin};

fn main() {
    App::new()
        .add_plugins(DefaultPlugins.set(ImagePlugin {
            default_sampler: bevy::render::texture::ImageSamplerDescriptor::nearest(),
        }))
        .add_plugins(bevy::diagnostic::FrameTimeDiagnosticsPlugin)
        .add_plugins(EnokiPlugin)
        .add_plugins(Particle2dMaterialPlugin::<FireParticleMaterial>::default())
        .add_systems(Startup, setup)
        .add_systems(Update, show_fps)
        .run()
}

#[derive(Component)]
pub struct FpsText;

fn setup(
    mut cmd: Commands,
    mut materials: ResMut<Assets<FireParticleMaterial>>,
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
            intensity: 0.1,
            ..default()
        },
    ));

    let material = materials.add(FireParticleMaterial {
        texture: server.load("noise.png"),
    });

    cmd.spawn((TextBundle::default(), FpsText));
    cmd.spawn((ParticleSpawnerBundle {
        transform: Transform::default(),
        effect: server.load("ice.particle.ron"),
        // material: DEFAULT_MATERIAL,
        material,
        ..default()
    },));
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
