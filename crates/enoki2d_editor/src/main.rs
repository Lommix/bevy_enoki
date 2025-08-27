use bevy::diagnostic::DiagnosticsStore;
use bevy::{core_pipeline::bloom::Bloom, log::LogPlugin, prelude::*};
use bevy_egui::egui::{self, Color32, RichText};
use bevy_egui::egui::{FontFamily, FontId};
use bevy_egui::EguiPrimaryContextPass;
use bevy_enoki::prelude::*;
use bevy_pancam::{PanCam, PanCamPlugin};
use file::{EffectChannel, TextureChannel};
use log::LogBuffer;

use crate::gui::{configure_egui, egui_settings};

mod file;
mod gui;
mod log;
mod shader;

pub const VERSION: &str = env!("CARGO_PKG_VERSION");

#[derive(Component, Reflect)]
#[reflect(Component)]
struct Spawner;

#[derive(Resource)]
pub struct SceneSettings {
    pub show_gizmos: bool,
    pub show_grid: bool,
    pub repeat_playback: bool,
    pub move_effect: bool,
    pub move_effect_radius: f32,
    pub move_effect_speed: f32,
    pub clear_color: Color32,
    pub bloom: Option<BloomSettings>,
}

impl Default for SceneSettings {
    fn default() -> Self {
        Self {
            show_gizmos: true,
            show_grid: true,
            repeat_playback: true,
            move_effect: false,
            move_effect_radius: 150.0,
            move_effect_speed: 1.0,
            clear_color: Color32::from_rgb(3, 3, 4),
            bloom: None,
        }
    }
}

pub struct BloomSettings {
    pub intensity: f32,
    pub threshold: f32,
    pub threshold_softness: f32,
    pub low_frequency_boost: f32,
    pub high_pass_frequency: f32,
}

impl Default for BloomSettings {
    fn default() -> Self {
        Self {
            intensity: 0.3,
            low_frequency_boost: 0.7,
            high_pass_frequency: 1.0,
            threshold: 0.0,
            threshold_softness: 0.0,
        }
    }
}

#[derive(Resource, Reflect)]
#[reflect(Resource)]
pub struct EditorState {
    pub open_settings: bool,
    pub open_toolbox: bool,
    pub settings_width: f32,
    pub logs_height: f32,
}
impl Default for EditorState {
    fn default() -> Self {
        Self {
            open_settings: false,
            open_toolbox: true,
            settings_width: 0.0,
            logs_height: 0.0,
        }
    }
}

fn main() {
    App::new()
        .add_plugins((
            DefaultPlugins
                .set(LogPlugin {
                    level: bevy::log::Level::INFO,
                    filter: "wgpu=error,naga=warn".into(),
                    custom_layer: log::log_capture_layer,
                })
                .set(WindowPlugin {
                    primary_window: Some(Window {
                        fit_canvas_to_parent: true,
                        ..Default::default()
                    }),
                    ..Default::default()
                }),
            PanCamPlugin,
            EnokiPlugin,
            bevy_egui::EguiPlugin::default(),
            file::FileManagerPlugin,
            log::LogPlugin,
            shader::ShaderPlugin,
        ))
        .register_type::<Spawner>()
        .register_type::<EditorState>()
        .init_resource::<EditorState>()
        .init_resource::<SceneSettings>()
        .add_systems(
            Startup,
            (setup, update_scene, center_camera, egui_settings).chain(),
        )
        .add_plugins(bevy::diagnostic::FrameTimeDiagnosticsPlugin::default())
        .add_systems(Update, gizmo.run_if(gizmos_active))
        .add_systems(Update, move_spawner)
        .add_systems(
            Update,
            (update_scene, update_spawner).run_if(resource_changed::<SceneSettings>),
        )
        .add_systems(Update, configure_egui)
        .add_systems(
            EguiPrimaryContextPass,
            (bottom_panel, gui, left_panel, in_game_settings).chain(),
        )
        .run();
}

