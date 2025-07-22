#![allow(unused_imports)]
#![allow(dead_code)]
use std::{
    path::{Path, PathBuf},
    sync::{Arc, Mutex},
    time::SystemTime,
};

use bevy::{
    asset::weak_handle, prelude::*, render::render_resource::AsBindGroup,
    tasks::AsyncComputeTaskPool, time::common_conditions::on_timer,
};
use bevy_enoki::prelude::{Particle2dMaterial, Particle2dMaterialPlugin};
use rfd::AsyncFileDialog;

pub(crate) const SPRITE_SHADER: Handle<Shader> =
    weak_handle!("f3c0d7d0-06ef-4a6a-b715-f3578c8692f2");

pub struct ShaderPlugin;
impl Plugin for ShaderPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins(Particle2dMaterialPlugin::<SpriteMaterial>::default())
            .init_resource::<ShaderWatch>()
            .add_systems(Startup, setup)
            .add_systems(
                Update,
                reload_shader.run_if(on_timer(std::time::Duration::from_millis(100))),
            );
    }
}

#[derive(Resource, Default, Deref)]
pub struct ShaderWatch(Arc<Mutex<Option<ShaderWrapper>>>);

impl ShaderWatch {
    pub fn file_name(&self) -> Option<String> {
        self.lock()
            .ok()
            .and_then(|inner| {
                inner
                    .as_ref()
                    .map(|wrapper| PathBuf::from(wrapper.path.clone()))
            })
            .and_then(|path| path.file_name().map(|os| os.to_string_lossy().to_string()))
    }
}

pub struct ShaderWrapper {
    path: String,
    last_modified: SystemTime,
}

fn reload_shader(mut shaders: ResMut<Assets<Shader>>, watcher: Res<ShaderWatch>) {
    if let Ok(mut s) = watcher.try_lock() {
        if let Some(inner) = s.as_mut() {
            let last_mod = get_last_modified(&inner.path).unwrap();
            if inner.last_modified < last_mod {
                let shader = load_shader(&inner.path).unwrap();
                shaders.insert(&SPRITE_SHADER, shader);
                info!("shader `{}` reloaded!", inner.path);
                inner.last_modified = last_mod;
            }
        }
    }
}

fn get_last_modified(path: impl AsRef<Path>) -> anyhow::Result<SystemTime> {
    let meta = std::fs::metadata(path)?;
    let time = meta.modified()?;
    Ok(time)
}

fn load_shader(path: impl AsRef<Path>) -> anyhow::Result<Shader> {
    let shader_str = std::fs::read_to_string(path)?;
    Ok(Shader::from_wgsl(shader_str, ""))
}

pub fn setup(mut shaders: ResMut<Assets<Shader>>) {
    shaders.insert(&SPRITE_SHADER, Shader::from_wgsl(DEFAULT_SHADER_STR, ""));
}

pub const DEFAULT_SHADER_STR: &str = r#"
#import bevy_enoki::particle_vertex_out::{ VertexOutput }

@group(1) @binding(0) var texture: texture_2d<f32>;
@group(1) @binding(1) var texture_sampler: sampler;

// what you get
//struct VertexOutput {
//  @builtin(position) clip_position: vec4<f32>,
//  @location(0) @interpolate(flat) color: vec4<f32>,
//  @location(1) uv : vec2<f32>,
//  @location(2) lifetime_frac : f32,
//  @location(3) lifetime_total : f32,
//};

@fragment
fn fragment(in: VertexOutput) -> @location(0) vec4<f32> {
    let sample = textureSample(texture, texture_sampler, in.uv);
    return sample * in.color;
}
"#;
#[derive(AsBindGroup, Default, Clone, Asset, TypePath)]
pub struct SpriteMaterial {
    #[texture(0)]
    #[sampler(1)]
    pub texture: Option<Handle<Image>>,
}

impl Particle2dMaterial for SpriteMaterial {
    fn fragment_shader() -> bevy::render::render_resource::ShaderRef {
        SPRITE_SHADER.into()
    }
}

#[cfg(not(target_arch = "wasm32"))]
pub fn open_shader_dialog(wrapper: Arc<Mutex<Option<ShaderWrapper>>>) {
    AsyncComputeTaskPool::get()
        .spawn(async move {
            if let Some(handle) = AsyncFileDialog::new()
                .set_title("watch shader file")
                .pick_file()
                .await
            {
                if let Ok(mut inner) = wrapper.lock() {
                    *inner = Some(ShaderWrapper {
                        path: handle.path().to_string_lossy().to_string(),
                        last_modified: SystemTime::UNIX_EPOCH,
                    });
                }
            }
        })
        .detach();
}
#[cfg(not(target_arch = "wasm32"))]
pub fn open_shader_save(wrapper: Arc<Mutex<Option<ShaderWrapper>>>) {
    AsyncComputeTaskPool::get()
        .spawn(async move {
            if let Some(handle) = AsyncFileDialog::new()
                .set_title("save shader template")
                .set_file_name("my_new_shader.wgsl")
                .save_file()
                .await
            {
                match handle.write(DEFAULT_SHADER_STR.as_bytes()).await {
                    Ok(_) => {
                        if let Ok(mut inner) = wrapper.lock() {
                            *inner = Some(ShaderWrapper {
                                path: handle.path().to_string_lossy().to_string(),
                                last_modified: SystemTime::UNIX_EPOCH,
                            });
                        }
                    }
                    Err(err) => {
                        error!("Could not be saved!\n\n {:?}", err);
                    }
                };
            }
        })
        .detach();
}
