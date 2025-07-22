#![allow(unused)]
use bevy::{core_pipeline::bloom::Bloom, prelude::*};
use bevy_egui::{
    egui::{self, RichText, Ui, WidgetText},
    EguiContext, EguiContexts, EguiGlobalSettings,
};
use bevy_enoki::prelude::*;
use egui_plot::{Line, PlotPoints};

use crate::{bevy_to_egui_color, egui_to_bevy_color, BloomSettings, SceneSettings};

pub(crate) fn scene_gui(ui: &mut Ui, settings: &mut SceneSettings) {
    egui::Grid::new("scene_setting")
        .num_columns(2)
        .show(ui, |ui| {
            ui.label("Scene background");
            egui::color_picker::color_edit_button_srgba(
                ui,
                &mut settings.clear_color,
                egui::color_picker::Alpha::Opaque,
            );
            ui.end_row();
            ui.label(RichText::new("Bloom Settings").strong());
            ui.end_row();
            match &mut settings.bloom {
                Some(bloom) => {
                    ui.label("Intensity");
                    ui.add(egui::Slider::new(&mut bloom.intensity, (0.)..=1.));
                    ui.end_row();
                    ui.label("Threshold");
                    ui.add_enabled(
                        bloom.intensity > 0.01,
                        egui::Slider::new(&mut bloom.threshold, (0.)..=1.),
                    );
                    ui.end_row();

                    ui.label("Softness");

                    ui.add_enabled(
                        bloom.intensity > 0.01,
                        egui::Slider::new(&mut bloom.threshold_softness, (0.)..=1.),
                    );
                    ui.end_row();

                    ui.label("low freq");

                    ui.add_enabled(
                        bloom.intensity > 0.01,
                        egui::Slider::new(&mut bloom.low_frequency_boost, (0.)..=1.),
                    );
                    ui.end_row();

                    ui.label("high freq");

                    ui.add_enabled(
                        bloom.intensity > 0.01,
                        egui::Slider::new(&mut bloom.high_pass_frequency, (0.001)..=1.),
                    );
                    ui.end_row();
                }
                None => {
                    if ui.button("Enable bloom").clicked() {
                        settings.bloom = Some(BloomSettings::default());
                    }
                }
            }
        });
}

#[derive(PartialEq, Eq, Clone, Copy, Debug)]
enum Shape {
    Point,
    Circle,
}

impl From<Shape> for &'static str {
    fn from(val: Shape) -> Self {
        match val {
            Shape::Point => "Point",
            Shape::Circle => "Circle",
        }
    }
}

impl From<Shape> for WidgetText {
    fn from(val: Shape) -> Self {
        let s: &'static str = val.into();
        s.into()
    }
}

impl From<&EmissionShape> for Shape {
    fn from(value: &EmissionShape) -> Self {
        match value {
            EmissionShape::Point => Self::Point,
            EmissionShape::Circle(_) => Self::Circle,
        }
    }
}

fn collapsing_header(text: impl Into<String>) -> egui::CollapsingHeader {
    egui::CollapsingHeader::new(RichText::new(text).strong().size(19.0)).default_open(true)
}

