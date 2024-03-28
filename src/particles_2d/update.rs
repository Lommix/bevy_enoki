use super::{EmissionShape, Particle2dEffect};
use crate::values::Random;
use bevy::prelude::*;
use std::time::Duration;

#[derive(Component, Deref, DerefMut, Reflect)]
pub struct ParticleState(Timer);

impl Default for ParticleState {
    fn default() -> Self {
        Self(Timer::new(Duration::ZERO, TimerMode::Repeating))
    }
}

#[derive(Component, Default)]
pub struct OneShot;

#[derive(Component, Clone, Debug, Reflect)]
pub struct ParticleController {
    pub max_particles: u32,
    pub active: bool,
    pub burst: Option<u32>,
}

#[derive(Component, Deref, DerefMut, Default)]
pub struct ParticleEffectOwner(pub Box<Particle2dEffect>);

impl Default for ParticleController {
    fn default() -> Self {
        Self {
            burst: None,
            max_particles: u32::MAX,
            active: true,
        }
    }
}

#[derive(Component, Default, Clone, Reflect, Deref)]
pub struct ParticleStore(pub Vec<Particle>);

#[derive(Clone, Reflect)]
pub struct Particle {
    pub(crate) transform: Transform,
    pub(crate) lifetime: Timer,
    pub(crate) velocity: (Vec3, f32),
    pub(crate) color: Color,
    pub(crate) frame: u32,
    pub(crate) linear_acceleration: f32,
    pub(crate) linear_damp: f32,
    pub(crate) angular_acceleration: f32,
    pub(crate) angular_damp: f32,
    pub(crate) gravity_speed: f32,
    pub(crate) gravity_direction: Vec2,
}

pub(crate) fn clone_effect(
    mut particle_spawners: Query<
        (&mut ParticleEffectOwner, &Handle<Particle2dEffect>),
        Added<ParticleController>,
    >,
    effects: Res<Assets<Particle2dEffect>>,
) {
    particle_spawners
        .iter_mut()
        .for_each(|(mut effect_overwrites, handle)| {
            let Some(effect) = effects.get(handle) else {
                return;
            };
            effect_overwrites.0 = Box::new(effect.clone());
        });
}

pub(crate) fn remove_finished_spawner(
    mut cmd: Commands,
    spawner: Query<(Entity, &ParticleStore, &ParticleController), With<OneShot>>,
) {
    spawner.iter().for_each(|(entity, store, controller)| {
        if !controller.active && store.len() == 0 {
            cmd.entity(entity).despawn_recursive();
        }
    })
}

pub(crate) fn update_spawner(
    mut particles: Query<(
        Entity,
        &mut ParticleStore,
        &mut ParticleState,
        &mut ParticleController,
        // &ParticleEffectOwner,
        &Handle<Particle2dEffect>,
        &GlobalTransform,
    )>,
    effects: Res<Assets<Particle2dEffect>>,
    one_shots: Query<&OneShot>,
    time: Res<Time>,
) {
    particles.par_iter_mut().for_each(
        |(entity, mut store, mut state, mut controller, handle, transform)| {
            if controller.max_particles <= store.0.len() as u32 {
                return;
            }

            let Some(effect) = effects.get(handle) else {
                return;
            };

            let transform = transform.compute_transform();

            state.set_duration(Duration::from_secs_f32(effect.spawn_rate));
            state.tick(time.delta());

            if state.finished() && controller.active {
                for _ in 0..effect.spawn_amount {
                    store.0.push(create_particle(&effect, &transform))
                }

                if one_shots.get(entity).is_ok() {
                    controller.active = false;
                }

                let finished_burst = controller
                    .burst
                    .as_mut()
                    .map(|counter| match counter.checked_sub(effect.spawn_amount) {
                        Some(remaining) => {
                            *counter = remaining;
                            false
                        }
                        None => true,
                    })
                    .unwrap_or_default();

                if finished_burst {
                    controller.active = false;
                }
            }

            // state.set_duration(Duration::from_secs_f32(effect.spawn_rate));
            store.0.retain_mut(|particle| {
                update_particle(particle, &effect, time.delta_seconds());
                particle.lifetime.tick(time.delta());
                !particle.lifetime.finished()
            });
        },
    );
}

#[rustfmt::skip]
#[inline]
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
    let scale = effect
        .scale
        .as_ref()
        .map(|s| s.rand())
        .unwrap_or_default();

    let gravity_direction = effect
        .gravity_direction
        .as_ref()
        .map(|g| g.rand())
        .unwrap_or_default();

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

    let mut transform = transform.clone();
    transform.scale = Vec3::splat(scale);

    transform.translation += match effect.emission_shape{
        EmissionShape::Point => Vec3::ZERO,
        EmissionShape::Circle(radius) => {
            Vec3::new(rand::random::<f32>() - 0.5,rand::random::<f32>() - 0.5,0.).normalize_or_zero() * radius * rand::random::<f32>()
        }
    };

    Particle {
        transform,
        velocity: ((direction * speed).extend(0.), angular),
        lifetime: Timer::new(Duration::from_secs_f32(effect.lifetime.rand()), TimerMode::Once),
        color: effect.color.unwrap_or(Color::WHITE),
        angular_damp,
        linear_damp,
        angular_acceleration,
        linear_acceleration,
        gravity_direction,
        gravity_speed,
        frame: 0,
    }
}

#[inline]
fn update_particle(particle: &mut Particle, effect: &Particle2dEffect, delta: f32) {
    let (lin_velo, rot_velo) = &mut particle.velocity;
    let progess = particle.lifetime.fraction();

    *lin_velo = *lin_velo - progess * particle.linear_damp * *lin_velo * delta
        + progess * particle.linear_acceleration * *lin_velo * delta;

    *rot_velo = *rot_velo - progess * particle.angular_damp * *rot_velo * delta
        + progess * particle.angular_acceleration * *rot_velo * delta;

    // particle.velocity.0 = particle.velocity.0
    //     - particle.lifetime.fraction() * particle.linear_damp * particle.velocity.0 * delta;
    //
    // particle.velocity.1 = particle.velocity.1
    //     - particle.lifetime.fraction() * particle.angular_damp * particle.velocity.1 * delta;

    if let Some(scale_curve) = effect.scale_curve.as_ref() {
        particle.transform.scale = Vec3::splat(scale_curve.lerp(progess));
    }

    if let Some(color_curve) = effect.color_curve.as_ref() {
        particle.color = color_curve.lerp(progess);
    }

    // particle.velocity.0 += particle.velocity.0 * particle.linear_acceleration * delta;
    // particle.velocity.1 += particle.velocity.1 * particle.angular_acceleration * delta;

    let gravity = particle.gravity_direction * particle.gravity_speed * delta;

    particle.transform.translation += *lin_velo * delta + gravity.extend(0.);
    particle.transform.rotate_local_z(*rot_velo * delta);
}
