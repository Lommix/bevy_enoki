use super::{prelude::EmissionShape, Particle2dEffect, ParticleEffectHandle};
use crate::values::Random;
use bevy_asset::Assets;
use bevy_camera::primitives::Aabb;
use bevy_color::LinearRgba;
use bevy_derive::{Deref, DerefMut};
use bevy_ecs::{
    component::Component,
    entity::Entity,
    query::{Added, Without},
    reflect::ReflectComponent,
    system::{Commands, Query, Res},
};
use bevy_math::{EulerRot, Vec2, Vec3};
use bevy_reflect::{prelude::ReflectDefault, Reflect};
use bevy_time::{Time, Timer, TimerMode, Virtual};
use bevy_transform::components::{GlobalTransform, Transform};
use std::time::Duration;
use wide::f32x8;

/// Tag Component, deactivates spawner after the first
/// spawning of particles
#[derive(Component, Default, Debug, Reflect)]
#[reflect(Component, Debug, Default)]
pub enum OneShot {
    #[default]
    Deactivate,
    Despawn,
}

/// Spawner states controls the spawner
#[derive(Component, Clone, Debug, Reflect)]
pub struct ParticleSpawnerState {
    pub max_particles: u32,
    pub active: bool,
    pub timer: Timer,
    pub previous_position: Option<Vec3>,
}

/// A clone of the asset, unique to each spawner
/// that can be mutated by any system.
/// Any change to `Particle2dEffect` Asset will
/// re-copy the effect.
#[derive(Component, Deref, DerefMut, Default)]
pub struct ParticleEffectInstance(pub Option<Particle2dEffect>);

impl Default for ParticleSpawnerState {
    fn default() -> Self {
        Self {
            active: true,
            max_particles: u32::MAX,
            timer: Timer::new(Duration::ZERO, TimerMode::Repeating),
            previous_position: None,
        }
    }
}

/// Component for storing particle data
#[derive(Component, Default, Clone, Reflect)]
pub struct ParticleStore {
    pub(crate) position_x: Vec<f32>,
    pub(crate) position_y: Vec<f32>,
    pub(crate) position_z: Vec<f32>,
    pub(crate) rotation: Vec<f32>,
    pub(crate) scale_x: Vec<f32>,
    pub(crate) scale_y: Vec<f32>,
    pub(crate) scale_z: Vec<f32>,
    pub(crate) duration: Vec<f32>,
    pub(crate) duration_fraction: Vec<f32>,
    pub(crate) velocity_x: Vec<f32>,
    pub(crate) velocity_y: Vec<f32>,
    pub(crate) velocity_z: Vec<f32>,
    pub(crate) angular_velocity: Vec<f32>,
    pub(crate) color_r: Vec<f32>,
    pub(crate) color_g: Vec<f32>,
    pub(crate) color_b: Vec<f32>,
    pub(crate) color_a: Vec<f32>,
    pub(crate) frame: Vec<u32>,
    pub(crate) linear_acceleration: Vec<f32>,
    pub(crate) linear_damp: Vec<f32>,
    pub(crate) angular_acceleration: Vec<f32>,
    pub(crate) angular_damp: Vec<f32>,
    pub(crate) gravity_speed: Vec<f32>,
    pub(crate) gravity_x: Vec<f32>,
    pub(crate) gravity_y: Vec<f32>,
    pub(crate) gravity_z: Vec<f32>,
}

impl ParticleStore {
    pub fn len(&self) -> usize {
        self.duration.len()
    }

    pub fn is_empty(&self) -> bool {
        self.duration.is_empty()
    }

    pub fn clear(&mut self) {
        macro_rules! clear {
            ($($field:ident),+ $(,)?) => {
                $(self.$field.clear();)+
            };
        }
        clear!(
            position_x,
            position_y,
            position_z,
            rotation,
            scale_x,
            scale_y,
            scale_z,
            duration,
            duration_fraction,
            velocity_x,
            velocity_y,
            velocity_z,
            angular_velocity,
            color_r,
            color_g,
            color_b,
            color_a,
            frame,
            linear_acceleration,
            linear_damp,
            angular_acceleration,
            angular_damp,
            gravity_speed,
            gravity_x,
            gravity_y,
            gravity_z,
        );
    }

