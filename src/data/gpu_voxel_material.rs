use std::num::NonZeroU64;

use bevy::{
    prelude::*,
    render::{
        render_resource::{Buffer, BufferDescriptor, BufferUsages, BufferVec, ShaderType},
        renderer::{RenderDevice, RenderQueue},
        Extract,
    },
};

use crate::{
    bundles::volumetric_bundle::Volumetric, render::voxel_mesh_compute_pipeline::VertexBuffer,
};

use super::{
    edge_table::EDGE_TABLE,
    triangle_table::TRI_TABLE,
    voxel::Voxel,
    voxel_material::{VoxelMaterial, VoxelMaterialComponents},
};

#[derive(Component)]
pub struct GpuVoxelMaterial {
    pub voxels_buffer: BufferVec<Voxel>,
    pub edge_table_buffer: BufferVec<u32>,
    pub tri_table_buffer: BufferVec<[i32; 16]>,
    pub vertices_buffer: BufferVec<Vec4>,
    pub normals_buffer: BufferVec<Vec4>,
    pub uvs_buffer: BufferVec<Vec2>,
    pub indices_buffer: BufferVec<u32>,
    pub atomics_buffer: BufferVec<u32>,

    pub vertices_staging_buffer: Buffer,
}

impl GpuVoxelMaterial {
    pub fn new(
        render_device: &RenderDevice,
        render_queue: &RenderQueue,
        voxel_material: &VoxelMaterial,
    ) -> Self {
        let mut voxels_buffer =
            BufferVec::<Voxel>::new(BufferUsages::STORAGE | BufferUsages::COPY_SRC);

        voxels_buffer.reserve(voxel_material.chunk_size as usize, render_device);

        for voxel in &voxel_material.voxels {
            voxels_buffer.push(*voxel);
        }
        voxels_buffer.write_buffer(render_device, render_queue);

        let mut edge_table_buffer =
            BufferVec::<u32>::new(BufferUsages::STORAGE | BufferUsages::COPY_SRC);
        edge_table_buffer.reserve(256, render_device);

        for edge in EDGE_TABLE {
            edge_table_buffer.push(edge);
        }
        edge_table_buffer.write_buffer(render_device, render_queue);

        let mut tri_table_buffer =
            BufferVec::<[i32; 16]>::new(BufferUsages::STORAGE | BufferUsages::COPY_SRC);
        tri_table_buffer.reserve(256, render_device);

        for tri in TRI_TABLE {
            tri_table_buffer.push(tri);
        }
        tri_table_buffer.write_buffer(render_device, render_queue);

        let mut vertices_buffer = BufferVec::<Vec4>::new(
            BufferUsages::STORAGE | BufferUsages::COPY_SRC | BufferUsages::COPY_DST,
        );
        vertices_buffer.reserve((voxel_material.chunk_size as usize), render_device);

        let vertices_staging_buffer = render_device.create_buffer(&BufferDescriptor {
            label: Some("vertices_staging_buffer"),
            size: (VertexBuffer::min_size()
                .checked_mul(NonZeroU64::new_unchecked(voxel_material.chunk_size))),
            usage: BufferUsages::MAP_READ | BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });

        let mut uvs_buffer = BufferVec::<Vec2>::new(
            BufferUsages::STORAGE | BufferUsages::COPY_SRC | BufferUsages::COPY_DST,
        );
        uvs_buffer.reserve((voxel_material.chunk_size as usize) * 4 * 6, render_device);

        let mut normals_buffer = BufferVec::<Vec4>::new(
            BufferUsages::STORAGE | BufferUsages::COPY_SRC | BufferUsages::COPY_DST,
        );
        normals_buffer.reserve((voxel_material.chunk_size as usize) * 4 * 6, render_device);

        let mut indices_buffer = BufferVec::<u32>::new(
            BufferUsages::STORAGE | BufferUsages::COPY_SRC | BufferUsages::COPY_DST,
        );
        indices_buffer.reserve((voxel_material.chunk_size as usize) * 6 * 6, render_device);

        let mut atomics_buffer = BufferVec::<u32>::new(
            BufferUsages::STORAGE | BufferUsages::COPY_SRC | BufferUsages::COPY_DST,
        );
        atomics_buffer.reserve(2, render_device);

        GpuVoxelMaterial {
            voxels_buffer,
            edge_table_buffer,
            tri_table_buffer,
            vertices_buffer,
            vertices_staging_buffer,
            normals_buffer,
            uvs_buffer,
            indices_buffer,
            atomics_buffer,
        }
    }

    /// Initializes the [`GpuVoxelMaterial`] of newly created [`VoxelMaterial`].
    pub fn initialize(
        mut commands: Commands,
        render_queue: Res<RenderQueue>,
        render_device: Res<RenderDevice>,
        mut gpu_voxel_materials: ResMut<VoxelMaterialComponents<GpuVoxelMaterial>>,
        voxel_material_query: Extract<Query<(Entity, &VoxelMaterial), Added<Volumetric>>>,
    ) -> () {
        for (entity, voxel_material) in voxel_material_query.iter() {
            let mut gpu_voxel_material = GpuVoxelMaterial::new(
                render_device.as_ref(),
                render_queue.as_ref(),
                voxel_material,
            );

            gpu_voxel_material
                .vertices_buffer
                .write_buffer(render_device.as_ref(), render_queue.as_ref());

            gpu_voxel_materials.insert(entity, gpu_voxel_material);

            commands.get_or_spawn(entity).insert(Volumetric);
        }
    }

    /// Extracts the current data from all [`VoxelMaterial`]s into the corresponding [`GpuVoxelMaterial`]s.
    pub fn extract(
        render_queue: Res<RenderQueue>,
        render_device: Res<RenderDevice>,
        mut gpu_voxel_materials: ResMut<VoxelMaterialComponents<GpuVoxelMaterial>>,
        voxel_material_query: Extract<Query<(Entity, &VoxelMaterial), With<Volumetric>>>,
    ) {
        for (entity, voxel_material) in voxel_material_query.iter() {
            let mut gpu_voxel_material = GpuVoxelMaterial::new(
                render_device.as_ref(),
                render_queue.as_ref(),
                voxel_material,
            );

            gpu_voxel_material
                .vertices_buffer
                .write_buffer(render_device.as_ref(), render_queue.as_ref());

            gpu_voxel_materials.insert(entity, gpu_voxel_material);
        }
    }
}
