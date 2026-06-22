use crate::RenderParticleTag;

use super::{ParticleSpawner, ParticleStore};
use bevy_app::{App, Plugin};
use bevy_asset::{Asset, AssetApp, AssetEvent, AssetId, AssetServer, Assets, Handle};
use bevy_camera::visibility::ViewVisibility;
use bevy_core_pipeline::core_2d::{Transparent2d, CORE_2D_DEPTH_FORMAT};
use bevy_derive::{Deref, DerefMut};
use bevy_ecs::{
    component::Component,
    entity::EntityHashMap,
    message::MessageReader,
    resource::Resource,
    schedule::IntoScheduleConfigs,
    system::{
        lifetimeless::{Read, SRes},
        Commands, Query, Res, ResMut, SystemParamItem,
    },
    world::{FromWorld, World},
};
use bevy_math::{FloatOrd, Vec4};
use bevy_mesh::{PrimitiveTopology, VertexBufferLayout};
use bevy_reflect::Reflect;
use bevy_render::{
    render_asset::{PrepareAssetError, RenderAsset, RenderAssetPlugin, RenderAssets},
    render_phase::{
        AddRenderCommand, DrawFunctions, PhaseItem, PhaseItemExtraIndex, RenderCommand,
        RenderCommandResult, SetItemPipeline, TrackedRenderPass, ViewSortedRenderPhases,
    },
    render_resource::{
        binding_types::uniform_buffer, AsBindGroup, AsBindGroupError, BindGroup, BindGroupEntries,
        BindGroupLayoutDescriptor, BindGroupLayoutEntries, BlendState, BufferUsages, BufferVec,
        ColorTargetState, ColorWrites, CompareFunction, DepthBiasState, DepthStencilState,
        FrontFace, IndexFormat, OwnedBindingResource, PipelineCache, PolygonMode, PrimitiveState,
        RenderPipelineDescriptor, ShaderStages, ShaderType, SpecializedRenderPipeline,
        SpecializedRenderPipelines, StencilFaceState, StencilState, VertexAttribute, VertexFormat,
        VertexStepMode,
    },
    renderer::{RenderDevice, RenderQueue},
    sync_world::RenderEntity,
    view::{
        ExtractedView, Msaa, RenderVisibleEntities, ViewUniform, ViewUniformOffset, ViewUniforms,
    },
    Extract, ExtractSchedule, Render, RenderApp, RenderSystems,
};
use bevy_shader::{Shader, ShaderRef};
use bevy_sprite_render::Mesh2dPipelineKey;
use bevy_transform::components::GlobalTransform;
use std::{hash::Hash, ops::Range};

/// Particle Material Trait
/// bind custom fragment shader to material
pub trait Particle2dMaterial: AsBindGroup + Asset + Clone + Sized {
    fn fragment_shader() -> ShaderRef {
        super::PARTICLE_COLOR_FRAG.into()
    }
}

pub struct Particle2dMaterialPlugin<M: Particle2dMaterial> {
    _m: std::marker::PhantomData<M>,
}

impl<M: Particle2dMaterial> Default for Particle2dMaterialPlugin<M> {
    fn default() -> Self {
        Self {
            _m: std::marker::PhantomData::<M>,
        }
    }
}

impl<M: Particle2dMaterial> Plugin for Particle2dMaterialPlugin<M> {
    fn build(&self, app: &mut App) {
        app.init_asset::<M>();

        app.add_plugins(RenderAssetPlugin::<PreparedParticleMaterial<M>>::default());
        app.sub_app_mut(RenderApp)
            .add_render_command::<Transparent2d, DrawParticle2d<M>>()
            .init_resource::<SpecializedRenderPipelines<Particle2dPipeline<M>>>()
            .init_resource::<ExtracedParticleSpawner<M>>()
            .init_resource::<ExtractedParticleMaterials<M>>()
            .init_resource::<RenderParticleMaterials<M>>()
            .add_systems(
                ExtractSchedule,
                (extract_particles::<M>, extract_materials::<M>),
            )
            .add_systems(
                Render,
                (
                    queue_particles::<M>.in_set(RenderSystems::Queue),
                    prepare_particles_instance_buffers::<M>.in_set(RenderSystems::PrepareResources),
                ),
            );
    }

