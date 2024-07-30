use bevy::{prelude::*, utils::HashMap};

use crate::{bundles::volumetric_bundle::VolumetricBundle, CHUNK_SZ, CHUNK_SZ_2, CHUNK_SZ_3};

use super::voxel::Voxel;

#[derive(Component)]
pub struct VoxelMaterial {
    pub voxels: Vec<Voxel>,
    pub chunk_size: u32,
}

impl VoxelMaterial {
    pub fn generate_random(mut commands: Commands) {
        let mut voxels = Vec::from_iter(0..CHUNK_SZ_3)
            .iter()
            .map(|_| Voxel::default())
            .collect::<Vec<Voxel>>();

        for z in 0..CHUNK_SZ {
            for y in 0..CHUNK_SZ {
                for x in 0..CHUNK_SZ {
                    let height = 4.0 + 8.0 - (y as f32);
                    let mut density = 0.0;

                    if height > 1.0 {
                        density = 1.0;
                    } else if height > 0.0 {
                        density = height;
                    }

                    voxels[x + y * CHUNK_SZ + z * CHUNK_SZ_2] = Voxel::new(0, density);
                }
            }
        }

        commands.spawn(VolumetricBundle::new(VoxelMaterial {
            chunk_size: CHUNK_SZ_3 as u32,
            voxels,
        }));
    }
}

#[derive(Clone, Resource)]
pub struct VoxelMaterialComponents<C>(pub HashMap<Entity, C>);

impl<C> VoxelMaterialComponents<C> {
    pub fn get(&self, k: &Entity) -> Option<&C> {
        self.0.get(k)
    }

    pub fn get_mut(&mut self, k: &Entity) -> Option<&mut C> {
        self.0.get_mut(k)
    }

    pub fn insert(&mut self, k: Entity, v: C) {
        self.0.insert(k, v);
    }
}

impl<C> FromWorld for VoxelMaterialComponents<C> {
    fn from_world(_world: &mut World) -> Self {
        Self(default())
    }
}
