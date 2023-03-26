use bevy::{
    prelude::{info_span, Mesh},
    render::{mesh::Indices, render_resource::PrimitiveTopology},
};

use crate::chunk::{to_sunlight, to_torchlight};

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
    let mut lights = Vec::with_capacity(num_vertices);
    let mut ao = Vec::with_capacity(num_vertices);
    let mut texture_indices = Vec::with_capacity(num_vertices);

    for face in buffer.iter_with_ao(chunk) {
        indices.extend_from_slice(&face.indices(positions.len() as u32));
        positions.extend_from_slice(&face.positions(1.0));
        normals.extend_from_slice(&face.normals());
        ao.extend_from_slice(&face.aos());
        texture_indices.extend_from_slice(&[face.texture_indice(); 4]);

        let [face_x, face_y, face_z] = face.voxel();
        let (x, y, z) = match (face.side().axis, face.side().positive) {
            (super::side::Axis::X, true) => (face_x + 1, face_y, face_z),
            (super::side::Axis::X, false) => (face_x - 1, face_y, face_z),
            (super::side::Axis::Y, true) => (face_x, face_y + 1, face_z),
            (super::side::Axis::Y, false) => (face_x, face_y - 1, face_z),
            (super::side::Axis::Z, true) => (face_x, face_y, face_z + 1),
            (super::side::Axis::Z, false) => (face_x, face_y, face_z - 1),
        };
        let combined_light = chunk.lights()[ChunkBoundary::linearize(x, y, z)];
        let torchlight = convert_light(to_torchlight(combined_light));
        let sunlight = convert_light(to_sunlight(combined_light));
        lights.extend_from_slice(&[
            [torchlight, sunlight],
            [torchlight, sunlight],
            [torchlight, sunlight],
            [torchlight, sunlight],
        ]);

        tex_coords.extend_from_slice(&face.uvs(false, true));
    }

    let mut mesh = Mesh::new(PrimitiveTopology::TriangleList);
    mesh.insert_attribute(Mesh::ATTRIBUTE_POSITION, positions);
    mesh.insert_attribute(Mesh::ATTRIBUTE_NORMAL, normals);
    mesh.insert_attribute(Mesh::ATTRIBUTE_UV_0, tex_coords);
    mesh.insert_attribute(Mesh::ATTRIBUTE_COLOR, convert_ao(&ao));
    mesh.insert_attribute(super::render::ATTRIBUTE_VOXEL_INDICES, texture_indices);
    mesh.insert_attribute(super::render::ATTRIBUTE_VOXEL_LIGHTS, lights);
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

fn convert_light(light: u8) -> f32 {
    match light {
        0 => 0.25,
        1 => 0.4,
        2 => 0.5,
        3 => 0.75,
        4 => 0.9,
        5 => 1.0,
        6 => 1.1,
        7 => 1.2,
        8 => 1.3,
        9 => 1.5,
        10 => 2.0,
        11 => 3.0,
        12 => 4.0,
        13 => 4.5,
        14 => 5.0,
        15 => 7.5,
        _ => 10.0,
    }
}
