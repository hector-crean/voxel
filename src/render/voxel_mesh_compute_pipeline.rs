use crate::{
    bundles::volumetric_bundle::Volumetric,
    data::{atomics::Atomics, voxel::Voxel},
    CHUNK_SZ, CHUNK_SZ_3,
};
use bevy::{
    ecs::{
        query::ROQueryItem,
        system::{lifetimeless::SRes, SystemParamItem},
        world::Command,
    },
    prelude::*,
    render::{
        extract_component::{ExtractComponent, ExtractComponentPlugin},
        render_graph::{self, NodeRunError, RenderGraph, RenderLabel},
        render_phase::{PhaseItem, RenderCommand, RenderCommandResult, TrackedRenderPass},
        render_resource::{binding_types::storage_buffer, *},
        renderer::{RenderContext, RenderDevice, RenderQueue},
        Extract, Render, RenderApp, RenderSet,
    },
    utils::{info, HashMap},
};
use crossbeam_channel::{Receiver, Sender};

use crate::{
    channels::RenderWorldSender,
    data::{
        gpu_voxel_material::GpuVoxelMaterial,
        gpu_voxel_material_bind_group::GpuVoxelMaterialBindGroups,
        voxel_material::VoxelMaterialComponents,
    },
};

const SHADER_ASSET_PATH: &str = "shaders/gpu_readback.wgsl";

#[derive(ShaderType, Clone)]
pub struct VoxelBuffer {
    #[size(runtime)]
    data: Vec<Voxel>,
}

#[derive(ShaderType, Clone)]
pub struct VertexBuffer {
    #[size(runtime)]
    data: Vec<Vec3>,
}

#[derive(ShaderType, Clone)]
pub struct NormalBuffer {
    #[size(runtime)]
    data: Vec<Vec3>,
}

#[derive(ShaderType, Clone)]
pub struct IndexBuffer {
    #[size(runtime)]
    data: Vec<UVec3>,
}

#[derive(ShaderType, Clone)]
pub struct UvBuffer {
    #[size(runtime)]
    data: Vec<Vec2>,
}

#[derive(ShaderType, Clone)]
pub struct EdgeTable {
    data: [u32; 256],
}

#[derive(ShaderType, Clone)]
pub struct TriangleTable {
    data: [[i32; 16]; 256],
}

#[derive(Resource)]
pub struct VoxelMeshComputePipeline {
    pub bind_group_1_layout: BindGroupLayout,
    pub pipeline: CachedComputePipelineId,
}

impl FromWorld for VoxelMeshComputePipeline {
    fn from_world(world: &mut World) -> Self {
        let render_device = world.resource::<RenderDevice>();

        let bind_group_1_layout = render_device.create_bind_group_layout(
            Some("VoxelMeshComputePipeline::bind_group_1_layout"),
            &BindGroupLayoutEntries::with_indices(
                ShaderStages::COMPUTE,
                (
                    (
                        0,
                        storage_buffer::<EdgeTable>(false).visibility(ShaderStages::COMPUTE),
                    ),
                    (
                        1,
                        storage_buffer::<TriangleTable>(false).visibility(ShaderStages::COMPUTE),
                    ),
                    (
                        2,
                        storage_buffer::<VoxelBuffer>(false).visibility(ShaderStages::COMPUTE),
                    ),
                    (
                        3,
                        storage_buffer::<Atomics>(false).visibility(ShaderStages::COMPUTE),
                    ),
                    (
                        4,
                        storage_buffer::<VertexBuffer>(false).visibility(ShaderStages::COMPUTE),
                    ),
                    (
                        5,
                        storage_buffer::<NormalBuffer>(false).visibility(ShaderStages::COMPUTE),
                    ),
                    (
                        6,
                        storage_buffer::<IndexBuffer>(false).visibility(ShaderStages::COMPUTE),
                    ),
                    (
                        7,
                        storage_buffer::<UvBuffer>(false).visibility(ShaderStages::COMPUTE),
                    ),
                ),
            ),
        );

        let shader = world.load_asset(SHADER_ASSET_PATH);

        let pipeline_cache = world.resource::<PipelineCache>();

        let pipeline = pipeline_cache.queue_compute_pipeline(ComputePipelineDescriptor {
            label: Some("VoxelMeshComputePipeline shader".into()),
            layout: vec![bind_group_1_layout.clone()],
            push_constant_ranges: Vec::new(),
            shader: shader.clone(),
            shader_defs: Vec::new(),
            entry_point: "main".into(),
        });

        VoxelMeshComputePipeline {
            bind_group_1_layout,
            pipeline,
        }
    }
}

