use bevy::{
    prelude::*,
    render::render_resource::{AsBindGroup, ShaderType},
};

use super::{Particle2dMaterial, PARTICLE_SPRITE_FRAG};

#[derive(Copy, Clone, ShaderType, Default)]
pub struct SpawnerUniform {
    max_frames: u32,
    // #[cfg(all(target_arch = "wasm32"))]
    _padding: Vec3,
}
#[derive(AsBindGroup, Asset, TypePath, Clone, Default)]
pub struct SpriteParticleMaterial {
    #[texture(0)]
    #[sampler(1)]
    pub texture: Option<Handle<Image>>,
    #[uniform(2)]
    uniform: SpawnerUniform,
}

impl SpriteParticleMaterial {
    pub fn new(texture: Handle<Image>, max_frames: u32) -> Self {
        SpriteParticleMaterial {
            texture: Some(texture),
            uniform: SpawnerUniform {
                max_frames,
                ..default()
            },
        }
    }
}

impl Particle2dMaterial for SpriteParticleMaterial {
    fn fragment_shader() -> bevy::render::render_resource::ShaderRef {
        PARTICLE_SPRITE_FRAG.into()
    }
}
