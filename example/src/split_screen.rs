/// ----------------------------------------------
/// like dynamic example, but with 1, 2, or 4 cameras in split screen.
/// ----------------------------------------------
use bevy::{
    core_pipeline::bloom::Bloom,
    diagnostic::{DiagnosticsStore, FrameTimeDiagnosticsPlugin},
    image::ImageSamplerDescriptor,
    input::common_conditions::input_just_pressed,
    prelude::*,
    render::camera::Viewport,
    window::WindowResized,
};
use bevy_enoki::{prelude::*, EnokiPlugin};
use std::time::Duration;

fn main() {
    App::new()
        .add_plugins(DefaultPlugins.set(ImagePlugin {
            default_sampler: ImageSamplerDescriptor::nearest(),
        }))
        .add_plugins(FrameTimeDiagnosticsPlugin::default())
        .add_plugins(EnokiPlugin)
        .init_resource::<CameraLayout>()
        .add_systems(Startup, (setup_cameras, setup))
        .add_systems(Update, (show_fps, change_dynamic, move_camera))
        .add_systems(
            Update,
            (
                (toggle_camera_layout, setup_cameras)
                    .chain()
                    .run_if(input_just_pressed(KeyCode::F1)),
                set_camera_viewports
                    .run_if(on_event::<WindowResized>.or(resource_changed::<CameraLayout>))
                    .after(setup_cameras),
            ),
        )
        .run();
}

#[derive(Component)]
pub struct FpsText;

#[derive(Deref, Component, DerefMut)]
pub struct ChangeTimer(Timer);

#[derive(Deref, Component, DerefMut)]
pub struct Pcindex(f32);

#[derive(Deref, Resource, DerefMut)]
pub struct ParticleMaterialAsset(Handle<SpriteParticle2dMaterial>);

#[derive(Component, Default, Debug, Reflect)]
struct CameraPositioning {
    offset_right: bool,
    offset_down: bool,
    half_width: bool,
    half_height: bool,
}

#[derive(Component)]
struct CamMarker1;

#[derive(Component)]
struct CamMarker2;

#[derive(Component)]
struct CamMarker3;

#[derive(Component)]
struct CamMarker4;

// TODO with Single as default, crashes when changing to VerticalSplit.
// (and if 2-player vertical split as default, crashes when changing to quad)
#[derive(Resource, Debug, Default, Clone, PartialEq, Eq, Hash, Reflect)]
pub enum CameraLayout {
    /// Single player game
    Single,
    /// 2 player vertical split game
    VerticalSplit,
    /// 4 player split
    #[default]
    QuadSplit,
}

fn toggle_camera_layout(mut cam_layout: ResMut<CameraLayout>) {
    info!("Toggling camera layout");
    *cam_layout = match *cam_layout {
        CameraLayout::Single => CameraLayout::VerticalSplit,
        CameraLayout::VerticalSplit => CameraLayout::QuadSplit,
        CameraLayout::QuadSplit => CameraLayout::Single,
    };
}

fn camera_bundle(order: isize) -> impl Bundle {
    (
        Camera2d,
        Camera {
            clear_color: ClearColorConfig::Custom(Color::BLACK),
            hdr: true,
            // set order to render cameras with different priorities, to prevent ambiguities
            order,
            ..default()
        },
        Bloom {
            intensity: 0.1,
            ..default()
        },
    )
}

// despawn all cameras and respawn the correct number based on the CameraLayout
fn setup_cameras(
    mut cmd: Commands,
    q_cameras: Query<
        Entity,
        Or<(
            With<CamMarker1>,
            With<CamMarker2>,
            With<CamMarker3>,
            With<CamMarker4>,
        )>,
    >,
    cam_layout: Res<CameraLayout>,
) {
    // despawn all existing cameras
    info!("Despawning {} cameras", q_cameras.iter().count());
    q_cameras.iter().for_each(|e| cmd.entity(e).despawn());
    info!("Spawning cameras for {:?}", *cam_layout);
    // spawn the correct number of cameras for the number of local players
    match *cam_layout {
        CameraLayout::Single => {
            info!("Spawning 1 camera");
            cmd.spawn((camera_bundle(1), CameraPositioning::default(), CamMarker1));
        }
        CameraLayout::VerticalSplit => {
            info!("Spawning 2 cameras");
            cmd.spawn((
                camera_bundle(1),
                CameraPositioning {
                    half_width: true,
                    ..default()
                },
                CamMarker1,
            ));
            cmd.spawn((
                camera_bundle(2),
                CameraPositioning {
                    half_width: true,
                    offset_right: true,
                    ..default()
                },
                CamMarker2,
            ));
        }
        CameraLayout::QuadSplit => {
            info!("Spawning 4 cameras");
            cmd.spawn((
                camera_bundle(1),
                CameraPositioning {
                    half_width: true,
                    half_height: true,
                    ..default()
                },
                CamMarker1,
            ));
            cmd.spawn((
                camera_bundle(2),
                CameraPositioning {
                    offset_right: true,
                    half_width: true,
                    half_height: true,
                    ..default()
                },
                CamMarker2,
            ));
            cmd.spawn((
                camera_bundle(3),
                CameraPositioning {
                    half_width: true,
                    half_height: true,
                    offset_down: true,
                    ..default()
                },
                CamMarker3,
            ));
            cmd.spawn((
                camera_bundle(4),
                CameraPositioning {
                    half_width: true,
                    half_height: true,
                    offset_right: true,
                    offset_down: true,
                    ..default()
                },
                CamMarker4,
            ));
        }
    }
}

