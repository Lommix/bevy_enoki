use bevy::prelude::*;

pub(crate) mod curve;
mod particles_2d;
pub(crate) mod values;

#[allow(unused)]
pub mod prelude {
    pub use crate::particles_2d::prelude::*;
    pub use crate::EnokiPlugin;
}

pub struct EnokiPlugin;
impl Plugin for EnokiPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins(particles_2d::Particles2dPlugin);
    }
}
