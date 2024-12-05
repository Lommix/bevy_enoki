#![cfg_attr(docsrs, feature(doc_auto_cfg))]
#![allow(rustdoc::redundant_explicit_links)]
#![doc = include_str!("../../../README.md")]

use self::prelude::{
    Particle2dMaterial, ParticleEffectInstance, ParticleSpawnerState, ParticleStore,
};
use crate::sprite::SpriteParticle2dMaterial;
use bevy::{
    asset::load_internal_asset,
    prelude::*,
    render::{
        primitives::Aabb,
        sync_world::SyncToRenderWorld,
        view::{check_visibility, VisibilitySystems},
    },
};
use color::ColorParticle2dMaterial;
use serde::{Deserialize, Serialize};
use values::Rval;

mod color;
mod curve;
mod loader;
mod material;
mod sprite;
mod update;
mod values;

#[allow(unused)]
pub mod prelude {
    pub use super::color::ColorParticle2dMaterial;
    pub use super::curve::{MultiCurve/* , ParticleEaseFunction */, LerpThat};
    pub use super::loader::ParticleEffectLoader;
    pub use super::material::{Particle2dMaterial, Particle2dMaterialPlugin};
    pub use super::sprite::SpriteParticle2dMaterial;
    pub use super::update::{OneShot, ParticleEffectInstance, ParticleSpawnerState, ParticleStore};
    pub use super::values::{Random, Rval};
    pub use super::{
        EmissionShape, EnokiPlugin, NoAutoAabb, Particle2dEffect, ParticleEffectHandle,
        ParticleSpawner,
    };
}

pub(crate) const PARTICLE_VERTEX_OUT: Handle<Shader> =
    Handle::weak_from_u128(97641680653231235698756524351080664231);
pub(crate) const PARTICLE_VERTEX: Handle<Shader> =
    Handle::weak_from_u128(47641680653221245698756524350619564219);
pub(crate) const PARTICLE_COLOR_FRAG: Handle<Shader> =
    Handle::weak_from_u128(27641685611347896254665674658656433339);
pub(crate) const PARTICLE_SPRITE_FRAG: Handle<Shader> =
    Handle::weak_from_u128(52323345476969163624641232313030656999);

pub struct EnokiPlugin;
impl Plugin for EnokiPlugin {
    fn build(&self, app: &mut App) {
        load_internal_asset!(
            app,
            PARTICLE_VERTEX_OUT,
            "shaders/particle_vertex_out.wgsl",
            Shader::from_wgsl
        );

        load_internal_asset!(
            app,
            PARTICLE_VERTEX,
            "shaders/particle_vertex.wgsl",
            Shader::from_wgsl
        );

        load_internal_asset!(
            app,
            PARTICLE_COLOR_FRAG,
            "shaders/particle_color_frag.wgsl",
            Shader::from_wgsl
        );

        load_internal_asset!(
            app,
            PARTICLE_SPRITE_FRAG,
            "shaders/particle_sprite_frag.wgsl",
            Shader::from_wgsl
        );

        app.add_plugins(material::Particle2dMaterialPlugin::<SpriteParticle2dMaterial>::default());
        app.add_plugins(material::Particle2dMaterialPlugin::<ColorParticle2dMaterial>::default());

        app.register_type::<update::ParticleStore>();
        app.register_type::<update::ParticleSpawnerState>();
        app.register_type::<update::ParticleSpawnerState>();
        app.register_type::<update::Particle>();
        app.register_type::<ParticleEffectHandle>();
        app.init_asset::<Particle2dEffect>();
        app.init_asset_loader::<loader::ParticleEffectLoader>();

        app.world_mut()
            .resource_mut::<Assets<ColorParticle2dMaterial>>()
            .insert(
                &Handle::<ColorParticle2dMaterial>::default(),
                ColorParticle2dMaterial::default(),
            );

        app.world_mut()
            .resource_mut::<Assets<Particle2dEffect>>()
            .insert(
                &Handle::<Particle2dEffect>::default(),
                Particle2dEffect::default(),
            );

        app.add_systems(
            First,
            loader::on_asset_loaded.run_if(on_event::<AssetEvent<Particle2dEffect>>),
        );

        app.add_systems(
            Update,
            (
                loader::reload_effect,
                update::clone_effect,
                update::remove_finished_spawner,
                update::update_spawner,
            ),
        );

        app.add_systems(
            PostUpdate,
            (
                update::calculcate_particle_bounds.in_set(VisibilitySystems::CalculateBounds),
                check_visibility::<WithParticles>.in_set(VisibilitySystems::CheckVisibility),
            ),
        );
    }
}

