use bevy::{
    asset::{uuid_handle, RenderAssetUsages},
    image::{CompressedImageFormats, ImageSampler},
    prelude::*,
    tasks::AsyncComputeTaskPool,
    time::common_conditions::on_timer,
};
use bevy_enoki::prelude::*;
use crossbeam::channel::{bounded, Receiver, Sender};
use rfd::AsyncFileDialog;
use std::{borrow::Cow, time::Duration};

pub struct FileManagerPlugin;
impl Plugin for FileManagerPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<EffectChannel>();
        app.init_resource::<TextureChannel>();
        app.add_systems(
            Update,
            (effect_file_watcher, texture_file_watcher).run_if(on_timer(Duration::from_millis(50))),
        );
    }
}

pub(crate) const SPRITE_TEXTURE: Handle<Image> =
    uuid_handle!("8ffc5db4-dcc4-4650-bc91-3a93247a4df3");

#[derive(Resource)]
pub(crate) struct TextureChannel {
    pub last_file_name: String,
    pub send: Sender<TextureFileWrapper>,
    rec: Receiver<TextureFileWrapper>,
}

#[derive(Resource)]
pub(crate) struct EffectChannel {
    pub last_file_name: String,
    pub send: Sender<EffectFileWrapper>,
    rec: Receiver<EffectFileWrapper>,
}

pub struct TextureFileWrapper {
    file_name: String,
    image: Image,
}

impl Default for TextureChannel {
    fn default() -> Self {
        let (tx, rx) = bounded(1);
        Self {
            last_file_name: "load texture".into(),
            send: tx,
            rec: rx,
        }
    }
}

pub struct EffectFileWrapper {
    file_name: String,
    effect: Particle2dEffect,
}

impl Default for EffectChannel {
    fn default() -> Self {
        let (tx, rx) = bounded(1);
        Self {
            last_file_name: "my_new_effect.ron".to_string(),
            send: tx,
            rec: rx,
        }
    }
}

fn effect_file_watcher(
    mut effect_channel: ResMut<EffectChannel>,
    mut instances: Query<&mut ParticleEffectInstance>,
) {
    let Ok(effect_wrapper) = effect_channel.rec.try_recv() else {
        return;
    };

    effect_channel.last_file_name = effect_wrapper.file_name;
    instances.iter_mut().for_each(|mut instance| {
        instance.0 = Some(effect_wrapper.effect.clone());
    });
}

fn texture_file_watcher(
    mut texture_channel: ResMut<TextureChannel>,
    mut images: ResMut<Assets<Image>>,
    mut materials: ResMut<Assets<crate::shader::SpriteMaterial>>,
) {
    let Ok(texture_wrapper) = texture_channel.rec.try_recv() else {
        return;
    };

    texture_channel.last_file_name = texture_wrapper.file_name;
    let _ = images.insert(&SPRITE_TEXTURE, texture_wrapper.image);

    materials
        .iter_mut()
        .for_each(|(_, mat)| mat.texture = Some(SPRITE_TEXTURE));
}

pub fn open_load_image_dialog(
    sender: Sender<TextureFileWrapper>,
    supported_extensions: Vec<Cow<'static, str>>,
) {
    AsyncComputeTaskPool::get()
        .spawn(async move {
            if let Some(handle) = AsyncFileDialog::new()
                .set_title("load effect asset")
                .add_filter("image", &supported_extensions)
                .pick_file()
                .await
            {
                let bytes = handle.read().await;

                let image = match Image::from_buffer(
                    &bytes,
                    bevy::image::ImageType::Format(ImageFormat::Png),
                    CompressedImageFormats::NONE,
                    false,
                    ImageSampler::nearest(),
                    RenderAssetUsages::RENDER_WORLD,
                ) {
                    Ok(img) => img,
                    Err(err) => {
                        error!("Failed to load image!\n\n {:?}", err);
                        return;
                    }
                };

                let packed_effect = TextureFileWrapper {
                    image,
                    file_name: handle.file_name(),
                };

                match sender.send(packed_effect) {
                    Ok(_) => (),
                    Err(err) => {
                        error!("Channel failed!\n\n {:?}", err);
                    }
                };
            }
        })
        .detach();
}

pub fn open_load_effect_dialog(sender: Sender<EffectFileWrapper>) {
    AsyncComputeTaskPool::get()
        .spawn(async move {
            if let Some(handle) = AsyncFileDialog::new()
                .set_title("load effect assset")
                .pick_file()
                .await
            {
                let bytes = handle.read().await;
                let effect: Particle2dEffect = match ron::de::from_bytes(&bytes) {
                    Ok(effect) => effect,
                    Err(err) => {
                        #[cfg(not(target_arch = "wasm32"))]
                        let path = handle.path().to_str().unwrap_or("File");

                        #[cfg(target_arch = "wasm32")]
                        let path = "File";

                        error!(
                            "`{}` is not a valid particle effect asset!\n\n {:?}",
                            path, err
                        );
                        return;
                    }
                };

                let packed_effect = EffectFileWrapper {
                    effect,
                    file_name: handle.file_name(),
                };

                match sender.send(packed_effect) {
                    Ok(_) => (),
                    Err(err) => {
                        error!("Channel failed!\n\n {:?}", err);
                    }
                };
            }
        })
        .detach();
}

pub fn open_save_effect_dialog(effect: Particle2dEffect, file_name: String) {
    AsyncComputeTaskPool::get()
        .spawn(async move {
            if let Some(handle) = AsyncFileDialog::new()
                .set_title("load effect assset")
                .set_file_name(file_name)
                .save_file()
                .await
            {
                let string = match ron::ser::to_string(&effect) {
                    Ok(b) => b,
                    Err(err) => {
                        error!(
                            "Ops, cannot convert to string, this should not happen!\n\n {:?}",
                            err
                        );
                        return;
                    }
                };

                match handle.write(string.as_bytes()).await {
                    Ok(_) => (),
                    Err(err) => {
                        error!("Could not be saved!\n\n {:?}", err);
                    }
                };
            }
        })
        .detach();
}
