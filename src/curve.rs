use bevy::prelude::*;
use interpolation::Ease;
use serde::Deserialize;

#[derive(Deserialize, Debug, Clone)]
pub enum EaseFunction {
    QuadraticIn,
    QuadraticOut,
    QuadraticInOut,

    CubicIn,
    CubicOut,
    CubicInOut,

    QuarticIn,
    QuarticOut,
    QuarticInOut,

    QuinticIn,
    QuinticOut,
    QuinticInOut,

    SineIn,
    SineOut,
    SineInOut,

    CircularIn,
    CircularOut,
    CircularInOut,

    ExponentialIn,
    ExponentialOut,
    ExponentialInOut,

    ElasticIn,
    ElasticOut,
    ElasticInOut,

    BackIn,
    BackOut,
    BackInOut,

    BounceIn,
    BounceOut,
    BounceInOut,
}

impl From<&EaseFunction> for interpolation::EaseFunction {
    fn from(value: &EaseFunction) -> Self {
        match value {
            EaseFunction::QuadraticIn => interpolation::EaseFunction::QuadraticIn,
            EaseFunction::QuadraticOut => interpolation::EaseFunction::QuadraticOut,
            EaseFunction::QuadraticInOut => interpolation::EaseFunction::QuadraticInOut,
            EaseFunction::CubicIn => interpolation::EaseFunction::CubicIn,
            EaseFunction::CubicOut => interpolation::EaseFunction::CubicOut,
            EaseFunction::CubicInOut => interpolation::EaseFunction::CubicInOut,
            EaseFunction::QuarticIn => interpolation::EaseFunction::QuarticIn,
            EaseFunction::QuarticOut => interpolation::EaseFunction::QuarticOut,
            EaseFunction::QuarticInOut => interpolation::EaseFunction::QuarticInOut,
            EaseFunction::QuinticIn => interpolation::EaseFunction::QuinticIn,
            EaseFunction::QuinticOut => interpolation::EaseFunction::QuinticOut,
            EaseFunction::QuinticInOut => interpolation::EaseFunction::QuinticInOut,
            EaseFunction::SineIn => interpolation::EaseFunction::SineIn,
            EaseFunction::SineOut => interpolation::EaseFunction::SineOut,
            EaseFunction::SineInOut => interpolation::EaseFunction::SineInOut,
            EaseFunction::CircularIn => interpolation::EaseFunction::CircularIn,
            EaseFunction::CircularOut => interpolation::EaseFunction::CircularOut,
            EaseFunction::CircularInOut => interpolation::EaseFunction::CircularInOut,
            EaseFunction::ExponentialIn => interpolation::EaseFunction::ExponentialIn,
            EaseFunction::ExponentialOut => interpolation::EaseFunction::ExponentialOut,
            EaseFunction::ExponentialInOut => interpolation::EaseFunction::ExponentialInOut,
            EaseFunction::ElasticIn => interpolation::EaseFunction::ElasticIn,
            EaseFunction::ElasticOut => interpolation::EaseFunction::ElasticOut,
            EaseFunction::ElasticInOut => interpolation::EaseFunction::ElasticInOut,
            EaseFunction::BackIn => interpolation::EaseFunction::BackIn,
            EaseFunction::BackOut => interpolation::EaseFunction::BackOut,
            EaseFunction::BackInOut => interpolation::EaseFunction::BackInOut,
            EaseFunction::BounceIn => interpolation::EaseFunction::BounceIn,
            EaseFunction::BounceOut => interpolation::EaseFunction::BounceOut,
            EaseFunction::BounceInOut => interpolation::EaseFunction::BounceInOut,
        }
    }
}

#[derive(Deserialize, Debug, Clone)]
pub struct Curve<T>
where
    T: LerpThat<T> + std::fmt::Debug,
{
    points: Vec<(T, f32, Option<EaseFunction>)>,
}

impl<T> Curve<T>
where
    T: LerpThat<T> + Clone + Copy + std::fmt::Debug,
{
    /// sorts the curve ASC
    pub fn sort(&mut self) {
        self.points.sort_by(|a, b| a.1.total_cmp(&b.1));
    }

    /// adds a point
    pub fn with_point(mut self, value: T, position: f32, easing: Option<EaseFunction>) -> Self {
        self.points.push((value, position, easing));
        self.sort();
        self
    }

    /// reads the value from a given position (0 - 1.)
    pub fn lerp(&self, position: f32) -> T {
        let position = position.max(0.);

        let right_index = self
            .points
            .iter()
            .position(|(_, pos, _)| *pos > position)
            .unwrap_or_default();

        let left_index = right_index.checked_sub(1).unwrap_or_default();

        if right_index == left_index {
            return self.points[0].0;
        }

        let left_pos = self.points[left_index].1;
        let right_pos = self.points[right_index].1;

        let left_value = self.points[left_index].0;
        let right_value = self.points[right_index].0;

        let progress = (position - left_pos) / (right_pos - left_pos);

        let eased_progress = match &self.points[right_index].2 {
            Some(easing) => progress.calc(interpolation::EaseFunction::from(easing)),
            None => progress,
        };

        left_value.lerp_that(right_value, eased_progress)
    }
}

pub trait LerpThat<T> {
    fn lerp_that(self, right: T, val: f32) -> T;
}

impl LerpThat<f32> for f32 {
    fn lerp_that(self, right: f32, val: f32) -> f32 {
        self.lerp(right, val)
    }
}

impl LerpThat<Color> for Color {
    fn lerp_that(self, right: Color, val: f32) -> Color {
        let out = self.rgba_to_vec4().lerp(right.rgba_to_vec4(), val);
        Color::rgba(out.x, out.y, out.z, out.w)
    }
}
