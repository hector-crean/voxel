// Define a structure representing a voxel with flags and density.
struct Voxel {
    flags: u32, // Stores flags related to the voxel (e.g., active or not).
    density: f32, // Stores the density value of the voxel.
};

// Define a structure representing a buffer containing an array of voxels.
struct VoxelBuffer {
    data: array<Voxel>, // Array of voxels.
};

// Define a structure representing a buffer containing an array of vertex positions.
struct VertexBuffer {
    data: array<vec3<f32>>, // Array of vertex positions.
};

// Define a structure representing a buffer containing an array of vertex normals.
struct NormalBuffer {
    data: array<vec3<f32>>, // Array of vertex normals.
};

// Define a structure representing a buffer containing an array of indices.
struct IndexBuffer {
    data: array<u32>, // Array of indices.
};

// Define a structure representing a buffer containing an array of texture coordinates.
struct UvBuffer {
    data: array<vec2<f32>>, // Array of UV coordinates.
};

// Define a structure containing atomic counters for vertices and indices.
struct Atomics {
    vertices_head: atomic<u32>, // Atomic counter for vertex indices.
    indices_head: atomic<u32>, // Atomic counter for indices.
};

// Define a structure representing a lookup table for edge cases.
struct EdgeTable {
    data: array<u32, 256>, // Array of edge data for 256 configurations.
};

// Define a structure representing a lookup table for triangle cases.
struct TriangleTable {
    data: array<array<i32, 16>, 256>, // Array of triangle data for 256 configurations, each with up to 16 entries.
};

// Bindings for various buffers and tables used in the shader.
@group(0) @binding(0) var<storage, read_write> uniform_edge_table: EdgeTable;
@group(0) @binding(1) var<storage, read_write> uniform_tri_table: TriangleTable;
@group(0) @binding(2) var<storage, read_write> in_voxels: VoxelBuffer;
@group(0) @binding(3) var<storage, read_write> global_atomics: Atomics;
@group(0) @binding(4) var<storage, read_write> out_vertices: VertexBuffer;
@group(0) @binding(5) var<storage, read_write> out_normals: NormalBuffer;
@group(0) @binding(6) var<storage, read_write> out_indices: IndexBuffer;
@group(0) @binding(7) var<storage, read_write> out_uvs: UvBuffer;

const chunk_sz = 32; // Define the size of a chunk.

// Function to get a flat index for a given position in the 3D grid.
fn get_flat_index(pos: vec3<i32>) -> u32 {
    return u32(pos.x + pos.y * chunk_sz + pos.z * chunk_sz * chunk_sz);
}

// Function to get the density of a voxel at a given position.
fn get_voxel_density(pos: vec3<i32>) -> f32 {
    var density: f32 = 0.0;
    if (pos.x >= 0 && pos.x < chunk_sz
     && pos.y >= 0 && pos.y < chunk_sz
     && pos.z >= 0 && pos.z < chunk_sz) {
        density = in_voxels.data[get_flat_index(pos)].density;
    }
    return density;
}

// Function to interpolate between two vertices based on their densities.
fn interp_vertex(p1: vec3<f32>, p2: vec3<f32>, v1: f32, v2: f32) -> vec3<f32> {
    let mu = (0.5 - v1) / (v2 - v1);
    return p1 + mu * (p2 - p1);
}

