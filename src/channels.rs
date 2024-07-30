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

use crate::data::{gpu_voxel_material::GpuVoxelMaterial, voxel_material::VoxelMaterialComponents};

#[derive(Resource, Deref)]
pub struct MainWorldReceiver(pub Receiver<Vec<u32>>);

impl MainWorldReceiver {
    pub fn receive(receiver: Res<Self>) {
        if let Ok(data) = receiver.try_recv() {
            println!("Received data from render world: {data:?}");
        }
    }
}

#[derive(Resource, Deref)]
pub struct RenderWorldSender(pub Sender<Vec<u32>>);

impl RenderWorldSender {
    pub fn map_and_read_buffer(
        render_device: Res<RenderDevice>,
        gpu_voxel_materials: ResMut<VoxelMaterialComponents<GpuVoxelMaterial>>,
        sender: Res<Self>,
    ) {
        for (_, gpu_voxel_material) in &gpu_voxel_materials.0 {
            let buffer_slice = gpu_voxel_material.vertices_staging_buffer.slice(..);

            let (s, r) = crossbeam_channel::unbounded::<()>();

            buffer_slice.map_async(MapMode::Read, move |result| match result {
                Ok(_) => s.send(()).expect("Failed to send map update"),
                Err(err) => panic!("Failed to map buffer {err}"),
            });

            render_device.poll(Maintain::Wait);

            r.recv().expect("Failed to receive the map_async message");

            {
                let buffer_view = buffer_slice.get_mapped_range();
                let data = buffer_view
                    .chunks(std::mem::size_of::<u32>())
                    .map(|chunk| u32::from_ne_bytes(chunk.try_into().expect("should be a u32")))
                    .collect::<Vec<u32>>();
                sender
                    .send(data)
                    .expect("Failed to send data to main world");
            }

            gpu_voxel_material.vertices_staging_buffer.unmap();
        }
    }
}
