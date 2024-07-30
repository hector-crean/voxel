use bevy::{
    ecs::{
        query::ROQueryItem,
        system::{lifetimeless::SRes, SystemParamItem},
    },
    prelude::*,
    render::{
        render_graph::{self, RenderLabel},
        render_phase::{PhaseItem, RenderCommand, RenderCommandResult, TrackedRenderPass},
        render_resource::{binding_types::storage_buffer, *},
        renderer::{RenderContext, RenderDevice},
        Extract, Render, RenderApp, RenderSet,
    },
    utils::{info, HashMap},
};

use crate::{
    bundles::volumetric_bundle::Volumetric,
    render::voxel_mesh_compute_pipeline::VoxelMeshComputePipeline,
};

use super::{
    gpu_voxel_material::GpuVoxelMaterial,
    voxel::Voxel,
    voxel_material::{VoxelMaterial, VoxelMaterialComponents},
};

pub struct GpuVoxelMaterialBindGroups(pub [BindGroup; 1]);

impl GpuVoxelMaterialBindGroups {
    pub fn new(
        render_device: &RenderDevice,
        voxel_pipeline: &VoxelMeshComputePipeline,
        GpuVoxelMaterial {
            voxels_buffer,
            edge_table_buffer,
            tri_table_buffer,
            vertices_buffer,
            normals_buffer,
            uvs_buffer,
            indices_buffer,
            atomics_buffer,
            ..
        }: &GpuVoxelMaterial,
    ) -> Self {
        let bind_group_1 = render_device.create_bind_group(
            None,
            &voxel_pipeline.bind_group_1_layout,
            &BindGroupEntries::with_indices((
                (
                    0,
                    edge_table_buffer
                        .binding()
                        .expect("Edge Table Buffer should have already been uploaded to the gpu"),
                ),
                (
                    1,
                    tri_table_buffer
                        .binding()
                        .expect("Tri Table Buffer should have already been uploaded to the gpu"),
                ),
                (
                    2,
                    voxels_buffer
                        .binding()
                        .expect("Voxels Buffer should have already been uploaded to the gpu"),
                ),
                (
                    3,
                    atomics_buffer
                        .binding()
                        .expect("Atomics Buffer should have already been uploaded to the gpu"),
                ),
                (
                    4,
                    vertices_buffer
                        .binding()
                        .expect("Vertices Buffer should have already been uploaded to the gpu"),
                ),
                (
                    5,
                    normals_buffer
                        .binding()
                        .expect("Normals Buffer should have already been uploaded to the gpu"),
                ),
                (
                    6,
                    indices_buffer
                        .binding()
                        .expect("Indices Buffer should have already been uploaded to the gpu"),
                ),
                (
                    7,
                    uvs_buffer
                        .binding()
                        .expect("UVs Buffer should have already been uploaded to the gpu"),
                ),
            )),
        );

        GpuVoxelMaterialBindGroups([bind_group_1])
    }
    /// Initializes the [`GpuVoxelMaterialBindGroup`] of newly created VoxelMaterial.
    pub fn initialise(
        render_device: Res<RenderDevice>,
        mut voxel_material_bind_groups: ResMut<VoxelMaterialComponents<GpuVoxelMaterialBindGroups>>,
        gpu_voxel_materials: ResMut<VoxelMaterialComponents<GpuVoxelMaterial>>,
        voxel_pipeline: Res<VoxelMeshComputePipeline>,
        volumetric_query: Extract<Query<Entity, (With<VoxelMaterial>, Added<Volumetric>)>>,
    ) -> () {
        let pipeline = voxel_pipeline.as_ref();

        for (entity) in volumetric_query.iter() {
            if let Some(gpu_voxel_material) = gpu_voxel_materials.get(&entity) {
                let voxel_bind_groups = GpuVoxelMaterialBindGroups::new(
                    render_device.as_ref(),
                    pipeline,
                    &gpu_voxel_material,
                );

                voxel_material_bind_groups.insert(entity, voxel_bind_groups);
            }
        }
    }
    pub fn prepare(
        render_device: Res<RenderDevice>,
        mut voxel_material_bind_groups: ResMut<VoxelMaterialComponents<GpuVoxelMaterialBindGroups>>,
        gpu_voxel_materials: ResMut<VoxelMaterialComponents<GpuVoxelMaterial>>,
        voxel_pipeline: Res<VoxelMeshComputePipeline>,
        volumetric_query: Query<Entity, (With<VoxelMaterial>, With<Volumetric>)>,
    ) {
        let pipeline = voxel_pipeline.as_ref();

        for (entity) in volumetric_query.iter() {
            if let Some(gpu_voxel_material) = gpu_voxel_materials.get(&entity) {
                let voxel_bind_groups = GpuVoxelMaterialBindGroups::new(
                    render_device.as_ref(),
                    pipeline,
                    &gpu_voxel_material,
                );

                voxel_material_bind_groups.insert(entity, voxel_bind_groups);
            }
        }
    }
}
