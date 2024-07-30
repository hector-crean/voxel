use bevy::render::render_resource::ShaderType;

#[derive(ShaderType, Clone, Copy)]
pub struct Voxel {
    flags: u32,
    density: f32,
}

impl Default for Voxel {
    fn default() -> Self {
        Self {
            flags: 0,
            density: 1.,
        }
    }
}

impl Voxel {
    pub fn new(flags: u32, density: f32) -> Self {
        Self { flags, density }
    }
}
