/// ----------------------------------------------
/// material example
/// how to add a custom material
/// ----------------------------------------------
use bevy::{image::ImageSamplerDescriptor, prelude::*, render::render_resource::AsBindGroup};
use bevy_enoki::{prelude::*, EnokiPlugin};

mod utils;

fn main() {
    App::new()
        .add_plugins(DefaultPlugins.set(ImagePlugin {
            default_sampler: ImageSamplerDescriptor::nearest(),
        }))
        .add_plugins(EnokiPlugin)
        .add_plugins(Particle2dMaterialPlugin::<FireParticleMaterial>::default())
        .add_plugins(utils::camera_and_ui_plugin)
        .add_systems(Startup, setup)
        .run();
}

fn setup(
    mut cmd: Commands,
    mut materials: ResMut<Assets<FireParticleMaterial>>,
    server: Res<AssetServer>,
) {
    let material_handle = materials.add(FireParticleMaterial {
        texture: server.load("noise.png"),
    });

    cmd.spawn((
        ParticleSpawnerState::default(),
        ParticleEffectHandle(server.load("ice.particle.ron")),
        ParticleSpawner(material_handle),
    ));
}

#[derive(AsBindGroup, Asset, TypePath, Clone, Default)]
pub struct FireParticleMaterial {
    #[texture(0)]
    #[sampler(1)]
    texture: Handle<Image>,
}

impl Particle2dMaterial for FireParticleMaterial {
    fn fragment_shader() -> bevy::shader::ShaderRef {
        "custom_material.wgsl".into()
    }
}