fn update_scene(settings: Res<SceneSettings>, mut camera_query: Query<(&mut Camera, &mut Bloom)>) {
    for (mut camera, mut bloom) in camera_query.iter_mut() {
        camera.clear_color = ClearColorConfig::Custom(egui_to_bevy_color(settings.clear_color));
        match &settings.bloom {
            Some(settings) => {
                bloom.intensity = settings.intensity;
                bloom.prefilter.threshold = settings.threshold;
                bloom.prefilter.threshold_softness = settings.threshold_softness;
                bloom.low_frequency_boost = settings.low_frequency_boost;
                bloom.high_pass_frequency = settings.high_pass_frequency;
            }
            _ => {
                bloom.intensity = 0.0;
            }
        }
    }
}

fn gizmos_active(settings: Res<SceneSettings>) -> bool {
    settings.show_gizmos || settings.show_grid
}


fn move_spawner(time: Res<Time>, settings: Res<SceneSettings>, mut query: Query<&mut Transform, With<Spawner>>) {
    for mut transform in query.iter_mut() {
        if settings.move_effect {
            let t = time.elapsed_secs();
            
            transform.translation.x = (t * settings.move_effect_speed).cos() * settings.move_effect_radius;
            transform.translation.y = (t * settings.move_effect_speed).sin() * settings.move_effect_radius;
            transform.translation.z = 0.0;
        } else {
            // Reset to origin when movement is disabled
            transform.translation = Vec3::ZERO;
        }
    }
}

fn setup(mut cmd: Commands, mut particle_materials: ResMut<Assets<shader::SpriteMaterial>>) {
    cmd.spawn((
        Camera {
            hdr: true,
            clear_color: ClearColorConfig::Custom(Color::BLACK),
            ..default()
        },
        Camera2d,
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
        Spawner,
        Transform::default(),
    ));
}

fn gizmo(
    mut gizmos: Gizmos,
    settings: Res<SceneSettings>,
    effect_query: Query<&ParticleEffectInstance>,
    query: Query<&Transform, With<Spawner>>,
) {
    if settings.show_grid {
        gizmos.grid_2d(
            Vec2::new(0.0, 0.0),
            UVec2::new(40, 40),
            Vec2::splat(1000.),
            Color::LinearRgba(LinearRgba::WHITE.with_alpha(0.02)),
        );
    }
    if !settings.show_gizmos {
        return;
    }
    let Ok(effect_instance) = effect_query.single() else {
        return;
    };
    let Some(effect) = &**effect_instance else {
        return;
    };

    for transform in query.iter() {
        match effect.emission_shape {
            EmissionShape::Point => {
                gizmos.circle_2d(
                    transform.translation.xy(),
                    2.0,
                    Color::LinearRgba(LinearRgba::RED),
                );
            }
            EmissionShape::Circle(radius) => {
                gizmos.circle_2d(
                    transform.translation.xy(),
                    radius,
                    Color::LinearRgba(LinearRgba::RED),
                );
            }
        };
    }
}

fn center_camera(
    effect_query: Query<&Transform, (With<ParticleEffectInstance>, Without<Camera>)>,
    mut camera_query: Query<&mut Transform, With<Camera>>,
) {
    let Ok(effect_query_pos) = effect_query.single() else {
        return;
    };
    let Ok(mut camera) = camera_query.single_mut() else {
        return;
    };

    camera.translation = effect_query_pos.translation.with_z(camera.translation.z);
}

fn update_spawner(
    mut effect_query: Query<(Entity, &mut ParticleSpawnerState)>,
    settings: Res<SceneSettings>,
    mut cmd: Commands,
) {
    let Ok((entity, mut state)) = effect_query.single_mut() else {
        return;
    };
    if !settings.repeat_playback {
        cmd.entity(entity).insert(OneShot::Deactivate);
    } else {
        cmd.entity(entity).remove::<OneShot>();
        state.active = true;
    }
}