pub(crate) fn config_gui(
    ui: &mut Ui,
    effect: &mut Particle2dEffect,
    state: &mut ParticleSpawnerState,
) {
    ui.add_space(10.0);
    collapsing_header("Spawner Settings").show(ui, |ui| {
        // ui.checkbox(&mut state.active, "Active");
        ui.horizontal(|ui| {
            ui.label("Amount");
            ui.add(egui::DragValue::new(&mut effect.spawn_amount).range(1..=9999));
        });
        ui.horizontal(|ui| {
            ui.label("Rate");
            ui.add(egui::Slider::new(&mut effect.spawn_rate, (0.01)..=2.));
        });
        ui.label("Emission type");

        ui.horizontal(|ui| {
            ui.style_mut().spacing.item_spacing = [4., 4.].into();
            let mut shape: Shape = (&effect.emission_shape).into();
            let before: Shape = shape;
            egui::ComboBox::from_label("")
                .selected_text(shape)
                .show_ui(ui, |ui| {
                    ui.selectable_value(&mut shape, Shape::Point, Shape::Point);
                    ui.selectable_value(&mut shape, Shape::Circle, Shape::Circle);
                });
            if before != shape {
                effect.emission_shape = match shape {
                    Shape::Point => EmissionShape::Point,
                    Shape::Circle => EmissionShape::Circle(15.0),
                };
            } else {
                let mut val = match effect.emission_shape {
                    EmissionShape::Point => {
                        return;
                    }
                    EmissionShape::Circle(val) => val,
                };
                if ui.add(egui::DragValue::new(&mut val)).changed() {
                    effect.emission_shape = EmissionShape::Circle(val);
                }
            }
        });
        rval_f32_field(ui, "Lifetime", &mut effect.lifetime);
    });
    ui.separator();
    collapsing_header("Linear velocity").show(ui, |ui| {
        if let Some(mut direction) = effect.direction.as_mut() {
            rval_vec2_field(ui, "Direction", direction);
        } else {
            effect.direction = Some(Rval::default());
        }

        if let Some(mut speed) = effect.linear_speed.as_mut() {
            rval_f32_field(ui, "Speed", speed);
        } else {
            effect.linear_speed = Some(Rval::default());
        }

        if let Some(mut damp) = effect.linear_damp.as_mut() {
            rval_f32_field(ui, "Damp", damp);
        } else {
            effect.linear_damp = Some(Rval::default());
        }

        if let Some(mut accel) = effect.linear_acceleration.as_mut() {
            rval_f32_field(ui, "Accel", accel);
        } else {
            effect.linear_acceleration = Some(Rval::default());
        }
    });
    ui.separator();
    collapsing_header("Angular velocity").show(ui, |ui| {
        if let Some(mut speed) = effect.angular_speed.as_mut() {
            rval_f32_field(ui, "Speed", speed);
        } else {
            effect.angular_speed = Some(Rval::default());
        }

        if let Some(mut damp) = effect.angular_damp.as_mut() {
            rval_f32_field(ui, "Damp", damp);
        } else {
            effect.angular_damp = Some(Rval::default());
        }

        if let Some(mut accel) = effect.angular_acceleration.as_mut() {
            rval_f32_field(ui, "Accel", accel);
        } else {
            effect.angular_acceleration = Some(Rval::default());
        }
    });

    ui.separator();
    collapsing_header("Gravity").show(ui, |ui| {
        if let Some(mut gravity_direction) = effect.gravity_direction.as_mut() {
            rval_vec2_field(ui, "Direction", gravity_direction);
        } else {
            effect.gravity_direction = Some(Rval::default());
        }

        if let Some(mut grav) = effect.gravity_speed.as_mut() {
            rval_f32_field(ui, "Speed", grav);
        } else {
            effect.gravity_speed = Some(Rval::default());
        }
    });

    ui.separator();
    collapsing_header("Scale").show(ui, |ui| {
        if let Some(scale_curve) = effect.scale_curve.as_mut() {
            curve_field_f32(ui, scale_curve);
        } else {
            if let Some(mut scale) = effect.scale.as_mut() {
                rval_f32_field(ui, "Init Scale", scale);
            } else {
                effect.scale = Some(Rval::default());
            }

            if ui.button("Add Scale Curve").clicked() {
                let curve = bevy_enoki::prelude::MultiCurve::new()
                    .with_point(1.0, 0.0, None)
                    .with_point(1.0, 1.0, None);
                effect.scale_curve = Some(curve);
            }
        }
    });

    ui.separator();
    collapsing_header("Color").show(ui, |ui| {
        if let Some(color_curve) = effect.color_curve.as_mut() {
            curve_field_color(ui, color_curve);
        } else if ui.button("Add Color Curve").clicked() {
            let curve = bevy_enoki::prelude::MultiCurve::new()
                .with_point(LinearRgba::WHITE, 0.0, None)
                .with_point(LinearRgba::WHITE, 1.0, None);
            effect.color_curve = Some(curve);
        }
    });
}

fn rval_f32_field(ui: &mut Ui, label: &str, field: &mut Rval<f32>) {
    egui::Grid::new(label)
        .spacing([4., 4.])
        .min_col_width(80.)
        .num_columns(2)
        .show(ui, |ui| {
            ui.label(label);
            ui.add(egui::DragValue::new(&mut field.0).range((0.)..=9999.));

            ui.end_row();

            ui.label(format!("{label} Rand"));
            ui.add(egui::Slider::new(&mut field.1, (0.)..=1.));
        });
}

fn rval_vec2_field(ui: &mut Ui, label: &str, field: &mut Rval<Vec2>) {
    egui::Grid::new(label)
        .spacing([4., 4.])
        .min_col_width(80.)
        .num_columns(2)
        .show(ui, |ui| {
            ui.label("Dir X");
            ui.add(egui::Slider::new(&mut field.0.x, (-1.)..=1.));
            ui.end_row();

            ui.label("Dir Y");
            ui.add(egui::Slider::new(&mut field.0.y, (-1.)..=1.));
            ui.end_row();

            ui.label("Randomness");
            ui.add(egui::Slider::new(&mut field.1, (0.)..=1.));
            ui.end_row();
        });
}

