use bevy::{
    asset::load_internal_asset,
    prelude::*,
    render::{batching::NoAutomaticBatching, render_resource::AsBindGroup},
    sprite::Mesh2dHandle,
};

use crate::particles_2d::sprite::SpriteParticleMaterial;

use self::prelude::*;

mod loader;
mod render;
mod sprite;
mod update;

#[allow(unused)]
pub mod prelude {
    pub use super::loader::{EmissionShape, Particle2dEffect, ParticleEffectLoader};
    pub use super::render::{Particle2dMaterial, Particle2dMaterialPlugin};
    pub use super::sprite::SpriteParticleMaterial;
    pub use super::update::{
        OneShot, ParticleController, ParticleEffectOwner, ParticleState, ParticleStore,
    };
    pub use super::{ColorParticleMaterial, ParticleSpawnerBundle};
}

pub(crate) const PARTICLE_MESH: Handle<Mesh> =
    Handle::weak_from_u128(31145795777456212157789641244521641567);
pub(crate) const PARTICLE_VERTEX_OUT: Handle<Shader> =
    Handle::weak_from_u128(97641680653231235698756524356666664231);
pub(crate) const PARTICLE_VERTEX: Handle<Shader> =
    Handle::weak_from_u128(47641680653221245698756524356666564219);
pub(crate) const PARTICLE_DEFAULT_FRAG: Handle<Shader> =
    Handle::weak_from_u128(27641685611347896254665674658656433339);
pub(crate) const PARTICLE_SPRITE_FRAG: Handle<Shader> =
    Handle::weak_from_u128(12323345476666666666641232313030656999);
pub const DEFAULT_MATERIAL: Handle<ColorParticleMaterial> =
    Handle::weak_from_u128(12323345476653413666641232313030656999);

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

        app.register_type::<update::ParticleStore>();
        app.register_type::<update::ParticleState>();
        app.register_type::<update::ParticleController>();
        app.register_type::<update::Particle>();

        app.add_plugins(render::Particle2dMaterialPlugin::<ColorParticleMaterial>::default());
        app.add_plugins(render::Particle2dMaterialPlugin::<SpriteParticleMaterial>::default());

        app.world
            .resource_mut::<Assets<ColorParticleMaterial>>()
            .insert(DEFAULT_MATERIAL, ColorParticleMaterial::default());

        app.world
            .get_resource_mut::<Assets<Mesh>>()
            .unwrap()
            .insert(PARTICLE_MESH, Rectangle::new(1., 1.).into());

        app.init_asset::<Particle2dEffect>();
        app.init_asset_loader::<loader::ParticleEffectLoader>();

        app.add_systems(
            Update,
            (
                loader::on_asset_loaded.run_if(on_event::<AssetEvent<Particle2dEffect>>()),
                // loader::reload_effect,
                update::update_spawner,
                // update::clone_effect,
                update::remove_finished_spawner,
            ),
        );
    }
}

#[derive(Asset, TypePath, AsBindGroup, Clone, Default)]
pub struct ColorParticleMaterial {
    pub color: Vec4,
}

impl Particle2dMaterial for ColorParticleMaterial {}

#[derive(Bundle)]
pub struct ParticleSpawnerBundle<M: Particle2dMaterial> {
    pub controller: ParticleController,
    pub state: ParticleState,
    pub effect: Handle<Particle2dEffect>,
    pub overwrite: ParticleEffectOwner,
    pub particle_store: ParticleStore,
    pub material: Handle<M>,
    pub mesh: Mesh2dHandle,
    pub visibility: Visibility,
    pub inherited_visibility: InheritedVisibility,
    pub view_visibility: ViewVisibility,
    pub transform: Transform,
    pub global_transform: GlobalTransform,
    pub no_batch: NoAutomaticBatching,
}

// pub type SpriteParticleBundle = ParticleSpawnerBundle<SpriteParticleMaterial>;
// pub type DefaultParticleBundle = ParticleSpawnerBundle<ColorParticleMaterial>;

impl<M: Particle2dMaterial + Default> Default for ParticleSpawnerBundle<M> {
    fn default() -> Self {
        Self {
            state: ParticleState::default(),
            controller: ParticleController::default(),
            effect: Handle::default(),
            overwrite: ParticleEffectOwner::default(),
            particle_store: ParticleStore::default(),
            mesh: Mesh2dHandle(PARTICLE_MESH),
            visibility: Visibility::default(),
            inherited_visibility: InheritedVisibility::default(),
            transform: Transform::default(),
            global_transform: GlobalTransform::default(),
            view_visibility: ViewVisibility::default(),
            no_batch: NoAutomaticBatching,
            material: Default::default(),
        }
    }
}
