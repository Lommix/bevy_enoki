use bevy::{
    log::{
        tracing::{self, Subscriber},
        BoxedLayer,
    },
    prelude::*,
};
use std::sync::mpsc::{self, Sender};
use tracing_subscriber::Layer;

pub struct LogPlugin;
impl Plugin for LogPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<LogBuffer>();
        app.add_systems(Update, update_buffer);
    }
}

#[derive(Resource, Default, Deref, DerefMut)]
pub struct LogBuffer(Vec<LogEvent>);

fn update_buffer(mut buffer: ResMut<LogBuffer>, logs_rx: Option<NonSend<LogEventsReceiver>>) {
    if let Some(receiver) = logs_rx {
        for e in receiver.try_iter() {
            buffer.push(e);
            if buffer.len() > 5 {
                buffer.remove(0);
            }
        }
    }
}
pub(crate) fn clear_logs(mut logs: ResMut<LogBuffer>) {
    logs.clear();
}

#[derive(Deref, DerefMut)]
pub struct LogEventsReceiver(mpsc::Receiver<LogEvent>);

pub fn log_capture_layer(app: &mut App) -> Option<BoxedLayer> {
    let (sender, receiver) = mpsc::channel();

    let layer = CaptureLayer { sender };
    let log_receiver = LogEventsReceiver(receiver);

    app.insert_non_send_resource(log_receiver);
    Some(layer.boxed())
}

#[derive(Debug, Event, Clone)]
pub struct LogEvent {
    pub message: String,
    pub metadata: &'static tracing::Metadata<'static>,
}

pub struct CaptureLayer {
    sender: Sender<LogEvent>,
}
impl<S: Subscriber> Layer<S> for CaptureLayer {
    fn on_event(
        &self,
        event: &tracing::Event<'_>,
        _ctx: tracing_subscriber::layer::Context<'_, S>,
    ) {
        let mut message = None;
        event.record(&mut CaptureLayerVisitor(&mut message));
        if let Some(message) = message {
            self.sender
                .send(LogEvent {
                    message,
                    metadata: event.metadata(),
                })
                .ok();
        }
    }
}

/// A [`Visit`](tracing::field::Visit)or that records log messages that are transferred to [`CaptureLayer`].
struct CaptureLayerVisitor<'a>(&'a mut Option<String>);
impl tracing::field::Visit for CaptureLayerVisitor<'_> {
    fn record_debug(&mut self, _field: &tracing::field::Field, value: &dyn std::fmt::Debug) {
        *self.0 = Some(format!("{value:?}"));
    }
}
