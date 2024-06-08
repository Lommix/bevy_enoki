use base64::Engine;
use bevy::{
    core_pipeline::bloom::BloomSettings,
    input::mouse::MouseWheel,
    prelude::*,
    render::render_resource::{AsBindGroup, Source},
};
use bevy_egui::egui::{self, Color32, Ui};
use bevy_enoki::prelude::*;
use file::FileResource;
use wasm_bindgen::prelude::wasm_bindgen;

mod bindings;
mod file;
mod gui;
mod texture;

pub(crate) const SPRITE_SHADER: Handle<Shader> =
    Handle::weak_from_u128(908340313783013137964307813738);

fn main() {
    #[cfg(not(target_arch = "wasm32"))]
    run(None);
}

#[wasm_bindgen]
pub fn run(options: Option<String>) {
    let config = options
        .map(|hash_encoded| base64::prelude::BASE64_URL_SAFE.decode(hash_encoded).ok())
        .flatten()
        .map(|config_string| ron::de::from_bytes::<bindings::ConfigOptions>(&config_string).ok())
        .flatten();

    App::new()
        .add_plugins((
            DefaultPlugins,
            EnokiPlugin,
            Particle2dMaterialPlugin::<SpriteMaterial>::default(),
            bevy_egui::EguiPlugin,
            file::FileManagerPlugin,
            texture::TextureLoaderPlugin,
            bindings::BindingPlugin,
            LoaderPlugin(config),
        ))
        .add_systems(Startup, setup)
        .add_systems(Update, (gui, zoom))
        .run();
}

// this is not how you should do it, i am just lazy
struct LoaderPlugin(Option<bindings::ConfigOptions>);
impl Plugin for LoaderPlugin {
    fn build(&self, app: &mut App) {
        let shader_content = match self.0.as_ref() {
            Some(config) => config.shader.clone(),
            None => r#"#import bevy_enoki::particle_vertex_out::{ VertexOutput }

@group(1) @binding(0) var texture: texture_2d<f32>;
@group(1) @binding(1) var texture_sampler: sampler;


@fragment
fn fragment(in: VertexOutput) -> @location(0) vec4<f32> {
	return textureSample(texture, texture_sampler, in.uv) * in.color;
}"#
            .into(),
        };

        app.world
            .resource_mut::<Assets<Shader>>()
            .insert(SPRITE_SHADER, Shader::from_wgsl(shader_content, ""));

        let effect_handle = match self.0.as_ref() {
            Some(config) => {
                let handle = app
                    .world
                    .resource_mut::<Assets<Particle2dEffect>>()
                    .add(config.effect.clone());
                handle
            }
            None => {
                let fireworks = include_str!("../../assets/firework.particle.ron");
                let effect: Particle2dEffect = ron::de::from_str(fireworks).unwrap();
                app.world
                    .resource_mut::<Assets<Particle2dEffect>>()
                    .add(effect)
            }
        };

        let material_handle = app
            .world
            .resource_mut::<Assets<SpriteMaterial>>()
            .add(SpriteMaterial { texture: None });

        app.world.spawn(ParticleSpawnerBundle {
            material: material_handle,
            effect: effect_handle,
            ..default()
        });
    }
}

fn zoom(mut camera: Query<&mut Transform, With<Camera>>, mut events: EventReader<MouseWheel>) {
    let Ok(mut transform) = camera.get_single_mut() else {
        return;
    };

    events.read().for_each(|ev| {
        transform.scale =
            (transform.scale + Vec3::splat(0.1) * ev.y.signum()).max(Vec3::splat(0.1));
    })
}

fn setup(mut cmd: Commands) {
    cmd.spawn((
        Camera2dBundle {
            transform: Transform::from_scale(Vec3::splat(2.0)),
            camera: Camera {
                hdr: true,
                clear_color: ClearColorConfig::Custom(Color::BLACK),
                ..default()
            },
            ..default()
        },
        BloomSettings {
            intensity: 0.,
            prefilter_settings: bevy::core_pipeline::bloom::BloomPrefilterSettings {
                threshold: 1.,
                ..default()
            },
            ..default()
        },
    ));
}

fn gui(
    mut cmd: Commands,
    mut context: bevy_egui::EguiContexts,
    mut effect_query: Query<(
        Entity,
        &mut ParticleEffectInstance,
        &mut ParticleSpawnerState,
    )>,
    mut file: ResMut<FileResource>, // world: &mut World,
    mut camera_query: Query<(&mut Camera, &mut BloomSettings)>,
    mut one_shot_mode: Local<bool>,
) {
    let Ok((entity, mut effect_instance, mut state)) = effect_query.get_single_mut() else {
        return;
    };

    egui::Window::new("Particle Config")
        .scroll2([false, true])
        .show(context.ctx_mut(), |ui| {
            file::file_panel(ui, &mut effect_instance, &mut file);
            egui::Grid::new("one_shot")
                .spacing([4., 4.])
                .num_columns(2)
                .min_col_width(100.)
                .show(ui, |ui| {
                    if ui.checkbox(&mut one_shot_mode, "One Shot").changed() {
                        if *one_shot_mode {
                            cmd.entity(entity).insert(OneShot::Deactivate);
                        } else {
                            cmd.entity(entity).remove::<OneShot>();
                        }
                    }

                    if ui
                        .add_sized([100., 30.], egui::Button::new("Spawn Once"))
                        .clicked()
                    {
                        state.active = true;
                    }
                });

            if let Ok((mut camera, mut bloom)) = camera_query.get_single_mut() {
                ui.collapsing("Scene Setting", |ui| {
                    egui::Grid::new("scene_setting")
                        .num_columns(2)
                        .show(ui, |ui| {
                            ui.label("Background");

                            let mut color32 = match camera.clear_color {
                                ClearColorConfig::Custom(color) => bevy_to_egui_color(color),
                                _ => bevy_to_egui_color(Color::DARK_GRAY),
                            };
                            egui::color_picker::color_edit_button_srgba(
                                ui,
                                &mut color32,
                                egui::color_picker::Alpha::Opaque,
                            );
                            camera.clear_color =
                                ClearColorConfig::Custom(egui_to_bevy_color(color32));

                            ui.end_row();
                            ui.label("Bloom Settings");
                            ui.end_row();

                            ui.label("Intensity");
                            ui.add(egui::Slider::new(&mut bloom.intensity, (0.)..=5.));
                            ui.end_row();

                            ui.label("Threshold");
                            ui.add(egui::Slider::new(
                                &mut bloom.prefilter_settings.threshold,
                                (0.)..=1.,
                            ));
                            ui.end_row();

                            ui.label("Softness");
                            ui.add(egui::Slider::new(
                                &mut bloom.prefilter_settings.threshold_softness,
                                (0.)..=1.,
                            ));
                            ui.end_row();

                            ui.label("low freq");
                            ui.add(egui::Slider::new(&mut bloom.low_frequency_boost, (0.)..=1.));
                            ui.end_row();

                            ui.label("high freq");
                            ui.add(egui::Slider::new(&mut bloom.high_pass_frequency, (0.)..=1.));
                            ui.end_row();
                        });
                });
            }

            if let Some(effect) = effect_instance.0.as_mut() {
                gui::config_gui(ui, effect, &mut state);
            }
        });
}

pub(crate) fn bevy_to_egui_color(color: Color) -> Color32 {
    let s = color.as_rgba_u8();
    Color32::from_rgba_unmultiplied(s[0], s[1], s[2], s[3])
}

pub(crate) fn egui_to_bevy_color(color: Color32) -> Color {
    Color::rgba_from_array(color.to_normalized_gamma_f32())
}

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