    #[allow(clippy::too_many_arguments)]
    fn push(
        &mut self,
        transform: Transform,
        duration: f32,
        velocity: Vec3,
        angular_velocity: f32,
        color: LinearRgba,
        linear_acceleration: f32,
        linear_damp: f32,
        angular_acceleration: f32,
        angular_damp: f32,
        gravity_speed: f32,
        gravity_direction: Vec3,
    ) {
        self.position_x.push(transform.translation.x);
        self.position_y.push(transform.translation.y);
        self.position_z.push(transform.translation.z);
        self.rotation
            .push(transform.rotation.to_euler(EulerRot::XYZ).2);
        self.scale_x.push(transform.scale.x);
        self.scale_y.push(transform.scale.y);
        self.scale_z.push(transform.scale.z);
        self.duration.push(duration);
        self.duration_fraction.push(0.0);
        self.velocity_x.push(velocity.x);
        self.velocity_y.push(velocity.y);
        self.velocity_z.push(velocity.z);
        self.angular_velocity.push(angular_velocity);
        self.color_r.push(color.red);
        self.color_g.push(color.green);
        self.color_b.push(color.blue);
        self.color_a.push(color.alpha);
        self.frame.push(0);
        self.linear_acceleration.push(linear_acceleration);
        self.linear_damp.push(linear_damp);
        self.angular_acceleration.push(angular_acceleration);
        self.angular_damp.push(angular_damp);
        self.gravity_speed.push(gravity_speed);
        self.gravity_x.push(gravity_direction.x);
        self.gravity_y.push(gravity_direction.y);
        self.gravity_z.push(gravity_direction.z);
    }

    fn swap_remove(&mut self, index: usize) {
        macro_rules! remove {
            ($($field:ident),+ $(,)?) => {
                $(self.$field.swap_remove(index);)+
            };
        }
        remove!(
            position_x,
            position_y,
            position_z,
            rotation,
            scale_x,
            scale_y,
            scale_z,
            duration,
            duration_fraction,
            velocity_x,
            velocity_y,
            velocity_z,
            angular_velocity,
            color_r,
            color_g,
            color_b,
            color_a,
            frame,
            linear_acceleration,
            linear_damp,
            angular_acceleration,
            angular_damp,
            gravity_speed,
            gravity_x,
            gravity_y,
            gravity_z,
        );
    }

    fn remove_expired(&mut self) {
        for index in (0..self.len()).rev() {
            if self.duration_fraction[index] >= 1.0 {
                self.swap_remove(index);
            }
        }
    }
}

pub(crate) fn clone_effect(
    mut particle_spawners: Query<
        (&mut ParticleEffectInstance, &ParticleEffectHandle),
        Added<ParticleSpawnerState>,
    >,
    effects: Res<Assets<Particle2dEffect>>,
) {
    particle_spawners
        .iter_mut()
        .for_each(|(mut effect_overwrites, handle)| {
            let Some(effect) = effects.get(&handle.0) else {
                return;
            };

            effect_overwrites.0 = Some(effect.clone());
        });
}

pub(crate) fn remove_finished_spawner(
    mut cmd: Commands,
    spawner: Query<(Entity, &ParticleStore, &ParticleSpawnerState, &OneShot)>,
) {
    spawner
        .iter()
        .for_each(|(entity, store, controller, one_shot)| {
            if matches!(one_shot, OneShot::Despawn) && !controller.active && store.is_empty() {
                cmd.entity(entity).try_despawn();
            }
        })
}

pub(crate) fn update_spawner(
    mut particles: Query<(
        Entity,
        &mut ParticleStore,
        &mut ParticleSpawnerState,
        &ParticleEffectInstance,
        &GlobalTransform,
    )>,
    one_shots: Query<&OneShot>,
    time: Res<Time<Virtual>>,
) {
    particles.par_iter_mut().for_each(
        |(entity, mut store, mut state, effect_instance, transform)| {
            if state.max_particles <= store.len() as u32 {
                return;
            }

            let Some(effect) = &effect_instance.0 else {
                return;
            };

            let transform = transform.compute_transform();

            state
                .timer
                .set_duration(Duration::from_secs_f32(effect.spawn_rate));
            state.timer.tick(time.delta());

            if state.timer.is_finished() && state.active {
                for _ in 0..effect.spawn_amount {
                    create_particle(&mut store, effect, &transform);
                }

                if one_shots.get(entity).is_ok() {
                    state.active = false;
                }
            }
            let delta = time.delta_secs();
            let spawner_world_pos = transform.translation;

            // Handle relative positioning
            let position_delta = if effect.relative_positioning.unwrap_or(false) {
                let current_pos = spawner_world_pos;
                let delta = if let Some(prev_pos) = state.previous_position {
                    current_pos - prev_pos
                } else {
                    Vec3::ZERO
                };
                state.previous_position = Some(current_pos);
                delta
            } else {
                Vec3::ZERO
            };

            update_particles(&mut store, effect, delta, spawner_world_pos, position_delta);
            store.remove_expired();
        },
    );
}

