use bevy::{input::mouse::MouseWheel, prelude::*, window::PrimaryWindow};
use bevy_egui::{egui, EguiContext};
use bevy_enoki::prelude::*;

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
    cmd.spawn(Camera2dBundle::default());

    let effect = effects.add(Particle2dEffect {
        spawn_rate: 0.2,
        spawn_amount: 5,
        lifetime: Rval::new(3., 0.),
        direction: Some(Rval::new(Vec2::Y, 0.2)),
        linear_speed: Some(Rval::new(50., 0.5)),
        scale: Some(Rval::new(5.0, 0.8)),
        ..default()
    });

    cmd.spawn(ParticleSpawnerBundle {
        material: DEFAULT_MATERIAL,
        effect,
        ..default()
    });
}

fn gui(world: &mut World) {
    let Ok(egui_context) = world
        .query_filtered::<&mut EguiContext, With<PrimaryWindow>>()
        .get_single(world)
    else {
        return;
    };

    let mut context = egui_context.clone();

    egui::Window::new("Particle Config").show(context.get_mut(), |ui| {

        file::file_panel(world, ui);
        gui::effect_gui(world, ui);
    });
}
