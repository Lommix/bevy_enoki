use bevy::prelude::*;
use serde::{Deserialize, Serialize};

#[derive(Deserialize, Default, Serialize, Debug, Clone)]
pub struct MultiCurve<T>
where
    T: LerpThat<T> + Clone + Copy + std::fmt::Debug + Default,
{
    pub points: Vec<(T, f32, Option<EaseFunction>)>,
    start_value: T,
    end_value: T,
}

impl<T> MultiCurve<T>
where
    T: LerpThat<T> + Clone + Copy + std::fmt::Debug + Default,
{
    pub fn new() -> Self {
        Self {
            points: vec![],
            start_value: T::default(),
            end_value: T::default(),
        }
    }

    /// sorts the curve ASC
    pub fn sort(&mut self) {
        self.points.sort_by(|a, b| a.1.total_cmp(&b.1));
        self.start_value = self
            .points
            .first()
            .map_or(T::default(), |(value, _, _)| *value);
        self.end_value = self
            .points
            .last()
            .map_or(T::default(), |(value, _, _)| *value);
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

        let right_index = self.points.iter().position(|(_, pos, _)| *pos > position);
        let Some(right_index) = right_index else {
            return self.end_value;
        };

        let left_index = right_index.saturating_sub(1);

        if right_index == left_index {
            return self.start_value;
        }

        let left_pos = self.points[left_index].1;
        let right_pos = self.points[right_index].1;

        let left_value = self.points[left_index].0;
        let right_value = self.points[right_index].0;

        let progress = (position - left_pos) / (right_pos - left_pos);

        let eased_progress = match &self.points[right_index].2 {
            Some(easing) => EasingCurve::new(0., 1., *easing)
                .sample(progress)
                .unwrap_or_default(),
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
        let out = self
            .to_linear()
            .to_vec4()
            .lerp(right.to_linear().to_vec4(), val);
        Color::linear_rgba(out.x, out.y, out.z, out.w)
    }
}

impl LerpThat<LinearRgba> for LinearRgba {
    fn lerp_that(self, right: LinearRgba, val: f32) -> LinearRgba {
        let out = self.to_vec4().lerp(right.to_vec4(), val);
        LinearRgba::from_vec4(out)
    }
}