fn update_particles(
    particles: &mut ParticleStore,
    effect: &Particle2dEffect,
    delta: f32,
    spawner_world_pos: Vec3,
    position_delta: Vec3,
) {
    update_particles_simd(particles, effect, delta, spawner_world_pos, position_delta);
}

fn create_particle(store: &mut ParticleStore, effect: &Particle2dEffect, transform: &Transform) {
    // direction
    let direction = effect
        .direction
        .as_ref()
        .map(|m| m.rand())
        .unwrap_or_default();

    // apply local rotation
    let direction = direction.rotate(transform.right().truncate());

    // speed
    let speed = effect
        .linear_speed
        .as_ref()
        .map(|s| s.rand())
        .unwrap_or_default();
    // angular
    let angular = effect
        .angular_speed
        .as_ref()
        .map(|s| s.rand())
        .unwrap_or_default();
    // angular
    let scale = effect.scale.as_ref().map(|s| s.rand()).unwrap_or_default();

    let gravity_direction = effect
        .gravity_direction
        .as_ref()
        .map(|g| g.rand())
        .unwrap_or_default()
        .extend(0.);

    let gravity_speed = effect
        .gravity_speed
        .as_ref()
        .map(|g| g.rand())
        .unwrap_or_default();

    let linear_damp = effect
        .linear_damp
        .as_ref()
        .map(|d| d.rand())
        .unwrap_or_default();

    let angular_damp = effect
        .angular_damp
        .as_ref()
        .map(|a| a.rand())
        .unwrap_or_default();

    let angular_acceleration = effect
        .angular_acceleration
        .as_ref()
        .map(|a| a.rand())
        .unwrap_or_default();

    let linear_acceleration = effect
        .linear_acceleration
        .as_ref()
        .map(|a| a.rand())
        .unwrap_or_default();

    let mut transform = *transform;
    transform.scale = Vec3::splat(scale);

    transform.translation += match effect.emission_shape {
        EmissionShape::Point => Vec3::ZERO,
        EmissionShape::Circle(radius) => {
            Vec3::new(rand::random::<f32>() - 0.5, rand::random::<f32>() - 0.5, 0.)
                .normalize_or_zero()
                * radius
                * rand::random::<f32>()
        }
    };

    store.push(
        transform,
        effect.lifetime.rand(),
        (direction * speed).extend(0.),
        angular,
        effect.color.unwrap_or(LinearRgba::WHITE),
        linear_acceleration,
        linear_damp,
        angular_acceleration,
        angular_damp,
        gravity_speed,
        gravity_direction,
    );
}

fn load8(values: &[f32], index: usize) -> f32x8 {
    f32x8::new(values[index..index + 8].try_into().unwrap())
}

fn store8(values: &mut [f32], index: usize, value: f32x8) {
    values[index..index + 8].copy_from_slice(&value.to_array());
}

