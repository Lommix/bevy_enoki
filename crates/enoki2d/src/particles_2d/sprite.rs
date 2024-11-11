use super::{Particle2dMaterial, PARTICLE_SPRITE_FRAG};
use bevy::{
    asset::{io::Reader, AssetLoadError, AssetLoader, AsyncReadExt},
    prelude::*,
    render::render_resource::AsBindGroup,
};
use serde::Deserialize;

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
}

impl Particle2dMaterial for SpriteParticle2dMaterial {
    fn fragment_shader() -> bevy::render::render_resource::ShaderRef {
        PARTICLE_SPRITE_FRAG.into()
    }
}

#[derive(Deserialize)]
pub struct SpriteParticle2dRon {
    pub image: String,
    pub h_frames: u32,
    pub v_frames: u32,
}

#[derive(Default)]
pub struct ColorParticle2dAssetLoader;
impl AssetLoader for ColorParticle2dAssetLoader {
    type Asset = SpriteParticle2dMaterial;
    type Settings = ();
    type Error = AssetLoadError;

    async fn load(
        &self,
        reader: &mut dyn Reader,
        _settings: &Self::Settings,
        load_context: &mut bevy::asset::LoadContext<'_>,
    ) -> Result<Self::Asset, Self::Error> {
        let mut bytes = Vec::new();
        reader.read_to_end(&mut bytes).await.unwrap();
        let ron_asset = ron::de::from_bytes::<SpriteParticle2dRon>(bytes.as_slice())
            .map_err(|_| AssetLoadError::AssetMetaReadError)?;

        Ok(SpriteParticle2dMaterial::new(
            load_context.load(ron_asset.image),
            ron_asset.h_frames,
            ron_asset.v_frames,
        ))
    }
}