    fn finish(&self, app: &mut App) {
        let render_app = app.sub_app_mut(RenderApp);
        render_app.init_resource::<Particle2dPipeline<M>>();
        render_app.init_resource::<InstanceBuffer<M>>();

        let particle_buffer = {
            let render_device = render_app.world().resource::<RenderDevice>();
            let render_queue = render_app.world().resource::<RenderQueue>();

            let mut particle_buffer = InstanceBuffer::<M>::default();
            particle_buffer.index_buffer.push(2);
            particle_buffer.index_buffer.push(0);
            particle_buffer.index_buffer.push(1);
            particle_buffer.index_buffer.push(1);
            particle_buffer.index_buffer.push(3);
            particle_buffer.index_buffer.push(2);
            particle_buffer
                .index_buffer
                .write_buffer(render_device, render_queue);
            particle_buffer
        };

        render_app.insert_resource(particle_buffer);
    }
}

#[derive(Resource)]
pub struct ExtractedParticleMaterials<M: Particle2dMaterial> {
    materials: Vec<(AssetId<M>, M)>,
}

impl<M: Particle2dMaterial> Default for ExtractedParticleMaterials<M> {
    fn default() -> Self {
        Self {
            materials: Vec::default(),
        }
    }
}

#[derive(Resource, Debug)]
pub struct ExtracedParticleSpawner<M: Particle2dMaterial> {
    particles: EntityHashMap<Range<u32>>,
    _m: std::marker::PhantomData<M>,
}

impl<M: Particle2dMaterial> Default for ExtracedParticleSpawner<M> {
    fn default() -> Self {
        Self {
            particles: Default::default(),
            _m: Default::default(),
        }
    }
}

// ----------------------------------------------
// #extract

fn extract_materials<M: Particle2dMaterial>(
    mut events: Extract<MessageReader<AssetEvent<M>>>,
    mut materials: ResMut<ExtractedParticleMaterials<M>>,
    assets: Extract<Res<Assets<M>>>,
) {
    for event in events.read() {
        match event {
            AssetEvent::Added { id } | AssetEvent::Modified { id } => {
                if let Some(asset) = assets.get(*id) {
                    materials.materials.push((*id, asset.clone()));
                }
            }
            AssetEvent::Removed { id } => {
                materials.materials.retain(|(i, _)| i != id);
            }
            _ => (),
        }
    }
}
#[allow(clippy::type_complexity)]
fn extract_particles<M: Particle2dMaterial>(
    mut cmd: Commands,
    mut extraced_batches: ResMut<ExtracedParticleSpawner<M>>,
    mut render_material_instances: ResMut<RenderParticleMaterials<M>>,
    mut particle_buffer: ResMut<InstanceBuffer<M>>,
    query: Extract<
        Query<(
            &ParticleStore,
            &GlobalTransform,
            &ParticleSpawner<M>,
            &ViewVisibility,
            &RenderEntity,
        )>,
    >,
) {
    extraced_batches.particles.clear();
    particle_buffer.instance_buffer.clear();
    query.iter().for_each(|emitter| {
        let (particle_store, global, material_handle, visbility, render_entity) = emitter;
        if !visbility.get() || particle_store.is_empty() {
            return;
        }

        cmd.entity(**render_entity)
            .insert((ZOrder(FloatOrd(global.translation().z)), ParticleTag));
        let start = particle_buffer.instance_buffer.len() as u32;
        for index in 0..particle_store.len() {
            particle_buffer
                .instance_buffer
                .push(InstanceData::from_store(particle_store, index));
        }
        let end = particle_buffer.instance_buffer.len() as u32;
        render_material_instances.insert(**render_entity, material_handle.id());
        extraced_batches
            .particles
            .insert(**render_entity, start..end);
    });
}

#[derive(Component, Default)]
pub struct ParticleTag;

#[derive(Component, Deref)]
pub struct ZOrder(FloatOrd);

