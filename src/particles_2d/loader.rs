use super::ParticleEffectInstance;
use crate::{curve::Curve, values::Rval};
use bevy::{
    asset::{AssetLoadError, AssetLoader, AsyncReadExt},
    prelude::*,
};
use serde::{ Deserialize, Serialize };

#[derive(Default)]
pub struct ParticleEffectLoader;
impl AssetLoader for ParticleEffectLoader {
    type Asset = Particle2dEffect;
    type Settings = ();
    type Error = AssetLoadError;

    fn load<'a>(
        &'a self,
        reader: &'a mut bevy::asset::io::Reader,
        _settings: &'a Self::Settings,
        _load_context: &'a mut bevy::asset::LoadContext,
    ) -> bevy::utils::BoxedFuture<'a, Result<Self::Asset, Self::Error>> {
        Box::pin(async move {
            let mut bytes = Vec::new();
            reader.read_to_end(&mut bytes).await.unwrap();
            let mut asset = ron::de::from_bytes::<Self::Asset>(bytes.as_slice())
                .map_err(|_| AssetLoadError::AssetMetaReadError)?;

            if let Some(curve) = asset.scale_curve.as_mut() {
                curve.sort();
            }

            if let Some(curve) = asset.color_curve.as_mut() {
                curve.sort();
            }

            Ok(asset)
        })
    }

    fn extensions(&self) -> &[&str] {
        &["particle.ron"]
    }
}

#[derive(Component)]
pub struct ReloadEffectTag;

pub(crate) fn on_asset_loaded(
    mut cmd: Commands,
    mut events: EventReader<AssetEvent<Particle2dEffect>>,
    mut query: Query<(Entity, &Handle<Particle2dEffect>)>,
) {
    events.read().for_each(|event| {
        let assset_id = match event {
            AssetEvent::LoadedWithDependencies { id } => id,
            _ => {
                return;
            }
        };
        query
            .iter_mut()
            .filter(|(_, handle)| handle.id() == *assset_id)
            .for_each(|(entity, _)| {
                if let Some(mut cmd) = cmd.get_entity(entity) {
                    cmd.insert(ReloadEffectTag);
                }
            });
    })
}

pub(crate) fn reload_effect(
    mut cmd: Commands,
    mut effect_owner: Query<
        (Entity, &mut ParticleEffectInstance, &Handle<Particle2dEffect>),
        With<ReloadEffectTag>,
    >,
    effects: Res<Assets<Particle2dEffect>>,
) {
    effect_owner
        .iter_mut()
        .for_each(|(entity, mut owner, handle)| {
            let Some(effect) = effects.get(handle) else {
                return;
            };
            owner.0 = Some(Box::new(effect.clone()));
            cmd.entity(entity).remove::<ReloadEffectTag>();
        });
}

#[derive(Deserialize, Default, Clone, Debug, Serialize)]
pub enum EmissionShape {
    #[default]
    Point,
    Circle(f32),
}

#[derive(Asset, TypePath, Default, Deserialize, Serialize, Clone, Debug)]
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
    pub color: Option<Color>,
    pub gravity_direction: Option<Rval<Vec2>>,
    pub gravity_speed: Option<Rval<f32>>,
    pub linear_damp: Option<Rval<f32>>,
    pub angular_damp: Option<Rval<f32>>,
    pub scale_curve: Option<Curve<f32>>,
    pub color_curve: Option<Curve<Color>>,
}
