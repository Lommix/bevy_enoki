use bevy::{
    core_pipeline::bloom::Bloom, input::mouse::MouseWheel, prelude::*,
    render::render_resource::AsBindGroup,
};
pub(crate) use bevy_egui::egui::{self, Color32};
use bevy_enoki::prelude::*;

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

fn setup(mut cmd: Commands, mut particle_materials: ResMut<Assets<ColorParticle2dMaterial>>) {
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

    cmd.spawn((
        ParticleSpawner(particle_materials.add(ColorParticle2dMaterial::default())),
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
    // mut file: ResMut<FileResource>, // world: &mut World,
    mut camera_query: Query<(&mut Camera, &mut Bloom)>,
    mut one_shot_mode: Local<bool>,
) {
    let Ok((entity, mut effect_instance, mut state)) = effect_query.get_single_mut() else {
        return;
    };

    let central = egui::CentralPanel::default().frame(egui::Frame { ..default() });

    central.show(context.ctx_mut(), |ui| {
        egui::TopBottomPanel::top("Enoki particle editor").show_inside(ui, |ui| {
            ui.horizontal(|ui| {
                ui.style_mut().spacing.item_spacing = [4., 4.].into();
                ui.label("Enoki Editor");

                if ui.button("export effect").clicked() {
                    info!("exporting!");
                }

                if ui.button("import effect").clicked() {
                    info!("importing!");
                }

                if ui.button("select shader").clicked() {
                    info!("importing!");
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