fn gui(
    mut context: bevy_egui::EguiContexts,
    mut effect_query: Query<(&mut ParticleEffectInstance, &mut ParticleSpawnerState)>,
    editor_state: Res<EditorState>,
    effect_channel: Res<EffectChannel>,
    texture_channel: Res<TextureChannel>,
    #[cfg(not(target_arch = "wasm32"))] watcher: Res<shader::ShaderWatch>,
) {
    let Ok((mut effect_instance, mut state)) = effect_query.single_mut() else {
        return;
    };
    let Ok(ctx) = context.ctx_mut() else {
        return;
    };
    ctx.all_styles_mut(|style| {
        style.interaction.selectable_labels = false;
    });
    let frame = egui::Frame::canvas(&ctx.style()).inner_margin(egui::Margin::same(5));

    egui::TopBottomPanel::top("Enoki particle editor")
        .frame(frame)
        .show(ctx, |ui| {
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

                ui.label(
                    RichText::new(format!("Enoki Editor {VERSION}"))
                        .heading()
                        .strong(),
                );
                ui.separator();
                ui.menu_button("File", |ui| {
                    ui.menu_button("Effect", |ui| {
                        ui.label(format!("File: {}", effect_channel.last_file_name));
                        if ui.button("Save").clicked() {
                            let effect = effect_instance.0.clone().unwrap_or_default();
                            file::open_save_effect_dialog(
                                effect,
                                effect_channel.last_file_name.clone(),
                            );
                            ui.close_menu();
                        }
                        if ui.button("Load").clicked() {
                            file::open_load_effect_dialog(effect_channel.send.clone());
                            ui.close_menu();
                        }
                    });
                    #[cfg(not(target_arch = "wasm32"))]
                    ui.menu_button("Shader", |ui| {
                        ui.label(format!(
                            "Shader: {}",
                            watcher.file_name().unwrap_or("Default".to_string())
                        ));
                        if ui.button("New Shader").clicked() {
                            shader::open_shader_save(watcher.clone());
                            ui.close_menu();
                        }
                        if ui
                            .button(watcher.file_name().unwrap_or("Watch shader".into()))
                            .clicked()
                        {
                            shader::open_shader_dialog(watcher.clone());
                            ui.close_menu();
                        }
                    });
                    if ui.button("Load Texture").clicked() {
                        file::open_load_image_dialog(
                            texture_channel.send.clone(),
                            vec!["png".into()],
                        );
                        ui.close_menu();
                    }
                });
            });
        });
    let frame = egui::Frame::canvas(&ctx.style()).inner_margin(egui::Margin::same(15));

    let Some(effect) = effect_instance.0.as_mut() else {
        return;
    };
    egui::SidePanel::right("Config")
        .frame(frame)
        .min_width(300.0)
        .show_animated(ctx, editor_state.open_toolbox, |ui| {
            egui::scroll_area::ScrollArea::new([false, true]).show(ui, |ui| {
                gui::config_gui(ui, effect, &mut state);
            });
        });
}

pub(crate) fn open_settings(In(open): In<bool>, mut editor_state: ResMut<EditorState>) {
    editor_state.open_settings = open;
}

pub(crate) fn left_panel(
    mut editor_state: ResMut<EditorState>,
    mut context: bevy_egui::EguiContexts,
    mut settings: ResMut<SceneSettings>,
    mut cmd: Commands,
) {
    let Ok(ctx) = context.ctx_mut() else {
        return;
    };
    let frame = egui::Frame::canvas(&ctx.style()).inner_margin(egui::Margin::same(15));

    let inner_response = egui::SidePanel::left("Settings")
        .frame(frame)
        .min_width(250.0)
        .show_animated(ctx, editor_state.open_settings, |ui| {
            ui.vertical_centered_justified(|ui| {
                ui.label(RichText::new("Settings").strong().size(25.0));
                ui.separator();
                ui.add_space(10.0);

                gui::scene_gui(ui, &mut settings);

                ui.add_space(10.0);
                if ui.button("close").clicked() {
                    cmd.run_system_cached_with(open_settings, false);
                }
            });
        });

    editor_state.settings_width = inner_response
        .map(|r| r.response.rect.width())
        .unwrap_or_default();
}