// We trigger this if a WindowResized event is received, or if the CameraLayout changes.
fn set_camera_viewports(
    mut resize_events: EventReader<WindowResized>,
    q_windows: Query<(Entity, &Window)>,
    mut q_cam: Query<(&CameraPositioning, &mut Camera)>,
) {
    info!("Setting camera viewports");
    // we want to process window entities that were resized, or if none were resized, do them all.
    let mut windows_to_process = resize_events.read().map(|e| e.window).collect::<Vec<_>>();
    if windows_to_process.is_empty() {
        windows_to_process.extend(q_windows.iter().map(|t| t.0));
    }

    // We need to dynamically resize the camera's viewports whenever the window size changes
    // so that each camera takes up half the screen (or whatever the layout is)
    // A resize_event is sent when the window is first created, allowing us to reuse this system for initial setup.
    for window_entity in windows_to_process.iter() {
        let window = q_windows.get(*window_entity).unwrap().1;
        let size = window.physical_size();
        let half_x = size.x / 2;
        let half_y = size.y / 2;
        // info!(
        //     "Window physical size: {:?}, half_x: {:?}, half_y: {:?}",
        //     size, half_x, half_y
        // );
        for (campos, mut camera) in &mut q_cam {
            let mut physical_position = UVec2::ZERO;
            let mut physical_size = size;
            if campos.offset_right {
                physical_position.x = half_x;
            }
            if campos.offset_down {
                physical_position.y = half_y;
            }
            if campos.half_width {
                physical_size.x = half_x;
            }
            if campos.half_height {
                physical_size.y = half_y;
            }
            let viewport = Viewport {
                physical_position,
                physical_size,
                ..default()
            };
            info!(
                "Setting viewport for camera {:?}: {:?}",
                camera.order, viewport
            );
            camera.viewport = Some(viewport);
        }
    }
}

fn setup(
    mut cmd: Commands,
    mut materials: ResMut<Assets<SpriteParticle2dMaterial>>,
    server: Res<AssetServer>,
) {
    cmd.spawn((
        ChangeTimer(Timer::new(Duration::from_millis(300), TimerMode::Repeating)),
        Pcindex(0.),
    ));

    cmd.spawn((
        Text::default(),
        TextFont {
            font_size: 24.,
            ..default()
        },
        FpsText,
    ));

    let material_handle = materials.add(SpriteParticle2dMaterial::from_texture(
        server.load("enoki.png"),
    ));

    cmd.spawn((
        ParticleSpawner(material_handle),
        ParticleEffectHandle(server.load("base.particle.ron")),
    ));
}

fn change_dynamic(
    mut elapsed: Local<f32>,
    mut query: Query<&mut ParticleEffectInstance>,
    time: Res<Time>,
) {
    *elapsed += time.delta_secs();

    let Ok(mut maybe_effect) = query.single_mut() else {
        return;
    };

    if let Some(effect) = maybe_effect.0.as_mut() {
        effect.linear_speed = Some(Rval::new(1000. * elapsed.sin().abs(), 0.1));
        effect.spawn_amount = 100;
    }
}

fn show_fps(
    diagnostics: Res<DiagnosticsStore>,
    particles: Query<&ParticleStore>,
    mut texts: Query<&mut Text, With<FpsText>>,
) {
    let Some(fps) = diagnostics
        .get(&bevy::diagnostic::FrameTimeDiagnosticsPlugin::FPS)
        .map(|v| v.value())
        .flatten()
    else {
        return;
    };

    let particle_count: usize = particles.iter().map(|store| store.len()).sum();

    let Ok(mut text) = texts.single_mut() else {
        return;
    };

    text.0 = format!(
        "F1: Toggle Camera Layout\nO:ZoomOut I:ZoomIn Arrow:Move\nFPS: {:.1}\nParticles: {}",
        fps, particle_count
    );
}

fn move_camera(
    mut cam: Query<&mut Transform, With<Camera>>,
    inputs: Res<ButtonInput<KeyCode>>,
    time: Res<Time>,
) {
    let x = inputs.pressed(KeyCode::ArrowRight) as i32 - inputs.pressed(KeyCode::ArrowLeft) as i32;
    let y = inputs.pressed(KeyCode::ArrowUp) as i32 - inputs.pressed(KeyCode::ArrowDown) as i32;

    let zoom = inputs.pressed(KeyCode::KeyO) as i32 - inputs.pressed(KeyCode::KeyI) as i32;

    cam.iter_mut().for_each(|mut t| {
        t.translation.x += x as f32 * 300. * time.delta_secs();
        t.translation.y += y as f32 * 300. * time.delta_secs();
        t.scale = (t.scale + (zoom as f32) * 0.1).max(Vec3::splat(0.1));
    });
}
