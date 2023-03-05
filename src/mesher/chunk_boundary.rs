use block_mesh::ndshape::{ConstShape, ConstShape3u32};

use crate::{chunk::ChunkData, voxel::Voxel};

const BOUNDARY_EDGE: u32 = ChunkData::edge() + 2;
pub type BoundaryShape = ConstShape3u32<BOUNDARY_EDGE, BOUNDARY_EDGE, BOUNDARY_EDGE>;

pub struct ChunkBoundary {
    voxels: Box<[Voxel]>,
}

#[allow(dead_code)]
impl ChunkBoundary {
    pub fn new(center: ChunkData, neighbors: [ChunkData; 6]) -> Self {
        const MAX: u32 = ChunkData::edge();
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
