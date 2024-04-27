use std::{sync::Arc, time::Duration};

use crate::SpriteMaterial;
use bevy::{
    prelude::*,
    render::{
        render_asset::RenderAssetUsages,
        texture::{CompressedImageFormats, ImageFormat, ImageType},
    },
    time::common_conditions::on_timer,
};
use crossbeam::channel::{unbounded, Receiver, Sender};
use wasm_bindgen::prelude::*;

pub(crate) struct TextureLoaderPlugin;
impl Plugin for TextureLoaderPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(
            Update,
            incoming_image.run_if(on_timer(Duration::from_millis(50))),
        );
    }
}

pub const IMAGE_HANDLE: Handle<Image> = Handle::weak_from_u128(232305661432306568803116897131);

lazy_static::lazy_static! {
    static ref IMAGE_COM: (Sender<Image>,Receiver<Image>) = unbounded();
}

fn incoming_image(
    mut images: ResMut<Assets<Image>>,
    mut materials: ResMut<Assets<SpriteMaterial>>,
) {
    let Ok(image) = IMAGE_COM.1.try_recv() else {
        return;
    };

    images.insert(IMAGE_HANDLE, image);
    materials
        .iter_mut()
        .for_each(|(_, mat)| mat.texture = Some(IMAGE_HANDLE));
}

#[wasm_bindgen]
pub fn load_image(bytes: &[u8]) {
    let image = Image::from_buffer(
        bytes,
        ImageType::Format(ImageFormat::Png),
        CompressedImageFormats::default(),
        true,
        bevy::render::texture::ImageSampler::Default,
        RenderAssetUsages::RENDER_WORLD,
    )
    .unwrap();
    IMAGE_COM.0.send(image).unwrap();
}
