use super::{prelude::EmissionShape, Particle2dEffect, ParticleEffectHandle};
use crate::values::Random;
use bevy::{
    camera::primitives::Aabb,
    prelude::*,
    tasks::{ComputeTaskPool, ParallelSliceMut},
};
use std::{ops::AddAssign, time::Duration};

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
#[derive(Component, Default, Clone, Reflect, Deref, DerefMut)]
pub struct ParticleStore {
    #[deref]
    pub particles: Vec<Particle>,
}

#[derive(Clone, Reflect)]
pub struct Particle {
    pub(crate) transform: Transform,
    pub(crate) duration: f32,
    pub(crate) duration_fraction: f32,
    pub(crate) velocity: (Vec3, f32),
    pub(crate) color: LinearRgba,
    pub(crate) frame: u32,
    pub(crate) linear_acceleration: f32,
    pub(crate) linear_damp: f32,
    pub(crate) angular_acceleration: f32,
    pub(crate) angular_damp: f32,
    pub(crate) gravity_speed: f32,
    pub(crate) gravity_direction: Vec3,
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
                cmd.entity(entity).despawn();
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
            if state.max_particles <= store.particles.len() as u32 {
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
                    store.push(create_particle(effect, &transform))
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

            store.par_splat_map_mut(ComputeTaskPool::get(), None, |_, particles| {
                for particle in particles.iter_mut() {
                    // position_delta should only be a value if relative_positioning is enabled
                    particle.transform.translation += position_delta;

                    particle
                        .duration_fraction
                        .add_assign(delta / particle.duration);
                    update_particle(particle, effect, delta, spawner_world_pos);
                }
            });
            store.retain(|particle| particle.duration_fraction < 1.0);
        },
    );
}

fn create_particle(effect: &Particle2dEffect, transform: &Transform) -> Particle {
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

    Particle {
        transform,
        velocity: ((direction * speed).extend(0.), angular),
        duration_fraction: 0.0,
        duration: effect.lifetime.rand(),
        color: effect.color.unwrap_or(LinearRgba::WHITE),
        angular_damp,
        linear_damp,
        angular_acceleration,
        linear_acceleration,
        gravity_direction,
        gravity_speed,
        frame: 0,
    }
}

fn update_particle(
    particle: &mut Particle,
    effect: &Particle2dEffect,
    delta: f32,
    spawner_world_pos: Vec3,
) {
    let (lin_velo, rot_velo) = &mut particle.velocity;
    let progress = particle.duration_fraction;

    *lin_velo = *lin_velo - progress * particle.linear_damp * *lin_velo * delta
        + progress * particle.linear_acceleration * *lin_velo * delta;

    *rot_velo = *rot_velo - progress * particle.angular_damp * *rot_velo * delta
        + progress * particle.angular_acceleration * *rot_velo * delta;

    if let Some(scale_curve) = effect.scale_curve.as_ref() {
        particle.transform.scale = Vec3::splat(scale_curve.lerp(progress));
    }

    if let Some(color_curve) = effect.color_curve.as_ref() {
        particle.color = color_curve.lerp(progress);
    }

    // Apply attractor forces from effect
    if let Some(attractors) = &effect.attractors {
        for attractor in attractors.iter() {
            // Transform attractor position from local to world space
            let attractor_world_pos = spawner_world_pos + attractor.position.extend(0.0);
            let to_attractor = attractor_world_pos - particle.transform.translation;
            let distance_sq = to_attractor.length_squared();

            if distance_sq > 0.0 {
                let distance = distance_sq.sqrt();
                let min_distance_sq = attractor.min_distance * attractor.min_distance;
                let force_magnitude = attractor.strength / distance_sq.max(min_distance_sq);
                let force_direction = to_attractor / distance;

                *lin_velo += force_direction * force_magnitude * delta;
            }
        }
    }

    let gravity = particle.gravity_direction * particle.gravity_speed * delta;

    particle.transform.translation += *lin_velo * delta + gravity;
    particle.transform.rotate_local_z(*rot_velo * delta);
}

pub(crate) fn calculcate_particle_bounds(
    mut cmd: Commands,
    spawners: Query<(Entity, &ParticleStore), Without<crate::NoAutoAabb>>,
) {
    spawners.iter().for_each(|(entity, store)| {
        if store.is_empty() {
            return;
        }
        let accuracy = (store.len() / 1000).clamp(1, 10);

        let (min, max) = store
            .iter()
            .enumerate()
            .filter(|(i, _)| i % accuracy == 0)
            .fold((Vec2::ZERO, Vec2::ZERO), |mut acc, (_, particle)| {
                acc.0.x = acc.0.x.min(particle.transform.translation.x);
                acc.0.y = acc.0.y.min(particle.transform.translation.y);
                acc.1.x = acc.1.x.max(particle.transform.translation.x);
                acc.1.y = acc.1.y.max(particle.transform.translation.y);
                acc
            });
        cmd.entity(entity)
            .try_insert(Aabb::from_min_max(min.extend(0.), max.extend(0.)));
    });
}
