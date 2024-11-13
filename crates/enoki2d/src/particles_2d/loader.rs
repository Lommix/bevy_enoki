use super::ParticleEffectInstance;
use crate::{curve::Curve, values::Rval};
use bevy::{
    asset::{io::Reader, AssetLoadError, AssetLoader, LoadContext},
    prelude::*,
};
use serde::{Deserialize, Serialize};

#[derive(Deserialize, Reflect, Default, Clone, Debug, Serialize)]
#[reflect]
pub enum EmissionShape {
    #[default]
    Point,
    Circle(f32),
}

#[derive(Component, Reflect, Deref, DerefMut, Default)]
#[reflect]
pub struct EffectHandle(pub Handle<Particle2dEffect>);

impl From<Handle<Particle2dEffect>> for EffectHandle {
    fn from(value: Handle<Particle2dEffect>) -> Self {
        Self(value)
    }
}

/// The particle effect asset
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
    pub scale_curve: Option<Curve<f32>>,
    pub color_curve: Option<Curve<LinearRgba>>,
}

impl Default for Particle2dEffect {
    fn default() -> Self {
        Self {
            spawn_rate: 1.,
            spawn_amount: 100,
            emission_shape: EmissionShape::Point,
            lifetime: Rval::new(1., 0.5),
            linear_speed: Some(Rval(100., 0.5)),
            linear_acceleration: None,
            direction: Some(Rval(Vec2::Y, 0.2)),
            angular_speed: None,
            angular_acceleration: None,
            scale: Some(Rval(5., 0.5)),
            color: None,
            gravity_direction: Some(Rval(Vec2::NEG_Y, 0.)),
            gravity_speed: Some(Rval(100., 0.)),
            linear_damp: None,
            angular_damp: None,
            scale_curve: None,
            color_curve: None,
        }
    }
}

#[derive(Default)]
pub struct ParticleEffectLoader;
impl AssetLoader for ParticleEffectLoader {
    type Asset = Particle2dEffect;
    type Settings = ();
    type Error = AssetLoadError;

    async fn load(
        &self,
        reader: &mut dyn Reader,
        _settings: &Self::Settings,
        _load_context: &mut LoadContext<'_>,
    ) -> Result<Self::Asset, Self::Error> {
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
    }

    fn extensions(&self) -> &[&str] {
        &["ron"]
    }
}

#[derive(Component)]
pub struct ReloadEffectTag;

pub(crate) fn on_asset_loaded(
    mut cmd: Commands,
    mut events: EventReader<AssetEvent<Particle2dEffect>>,
    spawners: Query<(Entity, &EffectHandle)>,
) {
    events.read().for_each(|event| {
        let assset_id = match event {
            AssetEvent::LoadedWithDependencies { id } => id,
            _ => {
                return;
            }
        };

        spawners
            .iter()
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
        (Entity, &mut ParticleEffectInstance, &EffectHandle),
        With<ReloadEffectTag>,
    >,
    effects: Res<Assets<Particle2dEffect>>,
) {
    effect_owner
        .iter_mut()
        .for_each(|(entity, mut owner, handle)| {
            let Some(effect) = effects.get(&handle.0) else {
                return;
            };
            owner.0 = Some(Box::new(effect.clone()));
            cmd.entity(entity).remove::<ReloadEffectTag>();
        });
}