// ----------------------------------------------
// #queue
#[allow(clippy::too_many_arguments)]
fn queue_particles<M: Particle2dMaterial>(
    transparent_2d_draw_functions: Res<DrawFunctions<Transparent2d>>,
    custom_pipeline: Res<Particle2dPipeline<M>>,
    mut pipelines: ResMut<SpecializedRenderPipelines<Particle2dPipeline<M>>>,
    pipeline_cache: Res<PipelineCache>,
    extract_particles: Res<ExtracedParticleSpawner<M>>,
    z_orders: Query<&ZOrder>,
    views: Query<(&ExtractedView, &RenderVisibleEntities, &Msaa)>,
    mut render_phases: ResMut<ViewSortedRenderPhases<Transparent2d>>,
) {
    let draw_particles = transparent_2d_draw_functions
        .read()
        .id::<DrawParticle2d<M>>();

    for (view, visible_entities, msaa) in &views {
        let Some(transparent_phase) = render_phases.get_mut(&view.retained_view_entity) else {
            continue;
        };

        let mesh_key = Mesh2dPipelineKey::from_msaa_samples(msaa.samples())
            | Mesh2dPipelineKey::from_target_format(view.target_format);

        let key = Particle2dPipelineKey { mesh_key };
        let pipeline = pipelines.specialize(&pipeline_cache, &custom_pipeline, key);

        let Some(visible_entities) = visible_entities.get::<RenderParticleTag>() else {
            continue;
        };

        for (entity, main_entity) in visible_entities.iter_visible() {
            if extract_particles.particles.get(entity).is_none() {
                continue;
            }

            let Ok(order) = z_orders.get(*entity) else {
                return;
            };

            transparent_phase.add_transient(Transparent2d {
                extracted_index: 0,
                indexed: false,
                extra_index: PhaseItemExtraIndex::None,
                sort_key: **order,
                entity: (*entity, *main_entity),
                pipeline,
                draw_function: draw_particles,
                batch_range: 0..1,
            });
        }
    }
}

// ----------------------------------------------
//

#[derive(Clone, Debug, Copy, ShaderType, Reflect)]
pub struct InstanceData {
    transform: Vec4,
    scale_lifetime: Vec4,
    color: Vec4,
}

