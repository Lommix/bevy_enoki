use super::{Particle2dMaterial, PARTICLE_SPRITE_FRAG};
use bevy::{prelude::*, render::render_resource::AsBindGroup};

/// Sprite Material lets you add textures and animations
/// to particles.
#[derive(AsBindGroup, Asset, TypePath, Clone)]
pub struct SpriteParticle2dMaterial {
    #[texture(0)]
    #[sampler(1)]
    pub texture: Option<Handle<Image>>,
    #[uniform(2)]
    frame_data: UVec4,
}

impl Default for SpriteParticle2dMaterial {
    fn default() -> Self {
        Self {
            texture: None,
            frame_data: UVec4::ONE,
        }
    }
}

impl SpriteParticle2dMaterial {
    pub fn new(texture: Handle<Image>, max_hframes: u32, max_vframes: u32) -> Self {
        Self {
            texture: Some(texture),
            frame_data: UVec4::new(max_hframes, max_vframes, 0, 0),
        }
    }

    pub fn from_texture(texture: Handle<Image>) -> Self {
        Self {
            texture: Some(texture),
            frame_data: UVec4::new(1, 1, 0, 0),
        }
    }
}

impl Particle2dMaterial for SpriteParticle2dMaterial {
    fn fragment_shader() -> bevy::render::render_resource::ShaderRef {
        PARTICLE_SPRITE_FRAG.into()
    }
}
