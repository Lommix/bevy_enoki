use crate::RenderParticleTag;

use super::{update::Particle, ParticleSpawner, ParticleStore};
use bevy::{
    core_pipeline::core_2d::{Transparent2d, CORE_2D_DEPTH_FORMAT},
    ecs::{
        entity::EntityHashMap,
        system::{
            lifetimeless::{Read, SRes},
            SystemParamItem,
        },
    },
    math::FloatOrd,
    prelude::*,
    render::{
        mesh::PrimitiveTopology,
        render_asset::{PrepareAssetError, RenderAsset, RenderAssetPlugin, RenderAssets},
        render_phase::{
            AddRenderCommand, DrawFunctions, PhaseItem, PhaseItemExtraIndex, RenderCommand,
            RenderCommandResult, SetItemPipeline, TrackedRenderPass, ViewSortedRenderPhases,
        },
        render_resource::{
            binding_types::uniform_buffer, AsBindGroup, AsBindGroupError, BindGroup,
            BindGroupEntries, BindGroupLayout, BindGroupLayoutEntries, BlendState, BufferUsages,
            BufferVec, ColorTargetState, ColorWrites, CompareFunction, DepthBiasState,
            DepthStencilState, FrontFace, IndexFormat, OwnedBindingResource, PipelineCache,
            PolygonMode, PrimitiveState, RenderPipelineDescriptor, ShaderRef, ShaderStages,
            ShaderType, SpecializedRenderPipeline, SpecializedRenderPipelines, StencilFaceState,
            StencilState, TextureFormat, VertexAttribute, VertexBufferLayout, VertexFormat,
            VertexStepMode,
        },
        renderer::{RenderDevice, RenderQueue},
        sync_world::RenderEntity,
        view::{
            ExtractedView, RenderVisibleEntities, ViewTarget, ViewUniform, ViewUniformOffset,
            ViewUniforms,
        },
        Extract, Render, RenderApp, RenderSet,
    },
    sprite::Mesh2dPipelineKey,
    tasks::{ComputeTaskPool, ParallelSlice},
};
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
                    queue_particles::<M>.in_set(RenderSet::Queue),
                    prepare_particles_instance_buffers::<M>.in_set(RenderSet::PrepareResources),
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
    particles: EntityHashMap<Vec<InstanceData>>,
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
    mut events: Extract<EventReader<AssetEvent<M>>>,
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
    query.iter().for_each(|emitter| {
        let (particle_store, global, material_handle, visbility, render_entity) = emitter;
        if !visbility.get() || particle_store.is_empty() {
            return;
        }

        cmd.entity(**render_entity)
            .insert((ZOrder(FloatOrd(global.translation().z)), ParticleTag));
        let particles = particle_store
            .par_splat_map(ComputeTaskPool::get(), None, |_, particles| {
                particles.iter().map(InstanceData::from).collect::<Vec<_>>()
            })
            .into_iter()
            .flatten()
            .collect();
        render_material_instances.insert(**render_entity, material_handle.id());
        extraced_batches
            .particles
            .insert(**render_entity, particles);
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
            | Mesh2dPipelineKey::from_hdr(view.hdr);

        let key = Particle2dPipelineKey { mesh_key };
        let pipeline = pipelines.specialize(&pipeline_cache, &custom_pipeline, key);

        for (entity, main_entity) in visible_entities.get::<RenderParticleTag>().iter() {
            if extract_particles.particles.get(entity).is_none() {
                continue;
            }

            let Ok(order) = z_orders.get(*entity) else {
                return;
            };

            transparent_phase.add(Transparent2d {
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
    transform: [Vec4; 3],
    color: [f32; 4],
    custom: Vec4,
}

impl From<&Particle> for InstanceData {
    #[inline(always)]
    fn from(value: &Particle) -> Self {
        let transpose_model_3x3 = value.transform.compute_affine().matrix3.transpose();
        Self {
            transform: [
                transpose_model_3x3
                    .x_axis
                    .extend(value.transform.translation.x),
                transpose_model_3x3
                    .y_axis
                    .extend(value.transform.translation.y),
                transpose_model_3x3
                    .z_axis
                    .extend(value.transform.translation.z),
            ],
            color: value.color.to_f32_array(),
            custom: Vec4::new(value.duration_fraction, value.duration, 0., 0.),
        }
    }
}

#[derive(Component, Deref)]
pub struct InstanceMaterialData(Vec<InstanceData>);

#[derive(Resource)]
pub struct PreparedParticleMaterial<M: Particle2dMaterial> {
    pub bind_group: BindGroup,
    pub _bindings: Vec<(u32, OwnedBindingResource)>,
    pub _key: M::Data,
}

impl<M: Particle2dMaterial> RenderAsset for PreparedParticleMaterial<M> {
    type SourceAsset = M;
    type Param = (SRes<RenderDevice>, SRes<Particle2dPipeline<M>>, M::Param);

    fn prepare_asset(
        material: Self::SourceAsset,
        _: AssetId<Self::SourceAsset>,
        (render_device, pipeline, param): &mut SystemParamItem<Self::Param>,
    ) -> Result<Self, bevy::render::render_asset::PrepareAssetError<Self::SourceAsset>> {
        match material.as_bind_group(&pipeline.uniform_layout, render_device, param) {
            Ok(prepared) => Ok(PreparedParticleMaterial {
                bind_group: prepared.bind_group,
                _bindings: prepared.bindings.0,
                _key: prepared.data,
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

fn prepare_particles_instance_buffers<M: Particle2dMaterial>(
    mut cmd: Commands,
    mut extracted_spawner: ResMut<ExtracedParticleSpawner<M>>,
    render_device: Res<RenderDevice>,
    render_queue: Res<RenderQueue>,
    view_uniforms: Res<ViewUniforms>,
    particle_pipeline: Res<Particle2dPipeline<M>>,
    mut particle_buffer: ResMut<InstanceBuffer<M>>,
) {
    if let Some(view_binding) = view_uniforms.uniforms.binding() {
        particle_buffer.view_bind_group = Some(render_device.create_bind_group(
            "particle_view_bind_group",
            &particle_pipeline.view_layout,
            &BindGroupEntries::single(view_binding),
        ));
    }

    particle_buffer.instance_buffer.clear();
    let mut index = 0;

    for (entity, instances) in extracted_spawner.particles.iter_mut() {
        if instances.is_empty() {
            continue;
        }

        let batch = ParticleInstanceBatch {
            range: index..index + instances.len() as u32,
        };

        index += instances.len() as u32;
        instances.drain(..).for_each(|i| {
            particle_buffer.instance_buffer.push(i);
        });

        cmd.entity(*entity).insert(batch);
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
    uniform_layout: BindGroupLayout,
    view_layout: BindGroupLayout,
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

        let view_layout = render_device.create_bind_group_layout(
            "particle_view_layout",
            &BindGroupLayoutEntries::single(
                ShaderStages::VERTEX_FRAGMENT,
                uniform_buffer::<ViewUniform>(true),
            ),
        );

        Particle2dPipeline {
            view_layout,
            uniform_layout: M::bind_group_layout(render_device), //world.resource::<ParticleUniformLayout>().0.clone(),
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

        let format = if key.mesh_key.contains(Mesh2dPipelineKey::HDR) {
            ViewTarget::TEXTURE_FORMAT_HDR
        } else {
            TextureFormat::bevy_default()
        };

        RenderPipelineDescriptor {
            zero_initialize_workgroup_memory: true,
            vertex: bevy::render::render_resource::VertexState {
                shader: self.vertex_shader.clone(),
                shader_defs: vec![],
                entry_point: "vertex".into(),
                buffers: vec![VertexBufferLayout {
                    array_stride: 80,
                    step_mode: VertexStepMode::Instance,
                    attributes: vec![
                        // translation
                        VertexAttribute {
                            format: VertexFormat::Float32x4,
                            offset: 0,
                            shader_location: 0,
                        },
                        // rotation
                        VertexAttribute {
                            format: VertexFormat::Float32x4,
                            offset: 16,
                            shader_location: 1,
                        },
                        // scale
                        VertexAttribute {
                            format: VertexFormat::Float32x4,
                            offset: 32,
                            shader_location: 2,
                        },
                        // color
                        VertexAttribute {
                            format: VertexFormat::Float32x4,
                            offset: 48,
                            shader_location: 3,
                        },
                        // custom
                        VertexAttribute {
                            format: VertexFormat::Float32x4,
                            offset: 64,
                            shader_location: 4,
                        },
                    ],
                }],
            },
            fragment: Some(bevy::render::render_resource::FragmentState {
                shader: self.fragment_shader.clone(),
                shader_defs: vec![],
                entry_point: "fragment".into(),
                targets: vec![Some(ColorTargetState {
                    format,
                    blend: Some(BlendState::ALPHA_BLENDING),
                    write_mask: ColorWrites::ALL,
                })],
            }),
            label: Some("particle 2d pipeline".into()),
            layout,
            push_constant_ranges: vec![],
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
                depth_write_enabled: false,
                depth_compare: CompareFunction::GreaterEqual,
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
            multisample: bevy::render::render_resource::MultisampleState {
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
        _view: bevy::ecs::query::ROQueryItem<'w, Self::ViewQuery>,
        _item_query: Option<bevy::ecs::query::ROQueryItem<'w, Self::ItemQuery>>,
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

        pass.set_index_buffer(index_buffer.slice(..), 0, IndexFormat::Uint32);
        pass.set_vertex_buffer(0, instance_buffer.slice(..));
        pass.draw_indexed(0..6, 0, batch.range.clone());

        RenderCommandResult::Success
    }
}
