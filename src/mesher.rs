use bevy::{
    prelude::Mesh,
    render::{mesh::Indices, render_resource::PrimitiveTopology},
};
use block_mesh::{greedy_quads, GreedyQuadsBuffer, RIGHT_HANDED_Y_UP_CONFIG};

use crate::{
    chunk::{self, ChunkBoundary},
    voxel::VOXEL_SIZE,
};

pub fn generate_mesh(chunk: &ChunkBoundary) -> Mesh {
    let mut buffer = GreedyQuadsBuffer::new(ChunkBoundary::size() as usize);
    generate_mesh_with_buffer(chunk, &mut buffer)
}

pub fn generate_mesh_with_buffer(chunk: &ChunkBoundary, buffer: &mut GreedyQuadsBuffer) -> Mesh {
    let faces = RIGHT_HANDED_Y_UP_CONFIG.faces;

    greedy_quads(
        chunk.voxels(),
        &chunk::BoundaryShape {},
        [0; 3],
        [17; 3],
        &faces,
        buffer,
    );
    let num_indices = buffer.quads.num_quads() * 6;
    let num_vertices = buffer.quads.num_quads() * 4;

    let mut indices = Vec::with_capacity(num_indices);
    let mut positions = Vec::with_capacity(num_vertices);
    let mut normals = Vec::with_capacity(num_vertices);
    let mut tex_coords = Vec::with_capacity(num_vertices);

    for (group, face) in buffer.quads.groups.iter().zip(faces.into_iter()) {
        for quad in group.iter() {
            indices.extend_from_slice(&face.quad_mesh_indices(positions.len() as u32));
            positions.extend_from_slice(&face.quad_mesh_positions(quad, VOXEL_SIZE));
            normals.extend_from_slice(&face.quad_mesh_normals());
            tex_coords.extend_from_slice(&face.tex_coords(
                RIGHT_HANDED_Y_UP_CONFIG.u_flip_face,
                true,
                quad,
            ));
        }
    }

    tex_coords
        .iter_mut()
        .flatten()
        .for_each(|uv| *uv *= 1.0 / 16.0);

    let mut mesh = Mesh::new(PrimitiveTopology::TriangleList);
    mesh.insert_attribute(Mesh::ATTRIBUTE_POSITION, positions);
    mesh.insert_attribute(Mesh::ATTRIBUTE_NORMAL, normals);
    mesh.insert_attribute(Mesh::ATTRIBUTE_UV_0, tex_coords);
    mesh.set_indices(Some(Indices::U32(indices)));

    mesh
}