// Main compute shader entry point with a workgroup size of 8x8x8.
@compute @workgroup_size(8, 8, 8)
fn main(@builtin(global_invocation_id) invocation_id: vec3<u32>) {

    let pos = vec3<i32>(invocation_id); // Convert invocation ID to integer position.
    let voxel = in_voxels.data[get_flat_index(pos)]; // Get the voxel data for the current position.

    // If the voxel is active (flags == 0).
    if (voxel.flags == 0u) {

        // Define the offsets for the 8 corners of the voxel cube.
        let smooth_adj_offsets = array<vec3<i32>, 8>(
            vec3<i32>(0, 0, 1),
            vec3<i32>(1, 0, 1),
            vec3<i32>(1, 0, 0),
            vec3<i32>(0, 0, 0),
            vec3<i32>(0, 1, 1),
            vec3<i32>(1, 1, 1),
            vec3<i32>(1, 1, 0),
            vec3<i32>(0, 1, 0)
        );

        var cube_idx: u32 = 0u; // Initialize the cube index.
        var orient: u32 = 0u; // Initialize the orientation.
        // Define the positions of the 8 corners of the voxel cube.
        let positions = array<vec3<f32>, 8>(
            vec3<f32>(pos + smooth_adj_offsets[0u]),
            vec3<f32>(pos + smooth_adj_offsets[1u]),
            vec3<f32>(pos + smooth_adj_offsets[2u]),
            vec3<f32>(pos + smooth_adj_offsets[3u]),
            vec3<f32>(pos + smooth_adj_offsets[4u]),
            vec3<f32>(pos + smooth_adj_offsets[5u]),
            vec3<f32>(pos + smooth_adj_offsets[6u]),
            vec3<f32>(pos + smooth_adj_offsets[7u]),
        );
        // Get the densities of the 8 corners of the voxel cube.
        let densities = array<f32, 8>(
            get_voxel_density(pos + smooth_adj_offsets[0u]),
            get_voxel_density(pos + smooth_adj_offsets[1u]),
            get_voxel_density(pos + smooth_adj_offsets[2u]),
            get_voxel_density(pos + smooth_adj_offsets[3u]),
            get_voxel_density(pos + smooth_adj_offsets[4u]),
            get_voxel_density(pos + smooth_adj_offsets[5u]),
            get_voxel_density(pos + smooth_adj_offsets[6u]),
            get_voxel_density(pos + smooth_adj_offsets[7u]),
        );
        // Calculate the cube index based on the densities.
        cube_idx = cube_idx | (u32(densities[0u] < 0.5) * (1u << 0u));
        cube_idx = cube_idx | (u32(densities[1u] < 0.5) * (1u << 1u));
        cube_idx = cube_idx | (u32(densities[2u] < 0.5) * (1u << 2u));
        cube_idx = cube_idx | (u32(densities[3u] < 0.5) * (1u << 3u));
        cube_idx = cube_idx | (u32(densities[4u] < 0.5) * (1u << 4u));
        cube_idx = cube_idx | (u32(densities[5u] < 0.5) * (1u << 5u));
        cube_idx = cube_idx | (u32(densities[6u] < 0.5) * (1u << 6u));
        cube_idx = cube_idx | (u32(densities[7u] < 0.5) * (1u << 7u));

        // If the cube is fully inside or outside the surface, skip it.
        if (cube_idx == 0x00u || cube_idx == 0xffu) {
            return;
        }

        // Interpolate the vertices along the edges of the cube.
        var vertices = array<vec3<f32>, 12>(
            f32((uniform_edge_table.data[cube_idx] & (1u <<  0u)) != 0u) * interp_vertex(positions[0u], positions[1u], densities[0u], densities[1u]),
            f32((uniform_edge_table.data[cube_idx] & (1u <<  1u)) != 0u) * interp_vertex(positions[1u], positions[2u], densities[1u], densities[2u]),
            f32((uniform_edge_table.data[cube_idx] & (1u <<  2u)) != 0u) * interp_vertex(positions[2u], positions[3u], densities[2u], densities[3u]),
            f32((uniform_edge_table.data[cube_idx] & (1u <<  3u)) != 0u) * interp_vertex(positions[3u], positions[0u], densities[3u], densities[0u]),
            f32((uniform_edge_table.data[cube_idx] & (1u <<  4u)) != 0u) * interp_vertex(positions[4u], positions[5u], densities[4u], densities[5u]),
            f32((uniform_edge_table.data[cube_idx] & (1u <<  5u)) != 0u) * interp_vertex(positions[5u], positions[6u], densities[5u], densities[6u]),
            f32((uniform_edge_table.data[cube_idx] & (1u <<  6u)) != 0u) * interp_vertex(positions[6u], positions[7u], densities[6u], densities[7u]),
            f32((uniform_edge_table.data[cube_idx] & (1u <<  7u)) != 0u) * interp_vertex(positions[7u], positions[4u], densities[7u], densities[4u]),
            f32((uniform_edge_table.data[cube_idx] & (1u <<  8u)) != 0u) * interp_vertex(positions[0u], positions[4u], densities[0u], densities[4u]),
            f32((uniform_edge_table.data[cube_idx] & (1u <<  9u)) != 0u) * interp_vertex(positions[1u], positions[5u], densities[1u], densities[5u]),
            f32((uniform_edge_table.data[cube_idx] & (1u << 10u)) != 0u) * interp_vertex(positions[2u], positions[6u], densities[2u], densities[6u]),
            f32((uniform_edge_table.data[cube_idx] & (1u << 11u)) != 0u) * interp_vertex(positions[3u], positions[7u], densities[3u], densities[7u]),
        );

        var tri_idx: u32 = 0u; // Initialize the triangle index.
        // Loop to generate triangles for the current voxel.
        loop {
            var start_vert_idx = atomicAdd(&global_atomics.vertices_head, 3u); // Allocate space for 3 vertices.
            var start_indices_idx = atomicAdd(&global_atomics.indices_head, 3u); // Allocate space for 3 indices.

            let v0 = vertices[ uniform_tri_table.data[cube_idx][tri_idx + 0u] ]; // Get the first vertex of the triangle.
            let v1 = vertices[ uniform_tri_table.data[cube_idx][tri_idx + 1u] ]; // Get the second vertex of the triangle.
            let v2 = vertices[ uniform_tri_table.data[cube_idx][tri_idx + 2u] ]; // Get the third vertex of the triangle.

            out_vertices.data[start_vert_idx + 0u] = v0; // Store the first vertex.
            out_vertices.data[start_vert_idx + 1u] = v1; // Store the second vertex.
            out_vertices.data[start_vert_idx + 2u] = v2; // Store the third vertex.

            out_indices.data[start_indices_idx + 0u] = start_vert_idx + 0u; // Store the first index.
            out_indices.data[start_indices_idx + 1u] = start_vert_idx + 1u; // Store the second index.
            out_indices.data[start_indices_idx + 2u] = start_vert_idx + 2u; // Store the third index.

            let normal = cross(v0 - v1, v0 - v2); // Calculate the normal for the triangle.
            out_normals.data[start_vert_idx + 0u] = normal; // Store the normal for the first vertex.
            out_normals.data[start_vert_idx + 1u] = normal; // Store the normal for the second vertex.
            out_normals.data[start_vert_idx + 2u] = normal; // Store the normal for the third vertex.

            // Store default UV coordinates for the triangle vertices.
            out_uvs.data[start_vert_idx + 0u] = vec2<f32>(0.0, 0.0);
            out_uvs.data[start_vert_idx + 1u] = vec2<f32>(1.0, 0.0);
            out_uvs.data[start_vert_idx + 2u] = vec2<f32>(0.0, 1.0);

            tri_idx = tri_idx + 3u; // Move to the next triangle index.
            // Break the loop if there are no more triangles to process.
            if (uniform_tri_table.data[cube_idx][tri_idx] == -1) {
                break;
            }
        }
    } else { // If the voxel is inactive (flags != 0).

        // Define the faces and adjacent offsets for a block.
        var block_faces = array<array<vec3<f32>, 4>, 6>(
            array<vec3<f32>, 4>(
                vec3<f32>(0.5, -0.5, -0.5),
                vec3<f32>(0.5,  0.5, -0.5),
                vec3<f32>(0.5,  0.5,  0.5),
                vec3<f32>(0.5, -0.5,  0.5),
            ),
            array<vec3<f32>, 4>(
                vec3<f32>(-0.5, -0.5,  0.5),
                vec3<f32>(-0.5,  0.5,  0.5),
                vec3<f32>(-0.5,  0.5, -0.5),
                vec3<f32>(-0.5, -0.5, -0.5)
            ),
            array<vec3<f32>, 4>(
                vec3<f32>(-0.5, 0.5,  0.5),
                vec3<f32>( 0.5, 0.5,  0.5),
                vec3<f32>( 0.5, 0.5, -0.5),
                vec3<f32>(-0.5, 0.5, -0.5)
            ),
            array<vec3<f32>, 4>(
                vec3<f32>(-0.5, -0.5, -0.5),
                vec3<f32>( 0.5, -0.5, -0.5),
                vec3<f32>( 0.5, -0.5,  0.5),
                vec3<f32>(-0.5, -0.5,  0.5)
            ),
            array<vec3<f32>, 4>(
                vec3<f32>( 0.5, -0.5, 0.5),
                vec3<f32>( 0.5,  0.5, 0.5),
                vec3<f32>(-0.5,  0.5, 0.5),
                vec3<f32>(-0.5, -0.5, 0.5)
            ),
            array<vec3<f32>, 4>(
                vec3<f32>(-0.5, -0.5, -0.5),
                vec3<f32>(-0.5,  0.5, -0.5),
                vec3<f32>( 0.5,  0.5, -0.5),
                vec3<f32>( 0.5, -0.5, -0.5)
            ),
        );
        // Define the adjacent offsets for the block faces.
        var block_adj_offsets = array<vec3<i32>, 6>(
            vec3<i32>( 1,  0,  0),
            vec3<i32>(-1,  0,  0),
            vec3<i32>( 0,  1,  0),
            vec3<i32>( 0, -1,  0),
            vec3<i32>( 0,  0,  1),
            vec3<i32>( 0,  0, -1),
        );

        var dir: u32 = 0u; // Initialize the direction index.
        // Loop to process each face of the block.
        loop {
            let adj_pos = pos + block_adj_offsets[dir]; // Get the adjacent position.
            let adj_density = get_voxel_density(pos); // Get the density of the adjacent voxel.

            // If the adjacent voxel is below the surface threshold.
            if (adj_density < 0.5) {
                var pos = vec3<f32>(invocation_id); // Convert the position to float.

                let start_vert_idx = atomicAdd(&global_atomics.vertices_head, 4u); // Allocate space for 4 vertices.
                let start_indices_idx = atomicAdd(&global_atomics.indices_head, 6u); // Allocate space for 6 indices.

                let v0 = block_faces[dir][0u]; // Get the first vertex of the face.
                let v1 = block_faces[dir][1u]; // Get the second vertex of the face.
                let v2 = block_faces[dir][2u]; // Get the third vertex of the face.
                let v3 = block_faces[dir][3u]; // Get the fourth vertex of the face.

                out_vertices.data[start_vert_idx + 0u] = pos + v0; // Store the first vertex.
                out_vertices.data[start_vert_idx + 1u] = pos + v1; // Store the second vertex.
                out_vertices.data[start_vert_idx + 2u] = pos + v2; // Store the third vertex.
                out_vertices.data[start_vert_idx + 3u] = pos + v3; // Store the fourth vertex.

                let normal = cross(v0 - v1, v0 - v2); // Calculate the normal for the face.
                out_normals.data[start_vert_idx + 0u] = normal; // Store the normal for the first vertex.
                out_normals.data[start_vert_idx + 1u] = normal; // Store the normal for the second vertex.
                out_normals.data[start_vert_idx + 2u] = normal; // Store the normal for the third vertex.
                out_normals.data[start_vert_idx + 3u] = normal; // Store the normal for the fourth vertex.

                // Store default UV coordinates for the face vertices.
                out_uvs.data[start_vert_idx + 0u] = vec2<f32>(0.0, 0.0);
                out_uvs.data[start_vert_idx + 1u] = vec2<f32>(1.0, 0.0);
                out_uvs.data[start_vert_idx + 2u] = vec2<f32>(1.0, 1.0);
                out_uvs.data[start_vert_idx + 3u] = vec2<f32>(0.0, 1.0);

                // Store indices for two triangles forming the face.
                out_indices.data[start_indices_idx + 0u] = start_vert_idx + 0u;
                out_indices.data[start_indices_idx + 1u] = start_vert_idx + 1u;
                out_indices.data[start_indices_idx + 2u] = start_vert_idx + 2u;
                out_indices.data[start_indices_idx + 3u] = start_vert_idx + 0u;
                out_indices.data[start_indices_idx + 4u] = start_vert_idx + 2u;
                out_indices.data[start_indices_idx + 5u] = start_vert_idx + 3u;
            }

            dir = dir + 1u; // Move to the next direction.
            // Break the loop if all directions have been processed.
            if (dir >= 6u) {
                break;
            }
        }
    }
}