fn curve_field_f32(ui: &mut Ui, curve: &mut bevy_enoki::prelude::MultiCurve<f32>) {
    let sin: PlotPoints = (0..100)
        .map(|i| {
            let x = i as f64 * 0.01;
            let y = curve.lerp(x as f32);
            [x, y as f64]
        })
        .collect();
    let line = Line::new("sin", sin);
    egui_plot::Plot::new("curve_f32")
        .height(100.)
        .allow_drag(false)
        .allow_double_click_reset(false)
        .allow_zoom(false)
        .allow_scroll(false)
        .show(ui, |ui| {
            ui.line(line);
        });

    let mut remove = Vec::new();
    curve
        .points
        .iter_mut()
        .enumerate()
        .for_each(|(i, (val, pos, easing))| {
            egui::Grid::new(format!("p_{i}")).show(ui, |ui| {
                ui.label("value");
                ui.add(egui::DragValue::new(val).range((0.)..=9999.0));
                if ui.button("Delete").clicked() {
                    remove.push(i);
                }

                ui.end_row();
                ui.label("position");
                ui.add(egui::Slider::new(pos, (0.)..=1.0));
                easing_select(ui, format!("easing_{i}"), easing);
            });

            ui.separator();
        });

    remove.drain(..).for_each(|i| {
        curve.points.remove(i);
    });

    curve.sort();

    if ui.button("Add Point").clicked() {
        curve.points.push((0.1, 1.0, None));
    }
}

fn curve_field_color(ui: &mut Ui, curve: &mut bevy_enoki::prelude::MultiCurve<LinearRgba>) {
    let mut remove = Vec::new();
    curve
        .points
        .iter_mut()
        .enumerate()
        .for_each(|(i, (val, pos, easing))| {
            egui::Grid::new(format!("p_{i}")).show(ui, |ui| {
                ui.label("value");

                let mut rgba =
                    egui::Rgba::from_rgba_premultiplied(val.red, val.green, val.blue, val.alpha);

                egui::color_picker::color_edit_button_rgba(
                    ui,
                    &mut rgba,
                    egui::color_picker::Alpha::Opaque,
                );

                val.red = rgba.r();
                val.green = rgba.g();
                val.blue = rgba.b();
                val.alpha = rgba.a();

                if ui.button("Delete").clicked() {
                    remove.push(i);
                }

                ui.end_row();
                ui.label("position");
                ui.add(egui::Slider::new(pos, (0.)..=1.0));
                easing_select(ui, format!("easing_{i}"), easing);
            });

            ui.separator();
        });

    remove.drain(..).for_each(|i| {
        curve.points.remove(i);
    });

    curve.sort();

    if ui.button("Add Point").clicked() {
        curve.points.push((LinearRgba::WHITE, 1.0, None));
    }
}

