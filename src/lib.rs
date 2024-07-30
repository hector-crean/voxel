pub mod bundles;
pub mod channels;
pub mod data;
pub mod render;
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
use bundles::volumetric_bundle::Volumetric;
use channels::{MainWorldReceiver, RenderWorldSender};
use crossbeam_channel::{Receiver, Sender};
use data::{
    gpu_voxel_material::GpuVoxelMaterial,
    gpu_voxel_material_bind_group::GpuVoxelMaterialBindGroups,
    voxel_material::{VoxelMaterial, VoxelMaterialComponents},
};
use render::voxel_mesh_compute_pipeline::{
    VoxelMeshComputeNode, VoxelMeshComputeNodeLabel, VoxelMeshComputePipeline,
};

const CHUNK_SZ: usize = 32;
const CHUNK_SZ_2: usize = CHUNK_SZ * CHUNK_SZ;
const CHUNK_SZ_3: usize = CHUNK_SZ * CHUNK_SZ * CHUNK_SZ;

pub struct GpuReadbackPlugin;

impl Plugin for GpuReadbackPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins((ExtractComponentPlugin::<Volumetric>::default(),))
            .add_systems(Startup, VoxelMaterial::generate_random)
            .add_systems(Update, MainWorldReceiver::receive);
    }

    fn finish(&self, app: &mut App) {
        let (s, r) = crossbeam_channel::unbounded();
        app.insert_resource(MainWorldReceiver(r));

        let render_app = app.sub_app_mut(RenderApp);

        render_app
            .init_resource::<VoxelMeshComputePipeline>()
            .insert_resource(RenderWorldSender(s))
            .init_resource::<VoxelMaterialComponents<GpuVoxelMaterial>>()
            .init_resource::<VoxelMaterialComponents<GpuVoxelMaterialBindGroups>>()
            .add_systems(
                ExtractSchedule,
                (
                    GpuVoxelMaterial::initialize,
                    GpuVoxelMaterial::extract.after(GpuVoxelMaterial::initialize),
                    GpuVoxelMaterialBindGroups::initialise.after(GpuVoxelMaterial::initialize),
                )
                    .in_set(RenderSet::ExtractCommands),
            )
            .add_systems(
                Render,
                (
                    GpuVoxelMaterialBindGroups::prepare.in_set(RenderSet::PrepareBindGroups), // We don't need to recreate the bind group every frame
                    RenderWorldSender::map_and_read_buffer.after(RenderSet::Render),
                ),
            );

        let voxel_mesh_compute_node = VoxelMeshComputeNode::from_world(render_app.world_mut());

        let mut render_graph = render_app.world_mut().resource_mut::<RenderGraph>();

        render_graph.add_node(VoxelMeshComputeNodeLabel, voxel_mesh_compute_node);
    }
}
