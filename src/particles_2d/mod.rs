use self::prelude::{
    Particle2dEffect, Particle2dMaterial, ParticleController, ParticleEffectOwner, ParticleState,
    ParticleStore,
};
use crate::particles_2d::sprite::ColorParticle2dMaterial;
use bevy::{asset::load_internal_asset, prelude::*};

mod loader;
mod render;
mod sprite;
mod update;

#[allow(unused)]
pub mod prelude {
    pub use super::loader::{EmissionShape, Particle2dEffect, ParticleEffectLoader};
    pub use super::render::{Particle2dMaterial, Particle2dMaterialPlugin};
    pub use super::sprite::ColorParticle2dMaterial;
    pub use super::update::{
        OneShot, ParticleController, ParticleEffectOwner, ParticleState, ParticleStore,
    };
    pub use super::{ParticleSpawnerBundle, DEFAULT_MATERIAL};
}

pub(crate) const PARTICLE_VERTEX_OUT: Handle<Shader> =
    Handle::weak_from_u128(97641680653231235698756524351080664231);
pub(crate) const PARTICLE_VERTEX: Handle<Shader> =
    Handle::weak_from_u128(47641680653221245698756524350619564219);
pub(crate) const PARTICLE_DEFAULT_FRAG: Handle<Shader> =
    Handle::weak_from_u128(27641685611347896254665674658656433339);
pub(crate) const PARTICLE_SPRITE_FRAG: Handle<Shader> =
    Handle::weak_from_u128(52323345476969163624641232313030656999);
pub const DEFAULT_MATERIAL: Handle<ColorParticle2dMaterial> =
    Handle::weak_from_u128(96423345474653413361641232813030656919);

pub struct Particles2dPlugin;
impl Plugin for Particles2dPlugin {
    fn build(&self, app: &mut App) {
        load_internal_asset!(
            app,
            PARTICLE_VERTEX_OUT,
            "particle_vertex_out.wgsl",
            Shader::from_wgsl
        );

        load_internal_asset!(
            app,
            PARTICLE_VERTEX,
            "particle_vertex.wgsl",
            Shader::from_wgsl
        );

        load_internal_asset!(
            app,
            PARTICLE_DEFAULT_FRAG,
            "particle_default_frag.wgsl",
            Shader::from_wgsl
        );

        load_internal_asset!(
            app,
            PARTICLE_SPRITE_FRAG,
            "particle_sprite_frag.wgsl",
            Shader::from_wgsl
        );

        app.add_plugins(render::Particle2dMaterialPlugin::<ColorParticle2dMaterial>::default());

        app.register_type::<update::ParticleStore>();
        app.register_type::<update::ParticleState>();
        app.register_type::<update::ParticleController>();
        app.register_type::<update::Particle>();

        app.world
            .resource_mut::<Assets<ColorParticle2dMaterial>>()
            .insert(DEFAULT_MATERIAL, ColorParticle2dMaterial::default());

        app.init_asset::<Particle2dEffect>();
        app.init_asset_loader::<loader::ParticleEffectLoader>();

        app.add_systems(
            First,
            loader::on_asset_loaded.run_if(on_event::<AssetEvent<Particle2dEffect>>()),
        );

        app.add_systems(
            Update,
            (
                loader::reload_effect,
                // update::update_spawner,
                update::clone_effect,
                update::remove_finished_spawner,
            ),
        );

        app.add_systems(
            PostUpdate,
            update::update_spawner.after(bevy::render::view::VisibilitySystems::CheckVisibility),
        );
    }
}

#[derive(Bundle)]
pub struct ParticleSpawnerBundle<M: Particle2dMaterial> {
    pub controller: ParticleController,
    pub state: ParticleState,
    pub effect: Handle<Particle2dEffect>,
    pub overwrite: ParticleEffectOwner,
    pub particle_store: ParticleStore,
    pub material: Handle<M>,
    pub visibility: Visibility,
    pub inherited_visibility: InheritedVisibility,
    pub view_visibility: ViewVisibility,
    pub transform: Transform,
    pub global_transform: GlobalTransform,
}

impl<M: Particle2dMaterial + Default> Default for ParticleSpawnerBundle<M> {
    fn default() -> Self {
        Self {
            state: ParticleState::default(),
            controller: ParticleController::default(),
            effect: Handle::default(),
            overwrite: ParticleEffectOwner::default(),
            particle_store: ParticleStore::default(),
            visibility: Visibility::default(),
            inherited_visibility: InheritedVisibility::default(),
            transform: Transform::default(),
            global_transform: GlobalTransform::default(),
            view_visibility: ViewVisibility::default(),
            material: Default::default(),
        }
    }
}
