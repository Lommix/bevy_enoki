use bevy::{core_pipeline::bloom::Bloom, log::LogPlugin, prelude::*};
use bevy_egui::egui::FontId;
use bevy_egui::egui::{self, Color32};
use bevy_enoki::prelude::*;
use bevy_pancam::{DirectionKeys, PanCam, PanCamPlugin};
use file::{EffectChannel, TextureChannel};
use log::LogBuffer;

mod file;
mod gui;
mod log;
mod shader;

fn main() {
    App::new()
        .add_plugins((
            DefaultPlugins.set(LogPlugin {
                level: bevy::log::Level::INFO,
                filter: "wgpu=error,naga=warn".into(),
                custom_layer: log::log_capture_layer,
            }),
            PanCamPlugin::default(),
            EnokiPlugin,
            bevy_egui::EguiPlugin,
            file::FileManagerPlugin,
            log::LogPlugin,
            shader::ShaderPlugin,
        ))
        .add_systems(Startup, setup)
        .add_systems(Update, gui)
        .run();
}

fn setup(mut cmd: Commands, mut particle_materials: ResMut<Assets<shader::SpriteMaterial>>) {
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
        Msaa::Off,
        PanCam {
            grab_buttons: vec![MouseButton::Middle],
            ..default()
        },
    ));

    cmd.spawn((
        ParticleSpawner(particle_materials.add(shader::SpriteMaterial::default())),
        Transform::from_xyz(-100., 0., 0.),
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
    mut camera_query: Query<(&mut Camera, &mut Bloom)>,
    mut one_shot_mode: Local<bool>,
    effect_channel: Res<EffectChannel>,
    texture_channel: Res<TextureChannel>,
    mut logs: ResMut<LogBuffer>,
    watcher: Res<shader::ShaderWatch>,
) {
    let Ok((entity, mut effect_instance, mut state)) = effect_query.get_single_mut() else {
        return;
    };

    let central = egui::CentralPanel::default().frame(egui::Frame { ..default() });
    central.show(context.ctx_mut(), |ui| {
        egui::TopBottomPanel::top("Enoki particle editor").show_inside(ui, |ui| {
            ui.horizontal(|ui| {
                let styles = ui.style_mut();

                styles.spacing.item_spacing = [8., 4.].into();
                styles.text_styles.insert(
                    egui::TextStyle::Heading,
                    FontId::new(30.0, egui::FontFamily::Proportional),
                );
                styles.text_styles.insert(
                    egui::TextStyle::Body,
                    FontId::new(18.0, egui::FontFamily::Proportional),
                );

                styles.text_styles.insert(
                    egui::TextStyle::Button,
                    FontId::new(18.0, egui::FontFamily::Proportional),
                );

                ui.heading("Enoki Editor 0.1.0");
                ui.separator();
                ui.label(format!("file: {}", effect_channel.last_file_name));

                ui.separator();
                if ui.button("Save Effect").clicked() {
                    let effect = effect_instance.0.clone().unwrap_or_default();
                    file::open_save_effect_dialog(effect, effect_channel.last_file_name.clone());
                }

                ui.separator();
                if ui.button("Load Effect").clicked() {
                    file::open_load_effect_dialog(effect_channel.send.clone());
                }

                ui.separator();
                if ui
                    .button(watcher.file_name().unwrap_or("Watch shader".into()))
                    .clicked()
                {
                    shader::open_shader_dialog(watcher.clone());
                }

                ui.separator();
                if ui.button("New Shader").clicked() {
                    shader::open_shader_save(watcher.clone());
                }

                ui.separator();
                if ui.button(&texture_channel.last_file_name).clicked() {
                    file::open_load_image_dialog(texture_channel.send.clone());
                }
            });
        });

        egui::TopBottomPanel::bottom("log").show_inside(ui, |ui| {
            ui.horizontal(|ui| {
                ui.collapsing("Log - [Mouse::Middle]: pan [Mouse::Wheel]: zoom", |ui| {
                    for entry in logs.iter() {
                        let level = entry.metadata.level().to_string();
                        let msg = format!("[{}]: {}", level, entry.message);
                        ui.label(msg);
                    }
                });

                if ui.button("Clear Log").clicked() {
                    logs.clear();
                }
            });
        });

        egui::SidePanel::right("Config").show_inside(ui, |ui| {
            egui::scroll_area::ScrollArea::new([false, true]).show(ui, |ui| {
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

                ui.separator();

                if let Ok((mut camera, mut bloom)) = camera_query.get_single_mut() {
                    gui::scene_gui(ui, &mut camera, &mut bloom);
                }

                if let Some(effect) = effect_instance.0.as_mut() {
                    gui::config_gui(ui, effect, &mut state);
                }
            });
        });
    });
}

pub(crate) fn bevy_to_egui_color(color: Color) -> Color32 {
    let s = color.to_linear().to_u8_array();
    Color32::from_rgba_unmultiplied(s[0], s[1], s[2], s[3])
}

pub(crate) fn egui_to_bevy_color(color: Color32) -> Color {
    Color::LinearRgba(LinearRgba::from_f32_array(color.to_normalized_gamma_f32()))
}