pub type WithParticles = With<ParticleSpawnerState>;

/// adding this component will disabled auto
/// aabb caluclation. Aabb resolves to it's default size.
#[derive(Component)]
pub struct NoAutoAabb;

/// The main particle spawner components
/// has required components
#[derive(Component, DerefMut, Deref, Clone)]
#[require(
    ParticleSpawnerState,
    ParticleEffectInstance,
    ParticleEffectHandle,
    ParticleStore,
    Transform,
    Visibility,
    Aabb,
    SyncToRenderWorld
)]
pub struct ParticleSpawner<T: Particle2dMaterial>(pub Handle<T>);

impl<T: Particle2dMaterial> From<Handle<T>> for ParticleSpawner<T> {
    fn from(value: Handle<T>) -> Self {
        Self(value)
    }
}

impl Default for ParticleSpawner<ColorParticle2dMaterial> {
    fn default() -> Self {
        ParticleSpawner(Handle::default())
    }
}

#[derive(Deserialize, Reflect, Default, Clone, Debug, Serialize)]
#[reflect]
pub enum EmissionShape {
    #[default]
    Point,
    Circle(f32),
}

/// holds the effect asset. Changing the Asset, will
/// effect all spanwers using it. Instead use `ParticleEffectInstance`,
/// which is a unique copy for each spawner,
#[derive(Component, Reflect, Deref, DerefMut, Default)]
#[reflect]
pub struct ParticleEffectHandle(pub Handle<Particle2dEffect>);

impl From<Handle<Particle2dEffect>> for ParticleEffectHandle {
    fn from(value: Handle<Particle2dEffect>) -> Self {
        Self(value)
    }
}

/// The particle effect asset.
#[derive(Asset, TypePath, Deserialize, Serialize, Clone, Debug)]
pub struct Particle2dEffect {
    pub spawn_rate: f32,
    pub spawn_amount: u32,
    pub emission_shape: EmissionShape,
    pub lifetime: Rval<f32>,
    pub linear_speed: Option<Rval<f32>>,
    pub linear_acceleration: Option<Rval<f32>>,
    pub direction: Option<Rval<Vec2>>,
    pub angular_speed: Option<Rval<f32>>,
    pub angular_acceleration: Option<Rval<f32>>,
    pub scale: Option<Rval<f32>>,
    pub color: Option<LinearRgba>,
    pub gravity_direction: Option<Rval<Vec2>>,
    pub gravity_speed: Option<Rval<f32>>,
    pub linear_damp: Option<Rval<f32>>,
    pub angular_damp: Option<Rval<f32>>,
    pub scale_curve: Option<curve::MultiCurve<f32>>,
    pub color_curve: Option<curve::MultiCurve<LinearRgba>>,
}

impl Default for Particle2dEffect {
    fn default() -> Self {
        Self {
            spawn_rate: 0.1,
            spawn_amount: 1,
            emission_shape: EmissionShape::Point,
            lifetime: Rval::new(1., 0.0),
            linear_speed: Some(Rval(100., 0.1)),
            linear_acceleration: None,
            direction: Some(Rval(Vec2::Y, 0.1)),
            angular_speed: None,
            angular_acceleration: None,
            scale: Some(Rval(5., 1.)),
            color: None,
            gravity_direction: None,
            gravity_speed: None,
            linear_damp: None,
            angular_damp: None,
            scale_curve: None,
            color_curve: None,
        }
    }
}