pub(crate) fn in_game_settings(
    mut effect_query: Query<(&mut ParticleStore, &mut ParticleSpawnerState)>,
    mut cmd: Commands,
    mut context: bevy_egui::EguiContexts,
    editor_state: Res<EditorState>,
    mut settings: ResMut<SceneSettings>,
    mut time: ResMut<Time<Virtual>>,
    diagnostics: Res<DiagnosticsStore>,
) {
    let Ok((particles, mut state)) = effect_query.single_mut() else {
        return;
    };
    let Ok(ctx) = context.ctx_mut() else {
        return;
    };
    let particle_count = particles.len();
    let frame = egui::Frame::canvas(&ctx.style())
        .fill(Color32::from_rgba_premultiplied(0, 0, 0, 150))
        .corner_radius(8)
        .inner_margin(egui::Margin::same(8));
    let window = egui::Window::new("In-Game Settings")
        .id(egui::Id::new("in-game-settings")) // required since we change the title
        .resizable(false)
        .constrain(true)
        .collapsible(false)
        .title_bar(false)
        .scroll(false)
        .enabled(true)
        .frame(frame)
        .anchor(
            egui::Align2::LEFT_BOTTOM,
            [
                editor_state.settings_width + 20.0,
                -editor_state.logs_height - 20.0,
            ],
        );
    window.show(ctx, |ui| {
        ui.vertical(|ui| {
            let Some(frame_time) = diagnostics
                .get(&bevy::diagnostic::FrameTimeDiagnosticsPlugin::FRAME_TIME)
                .and_then(|v| v.value())
            else {
                return;
            };
            ui.label(format!("Frame time: {frame_time:.2}"));
            ui.label(format!("Particles: {particle_count}"));
            if settings.repeat_playback {
                ui.label(format!(
                    "Time: {:.2}s / {:.2}s",
                    state.timer.elapsed().as_secs_f32(),
                    state.timer.duration().as_secs_f32()
                ));
            } else if ui.button("Emit particles").clicked() {
                state.active = true;
            }
            let mut speed = time.relative_speed();
            ui.checkbox(&mut settings.repeat_playback, "Play on repeat")
                .on_hover_text("Particle preview will restart after the effect is finished");
            ui.horizontal(|ui| {
                ui.label("Playback speed");
                if ui.add(egui::Slider::new(&mut speed, (0.)..=3.0)).changed() {
                    time.set_relative_speed(speed);
                }
            });
            ui.add_space(15.0);
            let button = ui
                .add(
                    egui::Button::new(
                        egui::RichText::new("Center Camera")
                            .font(FontId::new(13.0, FontFamily::Proportional)),
                    )
                    .frame(false)
                    .fill(egui::Color32::from_white_alpha(10)),
                )
                .on_hover_text("Center camera on particle");
            if button.clicked() {
                cmd.run_system_cached(center_camera);
            }
            ui.checkbox(&mut settings.show_gizmos, "Display Particle Gizmos");
            ui.checkbox(&mut settings.show_grid, "Display Grid");
        });
    });
}

pub(crate) fn bottom_panel(
    mut cmd: Commands,
    logs: Res<LogBuffer>,
    mut context: bevy_egui::EguiContexts,
    mut editor_state: ResMut<EditorState>,
) {
    let Ok(ctx) = context.ctx_mut() else {
        return;
    };
    let frame = egui::Frame::canvas(&ctx.style()).inner_margin(egui::Margin::same(5));
    let response = egui::TopBottomPanel::bottom("log")
        .frame(frame)
        .show(ctx, |ui| {
            ui.horizontal(|ui| {
                let text = "Log - [Mouse::Middle]: pan [Mouse::Wheel]: zoom";
                if logs.is_empty() {
                    ui.label(text);
                } else {
                    ui.collapsing(text, |ui| {
                        for entry in logs.iter() {
                            let msg = format!("[{}]: {}", entry.metadata.level(), entry.message);
                            ui.label(msg);
                        }
                    });
                    if ui.button("Clear Log").clicked() {
                        cmd.run_system_cached(log::clear_logs);
                    }
                }
                if ui.button("Settings").clicked() {
                    cmd.run_system_cached_with(open_settings, !editor_state.open_settings);
                }
            });
        });
    editor_state.logs_height = response.response.rect.height();
}

pub(crate) fn bevy_to_egui_color(color: Color) -> Color32 {
    let s = color.to_linear().to_u8_array();
    Color32::from_rgba_unmultiplied(s[0], s[1], s[2], s[3])
}

pub(crate) fn egui_to_bevy_color(color: Color32) -> Color {
    Color::LinearRgba(LinearRgba::from_f32_array(color.to_normalized_gamma_f32()))
}