fn update_particles_simd(
    particles: &mut ParticleStore,
    effect: &Particle2dEffect,
    delta: f32,
    spawner_world_pos: Vec3,
    position_delta: Vec3,
) {
    let delta8 = f32x8::splat(delta);
    let zero = f32x8::ZERO;
    let one = f32x8::ONE;
    let simd_len = particles.len() / 8 * 8;

    for index in (0..simd_len).step_by(8) {
        let mut px = load8(&particles.position_x, index) + f32x8::splat(position_delta.x);
        let mut py = load8(&particles.position_y, index) + f32x8::splat(position_delta.y);
        let mut pz = load8(&particles.position_z, index) + f32x8::splat(position_delta.z);
        let progress =
            load8(&particles.duration_fraction, index) + delta8 / load8(&particles.duration, index);

        let linear_factor = one
            + progress
                * (load8(&particles.linear_acceleration, index)
                    - load8(&particles.linear_damp, index))
                * delta8;
        let mut vx = load8(&particles.velocity_x, index) * linear_factor;
        let mut vy = load8(&particles.velocity_y, index) * linear_factor;
        let mut vz = load8(&particles.velocity_z, index) * linear_factor;

        let angular_factor = one
            + progress
                * (load8(&particles.angular_acceleration, index)
                    - load8(&particles.angular_damp, index))
                * delta8;
        let angular_velocity = load8(&particles.angular_velocity, index) * angular_factor;

        if let Some(attractors) = &effect.attractors {
            for attractor in attractors {
                let attractor_position = spawner_world_pos + attractor.position.extend(0.0);
                let dx = f32x8::splat(attractor_position.x) - px;
                let dy = f32x8::splat(attractor_position.y) - py;
                let dz = f32x8::splat(attractor_position.z) - pz;
                let distance_squared = dx * dx + dy * dy + dz * dz;
                let non_zero = distance_squared.simd_gt(zero);
                let safe_distance_squared = non_zero.blend(distance_squared, one);
                let force = f32x8::splat(attractor.strength)
                    / safe_distance_squared.max(f32x8::splat(
                        attractor.min_distance * attractor.min_distance,
                    ))
                    * delta8
                    / safe_distance_squared.sqrt();
                let force = non_zero.blend(force, zero);
                vx += dx * force;
                vy += dy * force;
                vz += dz * force;
            }
        }

        let gravity = load8(&particles.gravity_speed, index) * delta8;
        px += vx * delta8 + load8(&particles.gravity_x, index) * gravity;
        py += vy * delta8 + load8(&particles.gravity_y, index) * gravity;
        pz += vz * delta8 + load8(&particles.gravity_z, index) * gravity;
        let rotation = load8(&particles.rotation, index) + angular_velocity * delta8;

        store8(&mut particles.position_x, index, px);
        store8(&mut particles.position_y, index, py);
        store8(&mut particles.position_z, index, pz);
        store8(&mut particles.duration_fraction, index, progress);
        store8(&mut particles.velocity_x, index, vx);
        store8(&mut particles.velocity_y, index, vy);
        store8(&mut particles.velocity_z, index, vz);
        store8(&mut particles.angular_velocity, index, angular_velocity);
        store8(&mut particles.rotation, index, rotation);
    }

    for index in simd_len..particles.len() {
        update_particle_scalar(
            particles,
            effect,
            index,
            delta,
            spawner_world_pos,
            position_delta,
        );
    }

    if let Some(scale_curve) = effect.scale_curve.as_ref() {
        for index in 0..particles.len() {
            let scale = scale_curve.lerp(particles.duration_fraction[index]);
            particles.scale_x[index] = scale;
            particles.scale_y[index] = scale;
            particles.scale_z[index] = scale;
        }
    }

    if let Some(color_curve) = effect.color_curve.as_ref() {
        for index in 0..particles.len() {
            let color = color_curve.lerp(particles.duration_fraction[index]);
            particles.color_r[index] = color.red;
            particles.color_g[index] = color.green;
            particles.color_b[index] = color.blue;
            particles.color_a[index] = color.alpha;
        }
    }
}

fn update_particle_scalar(
    particles: &mut ParticleStore,
    effect: &Particle2dEffect,
    index: usize,
    delta: f32,
    spawner_world_pos: Vec3,
    position_delta: Vec3,
) {
    particles.position_x[index] += position_delta.x;
    particles.position_y[index] += position_delta.y;
    particles.position_z[index] += position_delta.z;
    particles.duration_fraction[index] += delta / particles.duration[index];
    let progress = particles.duration_fraction[index];

    let linear_factor = 1.0
        + progress * (particles.linear_acceleration[index] - particles.linear_damp[index]) * delta;
    particles.velocity_x[index] *= linear_factor;
    particles.velocity_y[index] *= linear_factor;
    particles.velocity_z[index] *= linear_factor;

    let angular_factor = 1.0
        + progress
            * (particles.angular_acceleration[index] - particles.angular_damp[index])
            * delta;
    particles.angular_velocity[index] *= angular_factor;

    if let Some(attractors) = &effect.attractors {
        for attractor in attractors {
            let attractor_position = spawner_world_pos + attractor.position.extend(0.0);
            let dx = attractor_position.x - particles.position_x[index];
            let dy = attractor_position.y - particles.position_y[index];
            let dz = attractor_position.z - particles.position_z[index];
            let distance_squared = dx * dx + dy * dy + dz * dz;

            if distance_squared > 0.0 {
                let force = attractor.strength
                    / distance_squared.max(attractor.min_distance * attractor.min_distance)
                    * delta
                    / distance_squared.sqrt();
                particles.velocity_x[index] += dx * force;
                particles.velocity_y[index] += dy * force;
                particles.velocity_z[index] += dz * force;
            }
        }
    }

    let gravity = particles.gravity_speed[index] * delta;
    particles.position_x[index] +=
        particles.velocity_x[index] * delta + particles.gravity_x[index] * gravity;
    particles.position_y[index] +=
        particles.velocity_y[index] * delta + particles.gravity_y[index] * gravity;
    particles.position_z[index] +=
        particles.velocity_z[index] * delta + particles.gravity_z[index] * gravity;

    particles.rotation[index] += particles.angular_velocity[index] * delta;
}

