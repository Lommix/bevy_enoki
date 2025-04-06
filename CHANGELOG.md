# 0.4.0

- bevy 0.16 update. No breaking changings.

# 0.3.3

- Particles now update (not render) if not visible by default.

# 0.3.2

- switched to bevy's new curve crate for interpolation.
  !BREAKINGCHANGE! In Ron effect rename `Curve` to `MultiCurve`. Explicit naming the struct
  name is typical optional. So it might not effect you.

# 0.3.1

- Restricted `ParticleSpawner` Asset Handle to a `Particle2dMaterial`

# 0.3.0 Release - Bevy 0.15

## Renaming

- Changed `ParticleSpawnerState` to just `ParticleSpawner`, which now uses `required components` instead of a Bundle.
    - Additionally for anything to render. The spawner entity, requires a MaterialHandle and a EffectHandle.
- Changed `Handle<Particle2dEffect>` to `EffectHandle`,
- Changed `Handle<ParticleMaterial>` to `MaterialHandle<ParticleMaterial>`,

## required components

- The crate uses required Components now. Which made it way simple to spawn particles, where you
  do not need a custom material.

# 0.2.0 Release - Bevy 0.14

- changed color api in ron files. See example

# 0.1.0 Release - Bevy 0.13

- initial realeae
