use bevy::{
    prelude::*,
    render::render_resource::{AsBindGroup, ShaderType},
};

use super::{prelude::Particle2dMaterial, PARTICLE_COLOR_FRAG};

/// simple color material that gets multiplied by
/// any color, caluclated in the particle effect
#[derive(AsBindGroup, Asset, TypePath, Clone)]
pub struct ColorParticle2dMaterial {
    #[uniform(0)]
    color: LinearRgba,
}

#[derive(ShaderType, Asset, TypePath, Clone)]
pub struct ColorParticle2dUniform {
    color: Vec4,
}

impl Default for ColorParticle2dMaterial {
    fn default() -> Self {
        Self {
            color: LinearRgba::WHITE,
        }
    }
}

impl ColorParticle2dMaterial {
    pub fn new(color: LinearRgba) -> Self {
        Self { color }
    }
}

impl Particle2dMaterial for ColorParticle2dMaterial {
    fn fragment_shader() -> bevy::render::render_resource::ShaderRef {
        PARTICLE_COLOR_FRAG.into()
    }
}
