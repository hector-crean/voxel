use std::sync::atomic::AtomicU32;

use bevy::render::render_resource::ShaderType;

#[derive(ShaderType)]
pub struct Atomics {
    vertices_head: AtomicU32,
    indices_head: AtomicU32,
}
