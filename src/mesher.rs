use bevy::{
    prelude::Mesh,
    render::{mesh::Indices, render_resource::PrimitiveTopology},
};
use block_mesh::{
    greedy_quads,
    ndshape::{ConstShape, ConstShape3u32},
    GreedyQuadsBuffer, RIGHT_HANDED_Y_UP_CONFIG,
};

use crate::{chunk::Chunk, voxel::Voxel};

const UV_SCALE: f32 = 1.0 / 16.0;

const BOUNDARY_EDGE: u32 = Chunk::edge() + 2;
pub type BoundaryShape = ConstShape3u32<BOUNDARY_EDGE, BOUNDARY_EDGE, BOUNDARY_EDGE>;

pub struct ChunkBoundary {
    voxels: Box<[Voxel]>,
}

#[allow(dead_code)]
impl ChunkBoundary {
    pub fn new(center: Chunk, neighbors: [Chunk; 6]) -> Self {
        const MAX: u32 = Chunk::edge();
        const BOUND: u32 = MAX + 1;

        let voxels: Box<[Voxel]> = (0..BoundaryShape::SIZE)
            .map(BoundaryShape::delinearize)
            .map(|[x, y, z]| match (x, y, z) {
                (1..=MAX, 1..=MAX, 1..=MAX) => center.get(x - 1, y - 1, z - 1),
                (BOUND, 1..=MAX, 1..=MAX) => neighbors[0].get(0, y - 1, z - 1),
                (0, 1..=MAX, 1..=MAX) => neighbors[1].get(MAX - 1, y - 1, z - 1),
                (1..=MAX, BOUND, 1..=MAX) => neighbors[2].get(x - 1, 0, z - 1),
                (1..=MAX, 0, 1..=MAX) => neighbors[3].get(x - 1, MAX - 1, z - 1),
                (1..=MAX, 1..=MAX, BOUND) => neighbors[4].get(x - 1, y - 1, 0),
                (1..=MAX, 1..=MAX, 0) => neighbors[5].get(x - 1, y - 1, MAX - 1),

                (_, _, _) => Voxel::Empty,
            })
            .collect();

        Self { voxels }
    }

    pub fn voxels(&self) -> &[Voxel] {
        &self.voxels
    }

    pub const fn size() -> u32 {
        BoundaryShape::SIZE
    }

    pub fn linearize(x: u32, y: u32, z: u32) -> usize {
        BoundaryShape::linearize([x, y, z]) as usize
    }

    pub fn delinearize(idx: u32) -> (u32, u32, u32) {
        let res = BoundaryShape::delinearize(idx);
        (res[0], res[1], res[2])
    }
}

pub fn generate_mesh(chunk: &ChunkBoundary) -> Mesh {
    let mut buffer = GreedyQuadsBuffer::new(ChunkBoundary::size() as usize);
    generate_mesh_with_buffer(chunk, &mut buffer)
}

pub fn generate_mesh_with_buffer(chunk: &ChunkBoundary, buffer: &mut GreedyQuadsBuffer) -> Mesh {
    let faces = RIGHT_HANDED_Y_UP_CONFIG.faces;

    greedy_quads(
        chunk.voxels(),
        &BoundaryShape {},
        [0; 3],
        [Chunk::edge() + 1; 3],
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
            positions.extend_from_slice(&face.quad_mesh_positions(quad, Voxel::size()));
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
        .for_each(|uv| *uv *= UV_SCALE);

    let mut mesh = Mesh::new(PrimitiveTopology::TriangleList);
    mesh.insert_attribute(Mesh::ATTRIBUTE_POSITION, positions);
    mesh.insert_attribute(Mesh::ATTRIBUTE_NORMAL, normals);
    mesh.insert_attribute(Mesh::ATTRIBUTE_UV_0, tex_coords);
    mesh.set_indices(Some(Indices::U32(indices)));

    mesh
}
