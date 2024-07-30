use bevy::{prelude::*, render::extract_component::ExtractComponent};

use crate::data::{voxel::Voxel, voxel_material::VoxelMaterial};

#[derive(Clone, Copy, Component, ExtractComponent)]
pub struct Volumetric;

#[derive(Bundle)]
pub struct VolumetricBundle {
    pub volumetric: Volumetric,
    pub material: VoxelMaterial,
}

impl VolumetricBundle {
    /// Creates a new terrain bundle from the config.
    pub fn new(voxel_material: VoxelMaterial) -> Self {
        Self {
            volumetric: Volumetric,
            material: voxel_material,
        }
    }
}
