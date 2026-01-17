use bevy::{
    diagnostic::DiagnosticsStore, post_process::bloom::Bloom, prelude::*, render::view::Hdr,
    text::LineHeight,
};
use bevy_enoki::prelude::ParticleStore;

#[derive(Component, Reflect)]
#[require(Text)]
pub struct FpsText;

#[derive(Component, Reflect)]
#[require(Text)]
pub struct ParticlesText;

pub(crate) fn camera_and_ui_plugin(app: &mut App) {
    app.add_systems(PostStartup, spawn_ui)
        .add_plugins(bevy::diagnostic::FrameTimeDiagnosticsPlugin::default())
        .add_systems(Update, (show_fps, move_camera));
}
fn spawn_ui(mut cmd: Commands) {
    cmd.spawn((
        Camera2d,
        Camera {
            clear_color: ClearColorConfig::Custom(Color::BLACK),
            ..default()
        },
        Hdr,
        Bloom {
            intensity: 0.1,
            ..default()
        },
    ));
    cmd.spawn(debug_ui());
}
fn show_fps(
    diagnostics: Res<DiagnosticsStore>,
    particles: Query<&ParticleStore>,
    mut texts: Query<&mut Text, With<FpsText>>,
    mut particles_texts: Query<&mut Text, (Without<FpsText>, With<ParticlesText>)>,
) {
    let Some(fps) = diagnostics
        .get(&bevy::diagnostic::FrameTimeDiagnosticsPlugin::FPS)
        .and_then(|v| v.value())
    else {
        return;
    };

    let particle_count: usize = particles.iter().map(|store| store.len()).sum();
    for mut text in texts.iter_mut() {
        text.0 = format!("{fps:.1}");
    }
    for mut text in particles_texts.iter_mut() {
        text.0 = format!("{particle_count}");
    }
}

fn debug_ui() -> impl Bundle {
    let font = TextFont::from_font_size(20.);
    let description_font = TextFont::from_font_size(16.);
    let row_node = Node {
        flex_direction: FlexDirection::Row,
        justify_content: JustifyContent::SpaceAround,
        margin: UiRect::bottom(px(6.0)),
        ..Default::default()
    };
    let description_node = Node {
        margin: UiRect::right(px(10.0)),
        width: Val::Px(100.0),
        ..Default::default()
    };
    let text_shadow = TextShadow {
        offset: Vec2::new(1.0, 3.0),
        color: bevy::color::palettes::tailwind::GRAY_800
            .with_alpha(0.5)
            .into(),
    };
    let secondary_color = TextColor(bevy::color::palettes::tailwind::GRAY_400.into());
    let primary_color = TextColor(bevy::color::palettes::tailwind::VIOLET_100.into());
    let secondary_bundle = |txt: &'static str| {
        (
            Text::new(txt),
            secondary_color,
            text_shadow,
            description_font.clone(),
            description_node.clone(),
            LineHeight::Px(24.),
        )
    };
    let value_bundle = || {
        (
            font.clone(),
            LineHeight::Px(24.),
            text_shadow,
            primary_color,
            TextLayout::new_with_justify(Justify::Right),
            Node {
                min_width: Val::Px(50.0),
                ..Default::default()
            },
        )
    };

    let frame_bundle = |is_top: bool| {
        (
            Node {
                flex_direction: FlexDirection::Column,
                padding: UiRect::all(px(8.0)),
                left: px(10.0),
                top: if is_top { px(10.0) } else { Val::Auto },
                bottom: if is_top { Val::Auto } else { px(10.0) },
                position_type: PositionType::Absolute,
                border: UiRect::all(px(5.0)),
                border_radius: BorderRadius::all(px(10.0)),
                ..Default::default()
            },
            BorderColor::all(bevy::color::palettes::tailwind::NEUTRAL_600.with_alpha(0.3)),
            BackgroundColor(
                bevy::color::palettes::tailwind::NEUTRAL_900
                    // .with_alpha(0.8)
                    .into(),
            ),
        )
    };
    (
        Node {
            width: Val::Percent(100.0),
            height: Val::Percent(100.0),
            ..Default::default()
        },
        children![
            (
                frame_bundle(true),
                children![
                    (
                        row_node.clone(),
                        children![
                            secondary_bundle("Particles"),
                            (value_bundle(), ParticlesText)
                        ]
                    ),
                    (
                        row_node.clone(),
                        children![secondary_bundle("FPS"), (value_bundle(), FpsText)]
                    )
                ],
            ),
            (
                frame_bundle(false),
                children![
                    (
                        Text(String::from("Camera Controls")),
                        font.clone(),
                        primary_color,
                        text_shadow,
                        row_node.clone(),
                    ),
                    (
                        row_node.clone(),
                        children![
                            secondary_bundle("Zoom out"),
                            (value_bundle(), Text::new("O")),
                        ],
                    ),
                    (
                        row_node.clone(),
                        children![
                            secondary_bundle("Zoom in"),
                            (value_bundle(), Text::new("I")),
                        ],
                    ),
                    (
                        row_node.clone(),
                        children![
                            secondary_bundle("Move"),
                            (value_bundle(), Text::new("WSAD")),
                        ],
                    )
                ],
            ),
        ],
    )
}

fn move_camera(
    mut cam: Query<&mut Transform, With<Camera>>,
    inputs: Res<ButtonInput<KeyCode>>,
    time: Res<Time>,
) {
    let is_pressed = |keys: &[KeyCode]| keys.iter().any(|key| inputs.pressed(*key)) as i32;
    let x = is_pressed(&[KeyCode::ArrowRight, KeyCode::KeyD])
        - is_pressed(&[KeyCode::ArrowLeft, KeyCode::KeyA]);
    let y = is_pressed(&[KeyCode::ArrowUp, KeyCode::KeyW])
        - is_pressed(&[KeyCode::ArrowDown, KeyCode::KeyS]);

    let zoom = is_pressed(&[KeyCode::KeyO]) - is_pressed(&[KeyCode::KeyI]);

    cam.iter_mut().for_each(|mut t| {
        t.translation.x += x as f32 * 300. * time.delta_secs();
        t.translation.y += y as f32 * 300. * time.delta_secs();
        t.scale = (t.scale + (zoom as f32) * 0.1).max(Vec3::splat(0.1));
    });
}
