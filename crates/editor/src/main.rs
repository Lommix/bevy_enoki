use bevy::{
    core_pipeline::bloom::Bloom, input::mouse::MouseWheel, prelude::*,
    render::render_resource::AsBindGroup,
};
pub(crate) use bevy_egui::egui::{self, Color32};
use bevy_enoki::prelude::*;
// use file::FileResource;
// use wasm_bindgen::prelude::wasm_bindgen;

// mod bindings;
// mod file;
mod gui;
// mod texture;

pub(crate) const SPRITE_SHADER: Handle<Shader> =
    Handle::weak_from_u128(908340313783013137964307813738);

fn main() {
    App::new()
        .add_plugins((
            DefaultPlugins,
            EnokiPlugin,
            Particle2dMaterialPlugin::<SpriteMaterial>::default(),
            bevy_egui::EguiPlugin,
            // file::FileManagerPlugin,
            // texture::TextureLoaderPlugin,
            // bindings::BindingPlugin,
        ))
        .add_systems(Startup, setup)
        .add_systems(Update, (gui, zoom))
        .run();
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
        Camera2d,
        Camera {
            hdr: true,
            clear_color: ClearColorConfig::Custom(Color::BLACK),
            ..default()
        },
        Transform::from_scale(Vec3::splat(2.0)),
        Bloom {
            intensity: 0.,
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
    // mut file: ResMut<FileResource>, // world: &mut World,
    mut camera_query: Query<(&mut Camera, &mut Bloom)>,
    mut one_shot_mode: Local<bool>,
) {
    let Ok((entity, mut effect_instance, mut state)) = effect_query.get_single_mut() else {
        return;
    };

    egui::Window::new("Particle Config")
        .scroll([false, true])
        .show(context.ctx_mut(), |ui| {
            // file::file_panel(ui, &mut effect_instance, &mut file);
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
                                _ => bevy_to_egui_color(Color::LinearRgba(LinearRgba::GREEN)),
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
                            ui.add(egui::Slider::new(&mut bloom.prefilter.threshold, (0.)..=1.));
                            ui.end_row();

                            ui.label("Softness");
                            ui.add(egui::Slider::new(
                                &mut bloom.prefilter.threshold_softness,
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
    let s = color.to_linear().to_u8_array();
    Color32::from_rgba_unmultiplied(s[0], s[1], s[2], s[3])
}

pub(crate) fn egui_to_bevy_color(color: Color32) -> Color {
    Color::LinearRgba(LinearRgba::from_f32_array(color.to_normalized_gamma_f32()))
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
