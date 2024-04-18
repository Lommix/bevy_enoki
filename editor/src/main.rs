use bevy::{
    core_pipeline::bloom::BloomSettings, input::mouse::MouseWheel, prelude::*,
    window::PrimaryWindow,
};
use bevy_egui::{
    egui::{self, Color32},
    EguiContext,
};
use bevy_enoki::prelude::*;
use file::FileResource;

mod file;
mod gui;

fn main() {
    App::new()
        .add_plugins((
            DefaultPlugins,
            EnokiPlugin,
            bevy_egui::EguiPlugin,
            file::FileManagerPlugin,
        ))
        .add_systems(Startup, setup)
        .add_systems(Update, (gui, zoom))
        .run()
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

fn setup(mut cmd: Commands, mut effects: ResMut<Assets<Particle2dEffect>>) {
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

    let fireworks = include_str!("../../assets/firework.particle.ron");
    let effect: Particle2dEffect = ron::de::from_str(fireworks).unwrap();
    let effect = effects.add(effect);

    cmd.spawn(ParticleSpawnerBundle {
        material: DEFAULT_MATERIAL,
        effect,
        ..default()
    });
}

fn gui(
    mut context: bevy_egui::EguiContexts,
    mut effect_query: Query<&mut ParticleEffectInstance>,
    mut file: ResMut<FileResource>, // world: &mut World,
    mut camera_query: Query<(&mut Camera, &mut BloomSettings)>,
) {
    let Ok(mut effect_instance) = effect_query.get_single_mut() else {
        return;
    };

    egui::Window::new("Particle Config")
        .scroll2([false, true])
        .show(context.ctx_mut(), |ui| {
            file::file_panel(ui, &mut effect_instance, &mut file);
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
                gui::base_values(ui, effect);
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
