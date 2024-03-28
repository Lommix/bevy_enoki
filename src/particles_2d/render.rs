use super::{update::Particle, ParticleStore};
use bevy::{
    core_pipeline::core_2d::Transparent2d,
    ecs::{
        entity::EntityHashMap,
        system::{
            lifetimeless::{Read, SRes},
            SystemParamItem,
        },
    },
    prelude::*,
    render::{
        mesh::{GpuBufferInfo, MeshVertexBufferLayout},
        render_asset::RenderAssets,
        render_phase::{
            AddRenderCommand, DrawFunctions, PhaseItem, RenderCommand, RenderCommandResult,
            RenderPhase, SetItemPipeline, TrackedRenderPass,
        },
        render_resource::{
            AsBindGroup, BindGroup, BindGroupLayout, Buffer, BufferInitDescriptor, BufferUsages,
            OwnedBindingResource, PipelineCache, RenderPipelineDescriptor, ShaderRef,
            SpecializedMeshPipeline, SpecializedMeshPipelineError, SpecializedMeshPipelines,
            VertexAttribute, VertexBufferLayout, VertexFormat, VertexStepMode,
        },
        renderer::RenderDevice,
        texture::FallbackImage,
        view::{ExtractedView, VisibleEntities},
        Extract, Render, RenderApp, RenderSet,
    },
    sprite::{
        Mesh2dPipeline, Mesh2dPipelineKey, RenderMesh2dInstances, SetMesh2dBindGroup,
        SetMesh2dViewBindGroup,
    },
    utils::{FloatOrd, HashMap, HashSet},
};
use bytemuck::{Pod, Zeroable};
use std::hash::Hash;

/// Particle Material Trait
/// bin custom fragment shader to material
pub trait Particle2dMaterial: AsBindGroup + Asset + Clone + Sized {
    fn fragment_shader() -> ShaderRef {
        super::PARTICLE_DEFAULT_FRAG.into()
    }
}

pub struct Particle2dMaterialPlugin<M: Particle2dMaterial> {
    _m: std::marker::PhantomData<M>,
}

impl<M: Particle2dMaterial> Default for Particle2dMaterialPlugin<M> {
    fn default() -> Self {
        Self {
            _m: std::marker::PhantomData::<M>::default(),
        }
    }
}

impl<M: Particle2dMaterial> Plugin for Particle2dMaterialPlugin<M> {
    fn build(&self, app: &mut App) {
        app.init_asset::<M>();
        app.sub_app_mut(RenderApp)
            .add_render_command::<Transparent2d, DrawParticle2d<M>>()
            .init_resource::<SpecializedMeshPipelines<Particle2dPipeline<M>>>()
            .init_resource::<ExtractedParticleBatches<M>>()
            .init_resource::<ExtractedParticleMaterials<M>>()
            .init_resource::<PreparedParticleMaterials<M>>()
            .init_resource::<RenderParticleMaterials<M>>()
            .add_systems(
                ExtractSchedule,
                (extract_particles::<M>, extract_materials::<M>),
            )
            .add_systems(
                Render,
                (
                    queue_particles::<M>.in_set(RenderSet::QueueMeshes),
                    prepare_particle_materials::<M>.in_set(RenderSet::PrepareBindGroups),
                    prepare_particles_instance_buffers::<M>.in_set(RenderSet::PrepareResources),
                ),
            );
    }

    fn finish(&self, app: &mut App) {
        let render_app = app.sub_app_mut(RenderApp);

        render_app.init_resource::<Particle2dPipeline<M>>();
    }
}

// ----------------------------------------------
// extract
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

#[derive(Resource)]
pub struct ExtractedParticleBatches<M: Particle2dMaterial> {
    particles: EntityHashMap<Vec<InstanceData>>,
    _m: std::marker::PhantomData<M>,
}
impl<M: Particle2dMaterial> Default for ExtractedParticleBatches<M> {
    fn default() -> Self {
        Self {
            particles: Default::default(),
            _m: Default::default(),
        }
    }
}

fn extract_materials<M: Particle2dMaterial>(
    mut cmd: Commands,
    mut events: Extract<EventReader<AssetEvent<M>>>,
    assets: Extract<Res<Assets<M>>>,
) {
    let mut changed_assets = HashSet::default();
    for event in events.read() {
        match event {
            AssetEvent::Added { id } | AssetEvent::Modified { id } => {
                changed_assets.insert(*id);
            }
            AssetEvent::Removed { id } => {
                changed_assets.remove(id);
            }
            AssetEvent::Unused { .. } => {}
            AssetEvent::LoadedWithDependencies { .. } => {}
        }

        let mut extracted_assets = Vec::new();
        for id in changed_assets.drain() {
            if let Some(asset) = assets.get(id) {
                extracted_assets.push((id, asset.clone()));
            }
        }

        cmd.insert_resource(ExtractedParticleMaterials {
            materials: extracted_assets,
        });
    }
}

