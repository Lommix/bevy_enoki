use self::prelude::{
    Particle2dEffect, Particle2dMaterial, ParticleEffectInstance, ParticleSpawnerState,
    ParticleStore,
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
use loader::EffectHandle;

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
    pub use super::curve::{Curve, EaseFunction, LerpThat};
    pub use super::loader::{EffectHandle, EmissionShape, Particle2dEffect, ParticleEffectLoader};
    pub use super::material::{Particle2dMaterial, Particle2dMaterialPlugin};
    pub use super::sprite::SpriteParticle2dMaterial;
    pub use super::update::{OneShot, ParticleEffectInstance, ParticleSpawnerState, ParticleStore};
    pub use super::values::{Random, Rval};
    pub use super::ParticleSpawner;
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
        app.register_type::<loader::EffectHandle>();
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
                calculcate_particle_bounds.in_set(VisibilitySystems::CalculateBounds),
                check_visibility::<WithParticles>.in_set(VisibilitySystems::CheckVisibility),
            ),
        );
    }
}

pub type WithParticles = With<ParticleSpawnerState>;
fn calculcate_particle_bounds(mut cmd: Commands, spawners: Query<(Entity, &ParticleStore)>) {
    spawners.iter().for_each(|(entity, store)| {
        let particle_count = store.len();

        if particle_count <= 0 {
            return;
        }

        let accuracy = (particle_count / 1000).min(1).max(10);

        let (min, max) = store
            .iter()
            .enumerate()
            .filter(|(i, _)| i % accuracy == 0)
            .fold((Vec2::ZERO, Vec2::ZERO), |mut acc, (_, particle)| {
                acc.0.x = acc.0.x.min(particle.transform.translation.x);
                acc.0.y = acc.0.y.min(particle.transform.translation.y);
                acc.1.x = acc.1.x.max(particle.transform.translation.x);
                acc.1.y = acc.1.y.max(particle.transform.translation.y);
                acc
            });
        cmd.entity(entity)
            .try_insert(Aabb::from_min_max(min.extend(0.), max.extend(0.)));
    });
}

/// The main particle spawner components
/// has required components
#[derive(Component, DerefMut, Deref, Clone)]
#[require(
    ParticleSpawnerState,
    ParticleEffectInstance,
    EffectHandle,
    ParticleStore,
    Transform,
    Visibility,
    Aabb,
    SyncToRenderWorld
)]
pub struct ParticleSpawner<T: Asset>(pub Handle<T>);

impl<T: Asset> From<Handle<T>> for ParticleSpawner<T> {
    fn from(value: Handle<T>) -> Self {
        Self(value)
    }
}

impl Default for ParticleSpawner<ColorParticle2dMaterial> {
    fn default() -> Self {
        ParticleSpawner(Handle::default())
    }
}