#[derive(Debug, Hash, PartialEq, Eq, Clone, RenderLabel)]
pub struct VoxelMeshComputeNodeLabel;

pub struct VoxelMeshComputeNode {
    voxel_material_query: QueryState<Entity, With<Volumetric>>,
}

impl FromWorld for VoxelMeshComputeNode {
    fn from_world(world: &mut World) -> Self {
        Self {
            voxel_material_query: world.query_filtered(),
        }
    }
}

impl render_graph::Node for VoxelMeshComputeNode {
    fn update(&mut self, world: &mut World) {
        self.voxel_material_query.update_archetypes(world);
    }

    fn run(
        &self,
        _graph: &mut render_graph::RenderGraphContext,
        render_context: &mut RenderContext,
        world: &World,
    ) -> Result<(), render_graph::NodeRunError> {
        let pipeline_cache = world.resource::<PipelineCache>();
        let voxel_mesh_pipeline = world.resource::<VoxelMeshComputePipeline>();
        let gpu_voxel_materials = world.resource::<VoxelMaterialComponents<GpuVoxelMaterial>>();
        let voxel_bind_groups =
            world.resource::<VoxelMaterialComponents<GpuVoxelMaterialBindGroups>>();

        let pipeline = match pipeline_cache.get_compute_pipeline(voxel_mesh_pipeline.pipeline) {
            None => return Ok(()), // some pipelines are not loaded yet
            Some(pipeline) => pipeline,
        };

        let command_encoder = render_context.command_encoder();

        for voxel_material_entity in self.voxel_material_query.iter_manual(world) {
            let gpu_voxel_material = gpu_voxel_materials.get(&voxel_material_entity);
            let voxel_bind_groups = voxel_bind_groups.get(&voxel_material_entity);
            match (gpu_voxel_material, voxel_bind_groups) {
                (Some(gpu_voxel_material), Some(voxel_bind_group)) => {
                    let mut pass =
                        command_encoder.begin_compute_pass(&ComputePassDescriptor::default());

                    for (bind_group_id, bind_group) in voxel_bind_group.0.iter().enumerate() {
                        pass.set_bind_group(bind_group_id as u32, &bind_group, &[]);
                    }

                    pass.set_pipeline(pipeline);
                    pass.dispatch_workgroups(8, 8, 8);

                    drop(pass);

                    command_encoder.copy_buffer_to_buffer(
                        gpu_voxel_material
                            .vertices_buffer
                            .buffer()
                            .expect("Vertices Buffer should have already been uploaded to the gpu"),
                        0,
                        &gpu_voxel_material.vertices_staging_buffer,
                        0,
                        (CHUNK_SZ as usize * std::mem::size_of::<Vec3>()) as u64,
                    );
                }
                _ => {
                    info!(
                        "No gpu_voxel material or gpu_bind_group for {}",
                        voxel_material_entity
                    );
                }
            }
        }

        Ok(())
    }
}

pub struct SetGpuVoxelMaterialBindGroup<const I: usize>;

impl<const I: usize, P: PhaseItem> RenderCommand<P> for SetGpuVoxelMaterialBindGroup<I> {
    type Param = SRes<VoxelMaterialComponents<GpuVoxelMaterialBindGroups>>;
    type ViewQuery = ();
    type ItemQuery = ();

    #[inline]
    fn render<'w>(
        item: &P,
        _: ROQueryItem<'w, Self::ViewQuery>,
        _: Option<ROQueryItem<'w, Self::ItemQuery>>,
        gpu_voxel_bind_groups: SystemParamItem<'w, '_, Self::Param>,
        pass: &mut TrackedRenderPass<'w>,
    ) -> RenderCommandResult {
        let data = gpu_voxel_bind_groups
            .into_inner()
            .get(&item.entity())
            .unwrap();

        pass.set_bind_group(I, &data.0[0], &[]);
        RenderCommandResult::Success
    }
}
