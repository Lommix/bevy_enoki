use crate::SPRITE_SHADER;
use base64::Engine;
use bevy::{prelude::*, render::render_resource::Source, time::common_conditions::on_timer};
use bevy_enoki::prelude::*;
use lazy_static::lazy_static;
use std::{
    sync::{Arc, Mutex},
    time::Duration,
};
use wasm_bindgen::prelude::*;

pub struct BindingPlugin;
impl Plugin for BindingPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(
            Update,
            (
                update_shader_code.run_if(on_timer(Duration::from_millis(100))),
                update_url.run_if(on_timer(Duration::from_millis(100))),
            ),
        );
        app.add_systems(Startup, startup);
    }
}

fn startup(shaders: Res<Assets<Shader>>) {
    let Some(shader) = shaders.get(super::SPRITE_SHADER) else {
        return;
    };

    let event = web_sys::CustomEvent::new("shader").unwrap();
    event.init_custom_event_with_can_bubble_and_cancelable_and_detail(
        "shader-loaded",
        false,
        false,
        &JsValue::from_str(shader.source.as_str()),
    );

    let window = web_sys::window().expect("should have a window in this context");
    window
        .dispatch_event(&event)
        .expect("should dispatch the custom event");
}

#[derive(serde::Deserialize, serde::Serialize, Debug)]
pub struct ConfigOptions {
    pub effect: Particle2dEffect,
    pub shader: String,
}

impl Default for ConfigOptions {
    fn default() -> Self {
        let shader_code = r#"#import bevy_enoki::particle_vertex_out::{ VertexOutput }

@group(1) @binding(0) var texture: texture_2d<f32>;
@group(1) @binding(1) var texture_sampler: sampler;

@fragment
fn fragment(in: VertexOutput) -> @location(0) vec4<f32> {
	return textureSample(texture, texture_sampler, in.uv) * in.color;
}"#
        .into();
        Self {
            effect: Default::default(),
            shader: shader_code,
        }
    }
}

#[derive(Default)]
struct SharedWebState {
    shader_code: String,
    url: String,
}

lazy_static! {
    static ref SHADER_CODE: Arc<Mutex<SharedWebState>> =
        Arc::new(Mutex::new(SharedWebState::default()));
}

#[wasm_bindgen]
pub fn load_shader(shader_code: String) {
    let Ok(mut shader) = SHADER_CODE.lock() else {
        return;
    };
    shader.shader_code = shader_code;
}

#[wasm_bindgen]
pub fn get_url() -> Option<String> {
    let Ok(state) = SHADER_CODE.lock() else {
        return None;
    };
    Some(state.url.clone())
}

// --------------------------------------------------

fn update_shader_code(mut shaders: ResMut<Assets<Shader>>) {
    let Ok(obj) = SHADER_CODE.try_lock() else {
        return;
    };

    let Some(shader) = shaders.get_mut(SPRITE_SHADER) else {
        return;
    };

    if !obj.shader_code.is_empty() {
        let code = obj.shader_code.clone();
        shader.source = Source::Wgsl(code.into());
    }
}

fn update_url(query: Query<&ParticleEffectInstance>, shaders: Res<Assets<Shader>>) {
    let Ok(mut obj) = SHADER_CODE.try_lock() else {
        return;
    };

    let Some(shader) = shaders.get(SPRITE_SHADER) else {
        return;
    };

    let Some(effect) = query
        .get_single()
        .ok()
        .map(|instance| instance.0.as_ref())
        .flatten()
    else {
        return;
    };

    let options = ConfigOptions {
        effect: (**effect).clone(),
        shader: shader.source.as_str().into(),
    };

    let option_string = ron::ser::to_string(&options).unwrap();
    let base64_string = base64::prelude::BASE64_STANDARD.encode(option_string);

    obj.url = base64_string;
}
