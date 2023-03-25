use bevy::{
    prelude::{info_span, Mesh},
    render::{mesh::Indices, render_resource::PrimitiveTopology},
};

use super::{
    chunk_boundary::ChunkBoundary,
    quads::{generate_quads_with_buffer, QuadGroups},
};

//const UV_SCALE: f32 = 1.0 / 16.0;

pub fn generate_mesh(chunk: ChunkBoundary) -> (Option<Mesh>, Option<Mesh>) {
    let _span = info_span!("Generate mesh only").entered();
    let mut buffer = QuadGroups::default();

    let solid_mesh = generate_mesh_with_buffer(true, &chunk, &mut buffer);
    let transparent_mesh = generate_mesh_with_buffer(false, &chunk, &mut buffer);

    (solid_mesh, transparent_mesh)
}

/// Generate a mesh according to the chunk boundary
/// Uses the algorithm described in this article : https://playspacefarer.com/voxel-meshing/
pub fn generate_mesh_with_buffer(
    solid_pass: bool,
    chunk: &ChunkBoundary,
    buffer: &mut QuadGroups,
) -> Option<Mesh> {
    generate_quads_with_buffer(solid_pass, chunk, buffer);

    let num_quads = buffer.num_quads();
    if num_quads == 0 {
        return None;
    }

    let num_indices = num_quads * 6;
    let num_vertices = num_quads * 4;

    let mut indices = Vec::with_capacity(num_indices);
    let mut positions = Vec::with_capacity(num_vertices);
    let mut normals = Vec::with_capacity(num_vertices);
    let mut tex_coords = Vec::with_capacity(num_vertices);
    let mut ao = Vec::with_capacity(num_vertices);
    let mut texture_indices = Vec::with_capacity(num_vertices);

    for face in buffer.iter_with_ao(chunk) {
        indices.extend_from_slice(&face.indices(positions.len() as u32));
        positions.extend_from_slice(&face.positions(1.0));
        normals.extend_from_slice(&face.normals());
        tex_coords.extend_from_slice(&face.uvs(false, true));
        ao.extend_from_slice(&face.aos());
        texture_indices.extend_from_slice(&[face.texture_indice(); 4]);
    }

    let mut mesh = Mesh::new(PrimitiveTopology::TriangleList);
    mesh.insert_attribute(Mesh::ATTRIBUTE_POSITION, positions);
    mesh.insert_attribute(Mesh::ATTRIBUTE_NORMAL, normals);
    mesh.insert_attribute(Mesh::ATTRIBUTE_UV_0, tex_coords);
    mesh.insert_attribute(Mesh::ATTRIBUTE_COLOR, convert_ao(&ao));
    mesh.insert_attribute(super::render::ATTRIBUTE_VOXEL_INDICES, texture_indices);
    mesh.set_indices(Some(Indices::U32(indices)));

    Some(mesh)
}

fn convert_ao(ao: &[u32]) -> Vec<[f32; 4]> {
    ao.iter()
        .map(|val| match val {
            0 => [0.1, 0.1, 0.1, 1.0],
            1 => [0.25, 0.25, 0.25, 1.0],
            2 => [0.5, 0.5, 0.5, 1.0],
            _ => [1.0, 1.0, 1.0, 1.0],
        })
        .collect()
}
