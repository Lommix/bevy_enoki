use bevy::{prelude::*, tasks::AsyncComputeTaskPool, time::common_conditions::on_timer};
use bevy_egui::egui::{self, Ui};
use bevy_enoki::prelude::*;
use crossbeam::channel::{bounded, Receiver, Sender};
use rfd::AsyncFileDialog;
use std::time::Duration;

pub struct FileManagerPlugin;
impl Plugin for FileManagerPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<FileResource>();
        app.add_systems(
            Update,
            file_watcher.run_if(on_timer(Duration::from_millis(50))),
        );
    }
}

#[derive(Resource)]
pub(crate) struct FileResource {
    pub name: String,
    send: Sender<EffectFile>,
    rec: Receiver<EffectFile>,
}

pub(crate) struct EffectFile {
    file_name: String,
    data: Vec<u8>,
}

impl TryInto<Particle2dEffect> for EffectFile {
    type Error = anyhow::Error;

    fn try_into(self) -> Result<Particle2dEffect, Self::Error> {
        ron::de::from_bytes(self.data.as_slice()).map_err(|e| e.into())
    }
}

impl Default for FileResource {
    fn default() -> Self {
        let (tx, rx) = bounded(1);
        Self {
            name: "new_effect.ron".to_string(),
            send: tx,
            rec: rx,
        }
    }
}

fn file_watcher(
    mut file_res: ResMut<FileResource>,
    mut instances: Query<&mut ParticleEffectInstance>,
) {
    let Ok(file) = file_res.rec.try_recv() else {
        return;
    };

    file_res.name = file.file_name.clone();
    let Ok(effect): Result<Particle2dEffect, _> = file.try_into() else {
        return;
    };

    instances.iter_mut().for_each(|mut instance| {
        instance.0 = Some(Box::new(effect.clone()));
    });
}

pub(crate) fn file_dialog_load(sender: Sender<EffectFile>) {
    AsyncComputeTaskPool::get()
        .spawn(async move {
            match AsyncFileDialog::new()
                .set_title("pick ron config")
                .add_filter("ron", &["ron"])
                .pick_file()
                .await
            {
                Some(handle) => {
                    let content = handle.read().await;
                    sender
                        .send(EffectFile {
                            file_name: handle.file_name(),
                            data: content,
                        })
                        .expect("could not send");
                }
                None => (),
            }
        })
        .detach();
}

pub(crate) fn file_dialog_save(file: EffectFile) {
    AsyncComputeTaskPool::get()
        .spawn(async move {
            match AsyncFileDialog::new()
                .set_file_name(file.file_name)
                .set_title("download ron config")
                .save_file()
                .await
            {
                Some(handle) => {
                    handle.write(file.data.as_slice()).await.unwrap();
                }
                None => (),
            }
        })
        .detach();
}

pub(crate) fn file_panel(world: &mut World, ui: &mut Ui) {
    egui::Grid::new("file_select")
        .num_columns(2)
        .spacing([4., 4.])
        .show(ui, |ui| {
            if ui
                .add_sized([100., 30.], egui::Button::new("Import"))
                .clicked()
            {
                let sender = world.resource::<FileResource>().send.clone();
                file_dialog_load(sender);
            }

            if ui
                .add_sized([100., 30.], egui::Button::new("Export"))
                .clicked()
            {
                let current_file_name = world.resource::<FileResource>().name.clone();
                let Some(file) = world
                    .query::<&ParticleEffectInstance>()
                    .get_single(world)
                    .ok()
                    .map(|instance| instance.0.as_ref())
                    .flatten()
                    .map(|effect| match ron::ser::to_string(&effect) {
                        Ok(content) => Some(content),
                        Err(err) => {
                            error!("could not serialize effect: {}", err);
                            None
                        }
                    })
                    .flatten()
                    .map(|content| EffectFile {
                        file_name: current_file_name,
                        data: content.as_bytes().to_vec(),
                    })
                else {
                    return;
                };

                file_dialog_save(file);
            }
        });
}
