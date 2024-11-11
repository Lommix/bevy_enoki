<div style="display:flex;gap:1rem;">
    <img src="docs/icon.png" width="45" height="64">
    <h1 style="border:none">Bevy Enoki</h1>
</div>

[![License: MIT or Apache 2.0](https://img.shields.io/badge/License-MIT%20or%20Apache2-blue.svg)](./LICENSE)
[![Crate](https://img.shields.io/crates/v/bevy_enoki.svg)](https://crates.io/crates/bevy_enoki)

Enoki - A 2D particle system for the Bevy game engine.

![animation](docs/output.gif)

## Overview

The Enoki particle system is a CPU calculate particle system, that uses GPU Instancing and works well with `wasm`. It provides a `material trait`
that lets you easily implement your own fragment shader for all your VFX needs.
Additionally, spawner configuration are provided via `ron` files, which can be hot reloaded.

The default material allows not only for custom textures, but also sprite sheet animations over the particle lifetime.

🚧 This Plugin just released and is still under heavy development. The particle behavior is very basic and performance could be better.
Expect rapid change. Planned is orbital velocity, attractors and simple physics. When 2d is fleshed out and the best it can be, 3d might happen.

## Compatibility

| bevy | bevy_enoki |
| ---: | ---------: |
| 0.15 |       main |
| 0.14 |      0.2.2 |
| 0.13 |        0.1 |

## Editor

[WIP]

---


## Examples

```shell
cargo run -p example --bin material
cargo run -p example --bin sprites
cargo run -p example --bin dynamic
```

## Usage

Add the `bevy_enoki` dependency to your `Cargo.toml`

```toml
bevy_enoki = "0.2.2"
```

Add the `EnokiPlugin` to your app

```rust
App::new()
    .add_plugins(DefaultPlugins)
    .add_plugins(EnokiPlugin)
    .run()
```

Create your first particle spawner. [Here is a default effect config](assets/base.particle.ron)

```rust
use bevy_enoki::prelude::*;

fn setup(
    mut cmd : Commands,
    mut materials: ResMut<Assets<SpriteParticle2dMaterial>>,
    server : Res<AssetServer>,
){
    cmd.spawn(Camera2dBundle::default());

    //spawning quads
    cmd.spawn((
        ParticleSpawnerBundle {
            // load the effect configuration from a ron file
            effect: server.load("fire.particle.ron"),
            // the default materiel is just a flat white color
            // that gets multiplied by any color curve inside the
            // effect definition of the `ron` file.
            material: DEFAULT_MATERIAL,
            ..default()
        },
    ));

    // if you want to add a texture, use the inbuild `ColorParticle2dMaterial`
    let texture_material = materials.add(
        // hframes and vframes define how the sprite sheet is divided for animations,
        // if you just want to bind a single texture, leave both at 1.
        SpriteParticle2dMaterial::new(server.load("particle.png"), 6, 1),
    );

    cmd.spawn((
        ParticleSpawnerBundle {
            effect: server.load("fire.particle.ron"),
            material: texture_material,
            ..default()
        },
    ));
}
```

## Create a custom Material

Just like any other Bevy material, you can define your own
fragment shader.

```rust
#[derive(AsBindGroup, Asset, TypePath, Clone, Default)]
pub struct FireParticleMaterial {
    #[texture(0)]
    #[sampler(1)]
    texture: Handle<Image>,
}

impl Particle2dMaterial for FireParticleMaterial {
    fn fragment_shader() -> bevy::render::render_resource::ShaderRef {
        "custom_material.wgsl".into()
    }
}

fn setup(){
    App::default()
        .add_plugins(DefaultPlugins)
        .add_plugins(EnokiPlugin)
        .add_plugins(Particle2dMaterialPlugin::<FireParticleMaterial>::default())
        .run()
}
```

Now create the Shader

```wgsl
//assets/custom_material.wgsl
#import bevy_enoki::particle_vertex_out::{ VertexOutput }

@group(1) @binding(0) var texture: texture_2d<f32>;
@group(1) @binding(1) var texture_sampler: sampler;


@fragment
fn fragment(in: VertexOutput) -> @location(0) vec4<f32> {
    var out = in.color
    // go wild
    return out;
}
```

That's it, now add the Material to your Spawner! These are the values provided by the vertex shader:

```wgsl
struct VertexOutput {
  @builtin(position) clip_position: vec4<f32>,
  @location(0) @interpolate(flat) color: vec4<f32>,
  @location(1) uv : vec2<f32>,
  @location(2) lifetime_frac : f32,
  @location(3) lifetime_total : f32,
};
```

## The Effect Asset

[Here is a default ron config](assets/base.particle.ron)

```rust
#[derive(Deserialize, Default, Clone, Debug)]
pub enum EmissionShape {
    #[default]
    Point,
    Circle(f32),
}

#[derive(Asset, TypePath, Default, Deserialize, Clone, Debug)]
pub struct Particle2dEffect {
    pub spawn_rate: f32,
    pub spawn_amount: u32,
    pub emission_shape: EmissionShape,
    pub lifetime: Rval<f32>,
    pub linear_speed: Option<Rval<f32>>,
    pub linear_acceleration: Option<Rval<f32>>,
    pub direction: Option<Rval<Vec2>>,
    pub angular_speed: Option<Rval<f32>>,
    pub angular_acceleration: Option<Rval<f32>>,
    pub scale: Option<Rval<f32>>,
    pub color: Option<LinearRgba>,
    pub gravity_direction: Option<Rval<Vec2>>,
    pub gravity_speed: Option<Rval<f32>>,
    pub linear_damp: Option<Rval<f32>>,
    pub angular_damp: Option<Rval<f32>>,
    pub scale_curve: Option<Curve<f32>>,
    pub color_curve: Option<Curve<LinearRgba>>,
}
```

This how you create a `Curve`. Currently, Supports `LinearRgba` and `f32`.
`RVal` stands for any Value with a randomness property between 0 - 1.

```rust
let curve = Curve::new()
    .with_point(LinearRgba::RED, 0.0, None)
    .with_point(LinearRgba::BLUE, 1.0, Some(bevy_enoki::prelude::EaseFunction::SineInOut));

// max 1.0, randomness of 0.1 (0.9 - 1.1)
let rval = Rval::new(1.0, 0.1);
```