fn extract_particles<M: Particle2dMaterial>(
    mut cmd: Commands,
    mut extraced_batches: ResMut<ExtractedParticleBatches<M>>,
    mut render_material_instances: ResMut<RenderParticleMaterials<M>>,
    query: Extract<
        Query<(
            Entity,
            &ParticleStore,
            &GlobalTransform,
            &Handle<M>,
            &ViewVisibility,
        )>,
    >,
) {
    extraced_batches.particles.clear();
    query.iter().for_each(|emitter| {
        let (entity, particle_store, global, material_handle, visbility) = emitter;
        if !visbility.get() {
            return;
        }

        let particles = particle_store
            .iter()
            .map(|particle| InstanceData::from(particle))
            .collect::<Vec<_>>();

        extraced_batches.particles.insert(entity, particles);
        render_material_instances.insert(entity, material_handle.id());

        cmd.get_or_spawn(entity)
            .insert(ZOrder(FloatOrd(global.translation().z)));
    });
}

#[derive(Component, Deref)]
pub struct ZOrder(FloatOrd);

// ----------------------------------------------
// queue

fn queue_particles<M: Particle2dMaterial>(
    transparent_2d_draw_functions: Res<DrawFunctions<Transparent2d>>,
    custom_pipeline: Res<Particle2dPipeline<M>>,
    msaa: Res<Msaa>,
    mut pipelines: ResMut<SpecializedMeshPipelines<Particle2dPipeline<M>>>,
    pipeline_cache: Res<PipelineCache>,
    meshes: Res<RenderAssets<Mesh>>,
    render_mesh_instances: Res<RenderMesh2dInstances>,
    extract_particles: Res<ExtractedParticleBatches<M>>,
    z_orders: Query<&ZOrder>,

    mut views: Query<(
        &ExtractedView,
        &VisibleEntities,
        &mut RenderPhase<Transparent2d>,
    )>,
) {
    let draw_particles = transparent_2d_draw_functions
        .read()
        .id::<DrawParticle2d<M>>();
    let msaa_key = Mesh2dPipelineKey::from_msaa_samples(msaa.samples());

    for (view, visible_entities, mut transparent_phase) in &mut views {
        let view_key = msaa_key | Mesh2dPipelineKey::from_hdr(view.hdr);

        for (entity, _) in extract_particles.particles.iter() {
            if !visible_entities.entities.contains(&entity) {
                continue;
            }

            let Some(mesh_instance) = render_mesh_instances.get(entity) else {
                continue;
            };

            let Some(mesh) = meshes.get(mesh_instance.mesh_asset_id) else {
                continue;
            };

            let mesh_key =
                view_key | Mesh2dPipelineKey::from_primitive_topology(mesh.primitive_topology);

            let key = Particle2dPipelineKey { mesh_key };

            let pipeline = pipelines
                .specialize(&pipeline_cache, &custom_pipeline, key, &mesh.layout)
                .unwrap();

            let order = z_orders.get(*entity).unwrap();

            transparent_phase.add(Transparent2d {
                sort_key: **order,
                entity: *entity,
                pipeline,
                draw_function: draw_particles,
                batch_range: 0..1,
                dynamic_offset: None,
            });
        }
    }
}

// ----------------------------------------------
//

#[derive(Clone, Copy, Pod, Zeroable)]
#[repr(C)]
pub struct InstanceData {
    transform: [Vec4; 3],
    color: [f32; 4],
    lifetime: f32,
    frame: u32,
    _p1: f32,
    _p2: f32,
    // frame: u32,
}
impl From<&Particle> for InstanceData {
    fn from(value: &Particle) -> Self {
        let transpose_model_3x3 = value.transform.compute_affine().matrix3;

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
            color: value.color.as_rgba_f32(),
            frame: value.frame,
            lifetime: value.lifetime.fraction(),
            _p1: 0.,
            _p2: 0.,
        }
    }
}

#[derive(Component, Deref)]
pub struct InstanceMaterialData(Vec<InstanceData>);

#[derive(Component)]
struct InstanceBuffer {
    buffer: Buffer,
    length: usize,
}

#[derive(Resource)]
pub struct PreparedParticleMaterial<M: Particle2dMaterial> {
    pub bindings: Vec<(u32, OwnedBindingResource)>,
    pub bind_group: BindGroup,
    pub key: M::Data,
}

#[derive(Resource, Deref, DerefMut)]
pub struct PreparedParticleMaterials<M: Particle2dMaterial>(
    HashMap<AssetId<M>, PreparedParticleMaterial<M>>,
);

