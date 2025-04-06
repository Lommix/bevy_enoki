use super::ParticleEffectInstance;
use crate::{ParticleEffectHandle, Particle2dEffect};
use bevy::{
    asset::{io::Reader, AssetLoadError, AssetLoader, LoadContext},
    prelude::*,
};

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
    spawners: Query<(Entity, &ParticleEffectHandle)>,
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
                if let Ok(mut cmd) = cmd.get_entity(entity) {
                    cmd.insert(ReloadEffectTag);
                }
            });
    })
}

pub(crate) fn reload_effect(
    mut cmd: Commands,
    mut effect_owner: Query<
        (Entity, &mut ParticleEffectInstance, &ParticleEffectHandle),
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
            owner.0 = Some(effect.clone());
            cmd.entity(entity).remove::<ReloadEffectTag>();
        });
}