impl InstanceData {
    #[inline(always)]
    fn from_store(store: &ParticleStore, index: usize) -> Self {
        Self {
            // xyz is world position, including depth; w is the 2D angle.
            transform: Vec4::new(
                store.position_x[index],
                store.position_y[index],
                store.position_z[index],
                store.rotation[index],
            ),
            scale_lifetime: Vec4::new(
                store.scale_x[index],
                store.scale_y[index],
                store.duration_fraction[index],
                store.duration[index],
            ),
            color: Vec4::new(
                store.color_r[index],
                store.color_g[index],
                store.color_b[index],
                store.color_a[index],
            ),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::{
        hint::black_box,
        time::{Duration, Instant},
    };

    #[test]
    #[ignore = "manual performance benchmark"]
    fn bench_pack_one_million_particles_into_render_buffer() {
        const PARTICLES: usize = 1_000_000;
        const WARMUP: usize = 3;
        const SAMPLES: usize = 20;

        let store = ParticleStore {
            position_x: vec![1.0; PARTICLES],
            position_y: vec![2.0; PARTICLES],
            position_z: vec![3.0; PARTICLES],
            rotation: vec![0.5; PARTICLES],
            scale_x: vec![1.0; PARTICLES],
            scale_y: vec![1.0; PARTICLES],
            duration: vec![10.0; PARTICLES],
            duration_fraction: vec![0.5; PARTICLES],
            color_r: vec![1.0; PARTICLES],
            color_g: vec![0.5; PARTICLES],
            color_b: vec![0.25; PARTICLES],
            color_a: vec![1.0; PARTICLES],
            ..Default::default()
        };
        let mut buffer = BufferVec::new(BufferUsages::VERTEX);

        let pack = |buffer: &mut BufferVec<InstanceData>| {
            buffer.clear();
            for index in 0..store.len() {
                buffer.push(InstanceData::from_store(&store, index));
            }
        };

        for _ in 0..WARMUP {
            pack(black_box(&mut buffer));
        }

        let mut samples = Vec::with_capacity(SAMPLES);
        for _ in 0..SAMPLES {
            let start = Instant::now();
            pack(black_box(&mut buffer));
            samples.push(start.elapsed());
            black_box(&buffer);
        }
        samples.sort_unstable();

        let median = samples[SAMPLES / 2];
        let total: Duration = samples.iter().sum();
        let average = total / SAMPLES as u32;
        let throughput = PARTICLES as f64 / median.as_secs_f64() / 1_000_000.0;

        println!(
            "pack 1,000,000 particles: median {median:?}, average {average:?}, {throughput:.2} M particles/s"
        );
    }

    #[test]
    fn compact_instance_preserves_particle_data() {
        assert_eq!(u64::from(InstanceData::min_size()), 48);

        let mut store = ParticleStore::default();
        store.position_x.push(1.0);
        store.position_y.push(2.0);
        store.position_z.push(3.0);
        store.rotation.push(0.5);
        store.scale_x.push(4.0);
        store.scale_y.push(5.0);
        store.scale_z.push(6.0);
        store.duration.push(10.0);
        store.duration_fraction.push(0.25);
        store.color_r.push(1.0);
        store.color_g.push(0.5);
        store.color_b.push(0.25);
        store.color_a.push(1.0);

        let instance = InstanceData::from_store(&store, 0);
        assert_eq!(instance.transform, Vec4::new(1.0, 2.0, 3.0, 0.5));
        assert_eq!(instance.scale_lifetime, Vec4::new(4.0, 5.0, 0.25, 10.0));
    }
}

// #[derive(Component, Deref)]
// pub struct InstanceMaterialData(Vec<InstanceData>);

#[derive(Resource)]
pub struct PreparedParticleMaterial<M: Particle2dMaterial> {
    pub bind_group: BindGroup,
    pub _bindings: Vec<(u32, OwnedBindingResource)>,
    pub _key: Option<M::Data>,
}

impl<M: Particle2dMaterial> RenderAsset for PreparedParticleMaterial<M> {
    type SourceAsset = M;
    type Param = (
        SRes<RenderDevice>,
        SRes<PipelineCache>,
        SRes<Particle2dPipeline<M>>,
        M::Param,
    );

    fn prepare_asset(
        material: Self::SourceAsset,
        _: AssetId<Self::SourceAsset>,
        (render_device, pipeline_cache, pipeline, param): &mut SystemParamItem<Self::Param>,
        _: Option<&Self>,
    ) -> Result<Self, bevy_render::render_asset::PrepareAssetError<Self::SourceAsset>> {
        match material.as_bind_group(
            &pipeline.uniform_layout,
            render_device,
            pipeline_cache,
            param,
        ) {
            Ok(prepared) => Ok(PreparedParticleMaterial {
                bind_group: prepared.bind_group,
                _bindings: prepared.bindings.0,
                _key: None,
            }),
            Err(AsBindGroupError::RetryNextUpdate) => {
                Err(PrepareAssetError::RetryNextUpdate(material))
            }
            Err(other) => Err(PrepareAssetError::AsBindGroupError(other)),
        }
    }
}

#[derive(Resource, DerefMut, Deref)]
pub struct RenderParticleMaterials<M: Particle2dMaterial>(EntityHashMap<AssetId<M>>);

impl<M: Particle2dMaterial> Default for RenderParticleMaterials<M> {
    fn default() -> Self {
        Self(EntityHashMap::default())
    }
}

// -----------------------------------
// #prep

#[allow(clippy::too_many_arguments)]
fn prepare_particles_instance_buffers<M: Particle2dMaterial>(
    mut cmd: Commands,
    extracted_spawner: Res<ExtracedParticleSpawner<M>>,
    render_device: Res<RenderDevice>,
    render_queue: Res<RenderQueue>,
    view_uniforms: Res<ViewUniforms>,
    particle_pipeline: Res<Particle2dPipeline<M>>,
    pipeline_cache: Res<PipelineCache>,
    mut particle_buffer: ResMut<InstanceBuffer<M>>,
) {
    if let Some(view_binding) = view_uniforms.uniforms.binding() {
        particle_buffer.view_bind_group = Some(render_device.create_bind_group(
            "particle_view_bind_group",
            &pipeline_cache.get_bind_group_layout(&particle_pipeline.view_layout),
            &BindGroupEntries::single(view_binding),
        ));
    }

    for (entity, range) in extracted_spawner.particles.iter() {
        if range.is_empty() {
            continue;
        }
        cmd.entity(*entity).insert(ParticleInstanceBatch {
            range: range.clone(),
        });
    }

    particle_buffer
        .instance_buffer
        .write_buffer(&render_device, &render_queue);
}

#[derive(Resource)]
pub struct InstanceBuffer<M: Particle2dMaterial> {
    view_bind_group: Option<BindGroup>,
    instance_buffer: BufferVec<InstanceData>,
    index_buffer: BufferVec<u32>,
    _m: std::marker::PhantomData<M>,
}

impl<M: Particle2dMaterial> Default for InstanceBuffer<M> {
    fn default() -> Self {
        Self {
            view_bind_group: None,
            instance_buffer: BufferVec::<InstanceData>::new(BufferUsages::VERTEX),
            index_buffer: BufferVec::<u32>::new(BufferUsages::INDEX),
            _m: Default::default(),
        }
    }
}

#[derive(Component, Debug)]
pub struct ParticleInstanceBatch {
    pub range: Range<u32>,
}
// ----------------------------------------------
// pipeline

#[derive(Resource)]
pub struct Particle2dPipeline<M: Particle2dMaterial> {
    vertex_shader: Handle<Shader>,
    fragment_shader: Handle<Shader>,
    uniform_layout: BindGroupLayoutDescriptor,
    view_layout: BindGroupLayoutDescriptor,
    _m: std::marker::PhantomData<M>,
}

#[derive(PartialEq, Eq, Hash, Clone)]
pub struct Particle2dPipelineKey {
    mesh_key: Mesh2dPipelineKey,
}

impl<M: Particle2dMaterial> FromWorld for Particle2dPipeline<M> {
    fn from_world(world: &mut World) -> Self {
        let server = world.resource::<AssetServer>();

        let fragment_shader = match M::fragment_shader() {
            ShaderRef::Default => super::PARTICLE_COLOR_FRAG,
            ShaderRef::Handle(handle) => handle,
            ShaderRef::Path(path) => server.load(path),
        };

        let vertex_shader = super::PARTICLE_VERTEX;
        let render_device = world.resource::<RenderDevice>();

        let view_layout = BindGroupLayoutDescriptor::new(
            "particle_view_layout",
            &BindGroupLayoutEntries::single(
                ShaderStages::VERTEX_FRAGMENT,
                uniform_buffer::<ViewUniform>(true),
            ),
        );

        Particle2dPipeline {
            view_layout,
            uniform_layout: M::bind_group_layout_descriptor(render_device), //world.resource::<ParticleUniformLayout>().0.clone(),
            vertex_shader,
            fragment_shader,
            _m: std::marker::PhantomData::<M>,
        }
    }
}

impl<M: Particle2dMaterial> SpecializedRenderPipeline for Particle2dPipeline<M> {
    type Key = Particle2dPipelineKey;

    fn specialize(&self, key: Self::Key) -> RenderPipelineDescriptor {
        let layout = vec![self.view_layout.clone(), self.uniform_layout.clone()];

        RenderPipelineDescriptor {
            zero_initialize_workgroup_memory: true,
            vertex: bevy_render::render_resource::VertexState {
                shader: self.vertex_shader.clone(),
                shader_defs: vec![],
                entry_point: Some("vertex".into()),
                buffers: vec![VertexBufferLayout {
                    array_stride: 48,
                    step_mode: VertexStepMode::Instance,
                    attributes: vec![
                        // xyz position, z retains particle depth; w rotation
                        VertexAttribute {
                            format: VertexFormat::Float32x4,
                            offset: 0,
                            shader_location: 0,
                        },
                        // xy scale, zw lifetime
                        VertexAttribute {
                            format: VertexFormat::Float32x4,
                            offset: 16,
                            shader_location: 1,
                        },
                        // color
                        VertexAttribute {
                            format: VertexFormat::Float32x4,
                            offset: 32,
                            shader_location: 2,
                        },
                    ],
                }],
            },
            fragment: Some(bevy_render::render_resource::FragmentState {
                shader: self.fragment_shader.clone(),
                shader_defs: vec![],
                entry_point: Some("fragment".into()),
                targets: vec![Some(ColorTargetState {
                    format: key.mesh_key.target_format(),
                    blend: Some(BlendState::ALPHA_BLENDING),
                    write_mask: ColorWrites::ALL,
                })],
            }),
            label: Some("particle 2d pipeline".into()),
            layout,
            immediate_size: 0,
            primitive: PrimitiveState {
                front_face: FrontFace::Ccw,
                cull_mode: None,
                unclipped_depth: false,
                polygon_mode: PolygonMode::Fill,
                conservative: false,
                topology: PrimitiveTopology::TriangleList,
                strip_index_format: None,
            },
            depth_stencil: Some(DepthStencilState {
                format: CORE_2D_DEPTH_FORMAT,
                depth_write_enabled: Some(false),
                depth_compare: Some(CompareFunction::GreaterEqual),
                stencil: StencilState {
                    front: StencilFaceState::IGNORE,
                    back: StencilFaceState::IGNORE,
                    read_mask: 0,
                    write_mask: 0,
                },
                bias: DepthBiasState {
                    constant: 0,
                    slope_scale: 0.0,
                    clamp: 0.0,
                },
            }),
            multisample: bevy_render::render_resource::MultisampleState {
                count: key.mesh_key.msaa_samples(),
                mask: !0,
                alpha_to_coverage_enabled: false,
            },
        }
    }
}

// ----------------------------------------------
// rendering

type DrawParticle2d<M> = (
    SetItemPipeline,
    SetParticleViewBindGroup<0, M>,
    SetParticle2dBindGroup<1, M>,
    DrawParticleInstanced<M>,
);

pub struct SetParticleViewBindGroup<const I: usize, M: Particle2dMaterial>(
    std::marker::PhantomData<M>,
);
impl<P: PhaseItem, M: Particle2dMaterial, const I: usize> RenderCommand<P>
    for SetParticleViewBindGroup<I, M>
{
    type Param = SRes<InstanceBuffer<M>>;
    type ViewQuery = Read<ViewUniformOffset>;
    type ItemQuery = ();

    fn render<'w>(
        _item: &P,
        view_uniform: &'_ ViewUniformOffset,
        _entity: Option<()>,
        particle_meta: SystemParamItem<'w, '_, Self::Param>,
        pass: &mut TrackedRenderPass<'w>,
    ) -> RenderCommandResult {
        if let Some(bind_group) = &particle_meta.into_inner().view_bind_group.as_ref() {
            pass.set_bind_group(I, bind_group, &[view_uniform.offset]);
            return RenderCommandResult::Success;
        }
        RenderCommandResult::Failure("failed to prep bind group")
    }
}

struct SetParticle2dBindGroup<const I: usize, M: Particle2dMaterial>(std::marker::PhantomData<M>);
impl<const I: usize, M: Particle2dMaterial, P: PhaseItem> RenderCommand<P>
    for SetParticle2dBindGroup<I, M>
{
    type Param = (
        SRes<RenderAssets<PreparedParticleMaterial<M>>>,
        SRes<RenderParticleMaterials<M>>,
    );
    type ViewQuery = ();
    type ItemQuery = ();

    #[inline]
    fn render<'w>(
        item: &P,
        _view: bevy_ecs::query::ROQueryItem<'w, '_, Self::ViewQuery>,
        _item_query: Option<bevy_ecs::query::ROQueryItem<'w, '_, Self::ItemQuery>>,
        params: SystemParamItem<'w, '_, Self::Param>,
        pass: &mut TrackedRenderPass<'w>,
    ) -> RenderCommandResult {
        let (prep_mats, prep_particles) = params;

        let Some(asset_id) = prep_particles.into_inner().get(&item.entity()) else {
            return RenderCommandResult::Failure("trying to render particle spawner without asset");
        };

        let Some(prepared_material) = prep_mats.into_inner().get(*asset_id) else {
            return RenderCommandResult::Failure(
                "trying to render particle spawner without preped material",
            );
        };

        pass.set_bind_group(I, &prepared_material.bind_group, &[]);
        RenderCommandResult::Success
    }
}