pub(crate) fn calculate_particle_bounds(
    mut cmd: Commands,
    spawners: Query<(Entity, &ParticleStore, &GlobalTransform), Without<crate::NoAutoAabb>>,
) {
    spawners.iter().for_each(|(entity, store, transform)| {
        if store.is_empty() {
            return;
        }
        let accuracy = (store.len() / 1000).clamp(1, 10);

        let (min, max) =
            (0..store.len())
                .step_by(accuracy)
                .fold((Vec2::MAX, Vec2::MIN), |mut acc, index| {
                    acc.0.x = acc.0.x.min(store.position_x[index]);
                    acc.0.y = acc.0.y.min(store.position_y[index]);
                    acc.1.x = acc.1.x.max(store.position_x[index]);
                    acc.1.y = acc.1.y.max(store.position_y[index]);
                    acc
                });

        let mut aabb = Aabb::from_min_max(min.extend(0.), max.extend(0.));
        aabb.center -= transform.translation().to_vec3a();

        cmd.entity(entity).try_insert(aabb);
    });
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::{
        hint::black_box,
        time::{Duration, Instant},
    };

    fn particle_store(count: usize) -> ParticleStore {
        let mut particles = ParticleStore::default();
        for _ in 0..count {
            particles.push(
                Transform::default(),
                1000.0,
                Vec3::new(10.0, 5.0, 0.0),
                0.5,
                LinearRgba::WHITE,
                0.2,
                0.1,
                0.2,
                0.1,
                9.81,
                Vec3::NEG_Y,
            );
        }
        particles
    }

    #[test]
    fn simd_matches_scalar_update() {
        let effect = Particle2dEffect::default();
        let mut simd = particle_store(8);
        let mut scalar = simd.clone();
        let delta = 1.0 / 60.0;

        update_particles(&mut simd, &effect, delta, Vec3::ZERO, Vec3::ZERO);
        for index in 0..scalar.len() {
            update_particle_scalar(&mut scalar, &effect, index, delta, Vec3::ZERO, Vec3::ZERO);
        }

        for index in 0..simd.len() {
            for (actual, expected) in [
                (simd.position_x[index], scalar.position_x[index]),
                (simd.position_y[index], scalar.position_y[index]),
                (simd.velocity_x[index], scalar.velocity_x[index]),
                (simd.velocity_y[index], scalar.velocity_y[index]),
                (
                    simd.duration_fraction[index],
                    scalar.duration_fraction[index],
                ),
                (simd.rotation[index], scalar.rotation[index]),
            ] {
                assert!((actual - expected).abs() < 1e-5, "{actual} != {expected}");
            }
        }
    }

    #[test]
    #[ignore = "manual performance benchmark"]
    fn bench_update_one_million_particles() {
        const PARTICLES: usize = 1_000_000;
        const WARMUP: usize = 3;
        const SAMPLES: usize = 20;

        let mut particles = particle_store(PARTICLES);
        let effect = Particle2dEffect::default();

        for _ in 0..WARMUP {
            update_particles(
                black_box(&mut particles),
                black_box(&effect),
                black_box(1.0 / 60.0),
                Vec3::ZERO,
                Vec3::ZERO,
            );
        }

        let mut samples = Vec::with_capacity(SAMPLES);
        for _ in 0..SAMPLES {
            let start = Instant::now();
            update_particles(
                black_box(&mut particles),
                black_box(&effect),
                black_box(1.0 / 60.0),
                Vec3::ZERO,
                Vec3::ZERO,
            );
            samples.push(start.elapsed());
        }
        samples.sort_unstable();

        let median = samples[SAMPLES / 2];
        let total: Duration = samples.iter().sum();
        let average = total / SAMPLES as u32;
        let throughput = PARTICLES as f64 / median.as_secs_f64() / 1_000_000.0;

        black_box(&particles);
        println!(
            "1,000,000 particles: median {median:?}, average {average:?}, {throughput:.2} M particles/s"
        );
    }
}
