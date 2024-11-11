use bevy::{
    prelude::*,
    render::{
        render_asset::RenderAssetUsages,
        render_resource::Source,
        texture::{CompressedImageFormats, ImageFormat, ImageType},
    },
    tasks::AsyncComputeTaskPool,
};
use bevy_egui::egui::{self, Ui};
use crossbeam::channel::{bounded, Receiver, Sender};
use rfd::AsyncFileDialog;

use crate::SpriteMaterial;

pub(crate) struct MaterialEditorPlugin;
impl Plugin for MaterialEditorPlugin {
    fn build(&self, app: &mut App) {
        let shader_content = r#"#import bevy_enoki::particle_vertex_out::{ VertexOutput }

@group(1) @binding(0) var texture: texture_2d<f32>;
@group(1) @binding(1) var texture_sampler: sampler;


@fragment
fn fragment(in: VertexOutput) -> @location(0) vec4<f32> {
	return textureSample(texture, texture_sampler, in.uv) * in.color;
}"#;

        app.world
            .resource_mut::<Assets<Shader>>()
            .insert(super::SPRITE_SHADER, Shader::from_wgsl(shader_content, ""));

        app.init_resource::<TextEditorState>();
        app.init_resource::<ImageLoaderState>();
        app.add_systems(Update, (code_gui, update_material));
    }
}

#[derive(Resource)]
struct ImageLoaderState {
    sender: Sender<LoadedImage>,
    reciever: Receiver<LoadedImage>,
}

struct LoadedImage {
    image: Image,
    file_name: String,
}

impl Default for ImageLoaderState {
    fn default() -> Self {
        let (tx, rx) = bounded(1);
        Self {
            sender: tx,
            reciever: rx,
        }
    }
}

const IMAGE_HANDLE: Handle<Image> = Handle::weak_from_u128(232305661432306568803116897131);

fn update_material(
    text_editor: ResMut<TextEditorState>,
    mut materials: ResMut<Assets<SpriteMaterial>>,
) {
    materials.iter_mut().for_each(|(_, mat)| {
        mat.texture = text_editor.texture.clone();
    });
}

fn code_gui(
    mut contexts: bevy_egui::EguiContexts,
    mut text_editor: ResMut<TextEditorState>,
    mut shaders: ResMut<Assets<Shader>>,
    mut images: ResMut<Assets<Image>>,
    image_state: Res<ImageLoaderState>,
) {
    if let Ok(loaded_image) = image_state.reciever.try_recv() {
        images.insert(IMAGE_HANDLE, loaded_image.image);
        text_editor.texture = Some(IMAGE_HANDLE);
        text_editor.texture_name = Some(loaded_image.file_name.clone());
    }

    egui::Window::new("Material Editor")
        .default_pos([1000., 10.])
        .show(contexts.ctx_mut(), |ui| {

            ui.label(r#"Fragment Shader - Check the browser console for errors in your shader. This is a very basic single texture particle material."#);
            ui.hyperlink_to("Custom material example","https://github.com/Lommix/bevy_enoki/blob/master/examples/material.rs");

            ui.collapsing("WGSL Info",|ui|{
                ui.label(
                    r#"
These are the inputs available in your fragment stage
struct VertexOutput {
    @builtin(position) clip_position: vec4<f32>,
    @location(0) @interpolate(flat) color: vec4<f32>,
    @location(1) uv : vec2<f32>,
    @location(2) lifetime_frac : f32,
    @location(3) lifetime_total : f32,
};
                    "#)
            });

            egui::Grid::new("code tabs")
                .num_columns(2)
                .spacing([10., 0.])
                .show(ui, |ui| {
                    ui.label("Upload a Texture");

                    let button_text = match text_editor.texture_name.as_ref() {
                        Some(file_name) => file_name.to_string(),
                        None => "None".to_string(),
                    };

                    if ui.button(button_text).clicked() {
                        let sender = image_state.sender.clone();
                        AsyncComputeTaskPool::get()
                            .spawn(async move {
                                match AsyncFileDialog::new()
                                    .set_title("Select Texture")
                                    .add_filter("png", &["png"])
                                    .pick_file()
                                    .await
                                {
                                    Some(file_handle) => {
                                        let content = file_handle.read().await;
                                        let image = Image::from_buffer(
                                            content.as_slice(),
                                            ImageType::Format(ImageFormat::Png),
                                            CompressedImageFormats::default(),
                                            true,
                                            bevy::render::texture::ImageSampler::Default,
                                            RenderAssetUsages::RENDER_WORLD,
                                        )
                                        .unwrap();

                                        sender
                                            .send(LoadedImage {
                                                image,
                                                file_name: file_handle.file_name(),
                                            })
                                            .unwrap();
                                        info!("loading complete");
                                    }
                                    None => (),
                                };
                            })
                            .detach();
                    }
                });

            text_editor.code_window(ui);

            egui::Grid::new("code tabs")
                .spacing([10., 0.])
                .num_columns(3)
                .show(ui, |ui| {
                    if ui.button("Update Shader").clicked() {
                        text_editor.update_shader(&mut shaders);
                    }
                });
        });
}

#[derive(Resource)]
pub struct TextEditorState {
    texture_name: Option<String>,
    texture: Option<Handle<Image>>,
    content: String,
    checksum: u64,
}

impl FromWorld for TextEditorState {
    fn from_world(world: &mut World) -> Self {
        let shader = world
            .resource::<Assets<Shader>>()
            .get(super::SPRITE_SHADER)
            .unwrap();

        TextEditorState {
            texture_name: None,
            texture: None,
            content: shader.source.as_str().to_string(),
            checksum: simple_checksum(shader.source.as_str()),
        }
    }
}

impl TextEditorState {
    fn code_window(&mut self, ui: &mut Ui) {
        ui.add(
            egui::TextEdit::multiline(&mut self.content)
                .desired_width(f32::INFINITY)
                .code_editor(),
        );
    }

    fn update_shader(&mut self, shaders: &mut Assets<Shader>) {
        let checksum = simple_checksum(self.content.as_str());
        if checksum == self.checksum {
            return;
        }

        self.checksum = checksum;

        let Some(shader) = shaders.get_mut(super::SPRITE_SHADER) else {
            return;
        };

        shader.source = Source::Wgsl(self.content.clone().into());
    }
}

fn simple_checksum(s: &str) -> u64 {
    let mut hash: u64 = 0;
    for (i, c) in s.chars().enumerate() {
        hash = hash.wrapping_add((c as u64).wrapping_mul(i as u64 + 1));
    }
    hash
}