#[derive(Deref, DerefMut)]
pub struct PrepareNextFrameParticleMaterials<M: Particle2dMaterial>(Vec<(AssetId<M>, M)>);

impl<M: Particle2dMaterial> Default for PrepareNextFrameParticleMaterials<M> {
    fn default() -> Self {
        Self(Vec::new())
    }
}

impl<M: Particle2dMaterial> Default for PreparedParticleMaterials<M> {
    fn default() -> Self {
        Self(HashMap::new())
    }
}

fn prepare_particle_materials<M: Particle2dMaterial>(
    mut extraced_materials: ResMut<ExtractedParticleMaterials<M>>,
    mut render_particle_materials: ResMut<PreparedParticleMaterials<M>>,
    mut prepare_next_frame: Local<PrepareNextFrameParticleMaterials<M>>,

    render_device: Res<RenderDevice>,
    images: Res<RenderAssets<Image>>,
    fallback_image: Res<FallbackImage>,
    pipeline: Res<Particle2dPipeline<M>>,
) {
    for (asset_id, material) in std::mem::take(&mut prepare_next_frame.0) {
        match material.as_bind_group(
            &pipeline.uniform_layout,
            &render_device,
            &images,
            &fallback_image,
        ) {
            Ok(prepared) => {
                render_particle_materials.insert(
                    asset_id,
                    PreparedParticleMaterial {
                        bindings: prepared.bindings,
                        bind_group: prepared.bind_group,
                        key: prepared.data,
                    },
                );
            }
            Err(_) => {
                prepare_next_frame.push((asset_id, material));
            }
        }
    }

    for (asset_id, material) in std::mem::take(&mut extraced_materials.materials) {
        match material.as_bind_group(
            &pipeline.uniform_layout,
            &render_device,
            &images,
            &fallback_image,
        ) {
            Ok(prepared) => {
                render_particle_materials.insert(
                    asset_id,
                    PreparedParticleMaterial {
                        bindings: prepared.bindings,
                        bind_group: prepared.bind_group,
                        key: prepared.data,
                    },
                );
            }
            Err(_) => {
                prepare_next_frame.push((asset_id, material));
            }
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

fn prepare_particles_instance_buffers<M: Particle2dMaterial>(
    mut cmd: Commands,
    extraced_batches: Res<ExtractedParticleBatches<M>>,
    render_device: Res<RenderDevice>,
) {
    for (entity, particle_instances) in extraced_batches.particles.iter() {
        let buffer = render_device.create_buffer_with_data(&BufferInitDescriptor {
            label: Some("instance data buffer"),
            contents: bytemuck::cast_slice(particle_instances.as_slice()),
            usage: BufferUsages::VERTEX | BufferUsages::COPY_DST,
        });

        cmd.entity(*entity).insert(InstanceBuffer {
            buffer,
            length: particle_instances.len(),
        });
    }
}

// ----------------------------------------------
// pipeline

#[derive(Resource)]
struct Particle2dPipeline<M: Particle2dMaterial> {
    vertex_shader: Handle<Shader>,
    fragment_shader: Handle<Shader>,
    mesh_pipeline: Mesh2dPipeline,
    uniform_layout: BindGroupLayout,
    _m: std::marker::PhantomData<M>,
}

#[derive(PartialEq, Eq, Hash, Clone)]
struct Particle2dPipelineKey {
    mesh_key: Mesh2dPipelineKey,
}

impl<M: Particle2dMaterial> FromWorld for Particle2dPipeline<M> {
    fn from_world(world: &mut World) -> Self {
        let server = world.resource::<AssetServer>();

        let fragment_shader = match M::fragment_shader() {
            ShaderRef::Default => super::PARTICLE_DEFAULT_FRAG,
            ShaderRef::Handle(handle) => handle,
            ShaderRef::Path(path) => server.load(path),
        };

        let vertex_shader = super::PARTICLE_VERTEX;
        let render_device = world.resource::<RenderDevice>();

        Particle2dPipeline {
            mesh_pipeline: world.resource::<Mesh2dPipeline>().clone(),
            uniform_layout: M::bind_group_layout(render_device), //world.resource::<ParticleUniformLayout>().0.clone(),
            vertex_shader,
            fragment_shader,
            _m: std::marker::PhantomData::<M>::default(),
        }
    }
}

impl<M: Particle2dMaterial> SpecializedMeshPipeline for Particle2dPipeline<M> {
    type Key = Particle2dPipelineKey;

    fn specialize(
        &self,
        key: Self::Key,
        layout: &MeshVertexBufferLayout,
    ) -> Result<RenderPipelineDescriptor, SpecializedMeshPipelineError> {
        let mut descriptor = self.mesh_pipeline.specialize(key.mesh_key, layout)?;

        descriptor.layout.push(self.uniform_layout.clone());

        let mut attributes = Vec::new();
        let mut offset = 0;

        // translation
        attributes.push(VertexAttribute {
            format: VertexFormat::Float32x4,
            offset,
            shader_location: 3,
        });
        offset += VertexFormat::Float32x4.size();

        // rotation
        attributes.push(VertexAttribute {
            format: VertexFormat::Float32x4,
            offset,
            shader_location: 4,
        });
        offset += VertexFormat::Float32x4.size();

        // scale
        attributes.push(VertexAttribute {
            format: VertexFormat::Float32x4,
            offset,
            shader_location: 5,
        });
        offset += VertexFormat::Float32x4.size();

        // color
        attributes.push(VertexAttribute {
            format: VertexFormat::Float32x4,
            offset,
            shader_location: 6,
        });
        offset += VertexFormat::Float32x4.size();

        // lifetime
        attributes.push(VertexAttribute {
            format: VertexFormat::Float32,
            offset,
            shader_location: 7,
        });
        offset += VertexFormat::Float32.size();

        // frame
        attributes.push(VertexAttribute {
            format: VertexFormat::Uint32,
            offset,
            shader_location: 8,
        });
        offset += VertexFormat::Float32.size();

        // p1
        attributes.push(VertexAttribute {
            format: VertexFormat::Float32,
            offset,
            shader_location: 9,
        });
        offset += VertexFormat::Float32.size();

        // p2
        attributes.push(VertexAttribute {
            format: VertexFormat::Float32,
            offset,
            shader_location: 10,
        });
        offset += VertexFormat::Float32.size();

        descriptor.vertex.shader = self.vertex_shader.clone();
        descriptor.vertex.buffers.push(VertexBufferLayout {
            array_stride: std::mem::size_of::<InstanceData>() as u64,
            step_mode: VertexStepMode::Instance,
            attributes,
        });
        descriptor.fragment.as_mut().unwrap().shader = self.fragment_shader.clone();
        Ok(descriptor)
    }
}

// ----------------------------------------------
// rendering

type DrawParticle2d<M> = (
    SetItemPipeline,
    SetMesh2dViewBindGroup<0>,
    SetMesh2dBindGroup<1>,
    SetParticle2dBindGroup<2, M>,
    DrawParticleInstanced,
);

struct SetParticle2dBindGroup<const I: usize, M: Particle2dMaterial> {
    _m: std::marker::PhantomData<M>,
}

impl<const I: usize, M: Particle2dMaterial, P: PhaseItem> RenderCommand<P>
    for SetParticle2dBindGroup<I, M>
{
    type Param = (
        SRes<PreparedParticleMaterials<M>>,
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
            return RenderCommandResult::Failure;
        };

        let Some(prepared_material) = prep_mats.into_inner().get(asset_id) else {
            return RenderCommandResult::Failure;
        };

        pass.set_bind_group(I, &prepared_material.bind_group, &[]);

        RenderCommandResult::Success
    }
}

struct DrawParticleInstanced;
impl<P: PhaseItem> RenderCommand<P> for DrawParticleInstanced {
    type Param = (SRes<RenderAssets<Mesh>>, SRes<RenderMesh2dInstances>);
    type ViewQuery = ();
    type ItemQuery = Read<InstanceBuffer>;

    #[inline]
    fn render<'w>(
        item: &P,
        _view: (),
        instance_buffer: Option<&'w InstanceBuffer>,
        (meshes, render_mesh_instances): SystemParamItem<'w, '_, Self::Param>,
        pass: &mut TrackedRenderPass<'w>,
    ) -> RenderCommandResult {
        let Some(instance_buffer) = instance_buffer else {
            return RenderCommandResult::Failure;
        };

        let Some(mesh_instance) = render_mesh_instances.get(&item.entity()) else {
            return RenderCommandResult::Failure;
        };

        let Some(gpu_mesh) = meshes.into_inner().get(mesh_instance.mesh_asset_id) else {
            return RenderCommandResult::Failure;
        };

        pass.set_vertex_buffer(0, gpu_mesh.vertex_buffer.slice(..));
        pass.set_vertex_buffer(1, instance_buffer.buffer.slice(..));

        match &gpu_mesh.buffer_info {
            GpuBufferInfo::Indexed {
                buffer,
                index_format,
                count,
            } => {
                pass.set_index_buffer(buffer.slice(..), 0, *index_format);
                pass.draw_indexed(0..*count, 0, 0..instance_buffer.length as u32);
            }
            GpuBufferInfo::NonIndexed => {
                pass.draw(0..gpu_mesh.vertex_count, 0..instance_buffer.length as u32);
            }
        }
        RenderCommandResult::Success
    }
}
