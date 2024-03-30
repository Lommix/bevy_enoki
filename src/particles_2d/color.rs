use bevy::{prelude::*, render::render_resource::AsBindGroup};

use super::{prelude::Particle2dMaterial, PARTICLE_COLOR_FRAG};

#[derive(AsBindGroup, Asset, TypePath, Clone)]
pub struct ColorParticle2dMaterial {
    #[uniform(0)]
    color: Color,
}

impl Default for ColorParticle2dMaterial {
    fn default() -> Self {
        Self {
            color: Color::WHITE,
        }
    }
}

impl ColorParticle2dMaterial {
    pub fn new(color: Color) -> Self {
        Self { color }
    }
}

impl Particle2dMaterial for ColorParticle2dMaterial {
    fn fragment_shader() -> bevy::render::render_resource::ShaderRef {
        PARTICLE_COLOR_FRAG.into()
    }
}