fn easing_select(ui: &mut Ui, id: impl std::hash::Hash, easing: &mut Option<EaseFunction>) {
    egui::ComboBox::new(id, "")
        .selected_text(ron::ser::to_string(easing).unwrap())
        .show_ui(ui, |ui| {
            ui.selectable_value(easing, None, "None");
            ui.selectable_value(easing, Some(EaseFunction::QuarticIn), "QuarticIn");
            ui.selectable_value(easing, Some(EaseFunction::QuadraticIn), "QuadraticIn");
            ui.selectable_value(easing, Some(EaseFunction::QuadraticOut), "QuadraticOut");
            ui.selectable_value(easing, Some(EaseFunction::QuadraticInOut), "QuadraticInOut");
            ui.selectable_value(easing, Some(EaseFunction::CubicIn), "CubicIn");
            ui.selectable_value(easing, Some(EaseFunction::CubicOut), "CubicOut");
            ui.selectable_value(easing, Some(EaseFunction::CubicInOut), "CubicInOut");
            ui.selectable_value(easing, Some(EaseFunction::QuarticIn), "QuarticIn");
            ui.selectable_value(easing, Some(EaseFunction::QuarticOut), "QuarticOut");
            ui.selectable_value(easing, Some(EaseFunction::QuarticInOut), "QuarticInOut");
            ui.selectable_value(easing, Some(EaseFunction::QuinticIn), "QuinticIn");
            ui.selectable_value(easing, Some(EaseFunction::QuinticOut), "QuinticOut");
            ui.selectable_value(easing, Some(EaseFunction::QuinticInOut), "QuinticInOut");
            ui.selectable_value(easing, Some(EaseFunction::SineIn), "SineIn");
            ui.selectable_value(easing, Some(EaseFunction::SineOut), "SineOut");
            ui.selectable_value(easing, Some(EaseFunction::SineInOut), "SineInOut");
            ui.selectable_value(easing, Some(EaseFunction::CircularIn), "CircularIn");
            ui.selectable_value(easing, Some(EaseFunction::CircularOut), "CircularOut");
            ui.selectable_value(easing, Some(EaseFunction::CircularInOut), "CircularInOut");
            ui.selectable_value(easing, Some(EaseFunction::ExponentialIn), "ExponentialIn");
            ui.selectable_value(easing, Some(EaseFunction::ExponentialOut), "ExponentialOut");
            ui.selectable_value(
                easing,
                Some(EaseFunction::ExponentialInOut),
                "ExponentialInOut",
            );
            ui.selectable_value(easing, Some(EaseFunction::ElasticIn), "ElasticIn");
            ui.selectable_value(easing, Some(EaseFunction::ElasticOut), "ElasticOut");
            ui.selectable_value(easing, Some(EaseFunction::ElasticInOut), "ElasticInOut");
            ui.selectable_value(easing, Some(EaseFunction::BackIn), "BackIn");
            ui.selectable_value(easing, Some(EaseFunction::BackOut), "BackOut");
            ui.selectable_value(easing, Some(EaseFunction::BackInOut), "BackInOut");
            ui.selectable_value(easing, Some(EaseFunction::BounceIn), "BounceIn");
            ui.selectable_value(easing, Some(EaseFunction::BounceOut), "BounceOut");
            ui.selectable_value(easing, Some(EaseFunction::BounceInOut), "BounceInOut");
        });
}

pub(crate) fn egui_settings(mut settings: ResMut<EguiGlobalSettings>) {
    settings.enable_absorb_bevy_input_system = true;
}

pub(crate) fn configure_egui(mut contexts: Query<&mut EguiContext, Added<EguiContext>>) {
    for mut context in contexts.iter_mut() {
        #[cfg(any(target_os = "windows", target_os = "macos"))]
        {
            if let Some((regular, semibold)) = get_fonts() {
                let mut fonts = egui::FontDefinitions::default();
                fonts.font_data.insert(
                    "regular".to_owned(),
                    egui::FontData::from_owned(regular).into(),
                );
                fonts.font_data.insert(
                    "semibold".to_owned(),
                    egui::FontData::from_owned(semibold).into(),
                );

                // Put my font first (highest priority) for proportional text:
                fonts
                    .families
                    .entry(egui::FontFamily::Proportional)
                    .or_default()
                    .insert(0, "regular".to_owned());
                fonts
                    .families
                    .entry(egui::FontFamily::Name("semibold".into()))
                    .or_default()
                    .insert(0, "semibold".to_owned());

                // Put my font as last fallback for monospace:
                fonts
                    .families
                    .entry(egui::FontFamily::Monospace)
                    .or_default()
                    .push("regular".to_owned());

                // Tell egui to use these fonts:
                context.get_mut().set_fonts(fonts);
            }
        }
        context.get_mut().style_mut(|style| {
            for font_id in style.text_styles.values_mut() {
                font_id.size *= 1.3;
            }
        });
    }
}

#[cfg(target_os = "macos")]
fn get_fonts() -> Option<(Vec<u8>, Vec<u8>)> {
    use std::fs;
    let font_path = std::path::Path::new("/System/Library/Fonts");

    let regular = fs::read(font_path.join("SFNSRounded.ttf")).ok()?;
    let semibold = fs::read(font_path.join("SFCompact.ttf")).ok()?;

    Some((regular, semibold))
}

#[cfg(target_os = "windows")]
fn get_fonts() -> Option<(Vec<u8>, Vec<u8>)> {
    use std::fs;

    let app_data = std::env::var("APPDATA").ok()?;
    let font_path = std::path::Path::new(&app_data);

    let regular = fs::read(font_path.join("../Local/Microsoft/Windows/Fonts/aptos.ttf")).ok()?;
    let semibold =
        fs::read(font_path.join("../Local/Microsoft/Windows/Fonts/aptos-bold.ttf")).ok()?;

    Some((regular, semibold))
}
