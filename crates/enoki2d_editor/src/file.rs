use bevy::{prelude::*, tasks::AsyncComputeTaskPool, time::common_conditions::on_timer};
use bevy_enoki::prelude::*;
use crossbeam::channel::{bounded, Receiver, Sender};
use rfd::AsyncFileDialog;
use std::time::Duration;

pub struct FileManagerPlugin;
impl Plugin for FileManagerPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<EffectChannel>();
        app.add_systems(
            Update,
            file_watcher.run_if(on_timer(Duration::from_millis(50))),
        );
    }
}

#[derive(Resource)]
pub(crate) struct EffectChannel {
    pub last_file_name: String,
    pub send: Sender<EffectFileWrapper>,
    rec: Receiver<EffectFileWrapper>,
}

pub struct EffectFileWrapper {
    file_name: String,
    effect: Particle2dEffect,
}

impl Default for EffectChannel {
    fn default() -> Self {
        let (tx, rx) = bounded(1);
        Self {
            last_file_name: "my_new_effect.ron".to_string(),
            send: tx,
            rec: rx,
        }
    }
}

fn file_watcher(
    mut effect_channel: ResMut<EffectChannel>,
    mut instances: Query<&mut ParticleEffectInstance>,
) {
    let Ok(effect_wrapper) = effect_channel.rec.try_recv() else {
        return;
    };

    effect_channel.last_file_name = effect_wrapper.file_name;
    instances.iter_mut().for_each(|mut instance| {
        instance.0 = Some(effect_wrapper.effect.clone());
    });
}

pub fn open_load_dialog(sender: Sender<EffectFileWrapper>) {
    AsyncComputeTaskPool::get()
        .spawn(async move {
            match AsyncFileDialog::new()
                .set_title("load effect assset")
                .pick_file()
                .await
            {
                Some(handle) => {
                    let bytes = handle.read().await;
                    let effect: Particle2dEffect = match ron::de::from_bytes(&bytes) {
                        Ok(effect) => effect,
                        Err(err) => {
                            let path = handle.path().to_str().unwrap_or("");
                            error!(
                                "`{}` is not a valid particle effect asset!\n\n {:?}",
                                path, err
                            );
                            return;
                        }
                    };

                    let packed_effect = EffectFileWrapper {
                        effect,
                        file_name: handle.file_name(),
                    };

                    match sender.send(packed_effect) {
                        Ok(_) => (),
                        Err(err) => {
                            error!("Channel failed!\n\n {:?}", err);
                        }
                    };
                }
                None => (),
            }
        })
        .detach();
}

pub fn open_save_dialog(effect: Particle2dEffect, file_name: String) {
    AsyncComputeTaskPool::get()
        .spawn(async move {
            match AsyncFileDialog::new()
                .set_title("load effect assset")
                .set_file_name(file_name)
                .save_file()
                .await
            {
                Some(handle) => {
                    let string = match ron::ser::to_string(&effect) {
                        Ok(b) => b,
                        Err(err) => {
                            error!(
                                "Ops, cannot convert to string, this should not happen!\n\n {:?}",
                                err
                            );
                            return;
                        }
                    };

                    match handle.write(string.as_bytes()).await {
                        Ok(_) => (),
                        Err(err) => {
                            error!("Could not be saved!\n\n {:?}", err);
                        }
                    };
                }
                None => (),
            }
        })
        .detach();
}