// ---------------------------
// #draw

struct DrawParticleInstanced<M: Particle2dMaterial>(std::marker::PhantomData<M>);
impl<P: PhaseItem, M: Particle2dMaterial> RenderCommand<P> for DrawParticleInstanced<M> {
    type Param = SRes<InstanceBuffer<M>>;
    type ViewQuery = ();
    type ItemQuery = Read<ParticleInstanceBatch>;

    #[inline]
    fn render<'w>(
        _item: &P,
        _view: (),
        instance_buffer: Option<&'w ParticleInstanceBatch>,
        meta: SystemParamItem<'w, '_, Self::Param>,
        pass: &mut TrackedRenderPass<'w>,
    ) -> RenderCommandResult {
        let Some(batch) = instance_buffer else {
            return RenderCommandResult::Failure("No batch buffer prepared");
        };

        let particle_meta = meta.into_inner();

        let Some(instance_buffer) = particle_meta.instance_buffer.buffer() else {
            return RenderCommandResult::Failure("Instance buffer was never written to GPU");
        };

        let Some(index_buffer) = particle_meta.index_buffer.buffer() else {
            return RenderCommandResult::Failure("Index buffer was never written to GPU");
        };

        pass.set_index_buffer(index_buffer.slice(..), IndexFormat::Uint32);
        pass.set_vertex_buffer(0, instance_buffer.slice(..));
        pass.draw_indexed(0..6, 0, batch.range.clone());

        RenderCommandResult::Success
    }
}
