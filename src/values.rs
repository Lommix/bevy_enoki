use bevy::math::Vec2;
use serde::Deserialize;

#[derive(Deserialize, Clone, Debug, Default)]
pub struct Rval<V>(V, f32);

pub trait Random<V> {
    fn rand(&self) -> V;
}

impl Random<Vec2> for Rval<Vec2> {
    fn rand(&self) -> Vec2 {
        let max_angle = 2. * std::f32::consts::PI * self.1;
        let random_angle = (rand::random::<f32>() - 0.5) * max_angle;

        let angle = self.0.to_angle() + random_angle;
        angle_to_direction(angle)
    }
}

impl Random<f32> for Rval<f32> {
    fn rand(&self) -> f32 {
        let r = (rand::random::<f32>() - 0.5) * 2. * self.1;
        self.0 + self.0 * r
    }
}

fn angle_to_direction(angle_radians: f32) -> Vec2 {
    Vec2::new(angle_radians.cos(), angle_radians.sin())
}
