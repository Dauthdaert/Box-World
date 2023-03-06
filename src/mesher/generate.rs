use bevy::{
    prelude::Mesh,
    render::{mesh::Indices, render_resource::PrimitiveTopology},
};
use block_mesh::{visible_block_faces, UnitQuadBuffer, RIGHT_HANDED_Y_UP_CONFIG};

use crate::{chunk::ChunkData, voxel::Voxel};

use super::chunk_boundary::{BoundaryShape, ChunkBoundary};

const UV_SCALE: f32 = 1.0 / 16.0;

pub fn generate_mesh(chunk: ChunkBoundary) -> Mesh {
    let mut buffer = UnitQuadBuffer::new();
    generate_mesh_with_buffer(chunk, &mut buffer)
}

pub fn generate_mesh_with_buffer(chunk: ChunkBoundary, buffer: &mut UnitQuadBuffer) -> Mesh {
    let faces = RIGHT_HANDED_Y_UP_CONFIG.faces;

    visible_block_faces(
        chunk.voxels(),
        &BoundaryShape {},
        [0; 3],
        [ChunkData::edge() + 1; 3],
        &faces,
        buffer,
    );
    let num_indices = buffer.num_quads() * 6;
    let num_vertices = buffer.num_quads() * 4;

    let mut indices = Vec::with_capacity(num_indices);
    let mut positions = Vec::with_capacity(num_vertices);
    let mut normals = Vec::with_capacity(num_vertices);
    let mut tex_coords = Vec::with_capacity(num_vertices);
    let mut ao = Vec::with_capacity(num_vertices);

    for (group, face) in buffer.groups.iter().zip(faces.into_iter()) {
        for quad in group.iter() {
            indices.extend_from_slice(
                &face.quad_mesh_indices(&(*quad).into(), positions.len() as u32),
            );
            positions.extend_from_slice(&face.quad_mesh_positions(&(*quad).into(), Voxel::size()));
            normals.extend_from_slice(&face.quad_mesh_normals());
            tex_coords.extend_from_slice(&face.tex_coords(
                RIGHT_HANDED_Y_UP_CONFIG.u_flip_face,
                true,
                &(*quad).into(),
            ));
            ao.extend_from_slice(&quad.ao);
        }
    }

    tex_coords
        .iter_mut()
        .flatten()
        .for_each(|uv| *uv *= UV_SCALE);

    let mut mesh = Mesh::new(PrimitiveTopology::TriangleList);
    mesh.insert_attribute(Mesh::ATTRIBUTE_POSITION, positions);
    mesh.insert_attribute(Mesh::ATTRIBUTE_NORMAL, normals);
    mesh.insert_attribute(Mesh::ATTRIBUTE_UV_0, tex_coords);
    mesh.insert_attribute(Mesh::ATTRIBUTE_COLOR, convert_ao(&ao));
    mesh.set_indices(Some(Indices::U32(indices)));

    mesh
}

fn convert_ao(ao: &[u8]) -> Vec<[f32; 4]> {
    ao.iter()
        .map(|val| match val {
            0 => [0.1, 0.1, 0.1, 1.0],
            1 => [0.25, 0.25, 0.25, 1.0],
            2 => [0.5, 0.5, 0.5, 1.0],
            _ => [1.0, 1.0, 1.0, 1.0],
        })
        .collect()
}
