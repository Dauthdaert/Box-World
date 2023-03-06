use ndshape::{ConstShape, ConstShape3usize};

use crate::{chunk::ChunkData, voxel::Voxel};

const BOUNDARY_EDGE: usize = ChunkData::edge() + 2;
pub type BoundaryShape = ConstShape3usize<BOUNDARY_EDGE, BOUNDARY_EDGE, BOUNDARY_EDGE>;

pub struct ChunkBoundary {
    voxels: Box<[Voxel]>,
}

#[allow(dead_code)]
impl ChunkBoundary {
    pub fn new(center: ChunkData, neighbors: Vec<ChunkData>) -> Self {
        // Must have 26 neighbors
        assert!(neighbors.len() == 26);

        const MAX: usize = ChunkData::edge();
        const BOUND: usize = MAX + 1;

        let voxels: Box<[Voxel]> = (0..BoundaryShape::SIZE)
            .map(BoundaryShape::delinearize)
            .map(|[x, y, z]| match (x, y, z) {
                (0, 0, 0) => neighbors[0].get(MAX - 1, MAX - 1, MAX - 1),
                (0, 0, 1..=MAX) => neighbors[1].get(MAX - 1, MAX - 1, z - 1),
                (0, 0, BOUND) => neighbors[2].get(MAX - 1, MAX - 1, 0),
                (0, 1..=MAX, 0) => neighbors[3].get(MAX - 1, y - 1, MAX - 1),
                (0, 1..=MAX, 1..=MAX) => neighbors[4].get(MAX - 1, y - 1, z - 1),
                (0, 1..=MAX, BOUND) => neighbors[5].get(MAX - 1, y - 1, 0),
                (0, BOUND, 0) => neighbors[6].get(MAX - 1, 0, MAX - 1),
                (0, BOUND, 1..=MAX) => neighbors[7].get(MAX - 1, 0, z - 1),
                (0, BOUND, BOUND) => neighbors[8].get(MAX - 1, 0, 0),
                (1..=MAX, 0, 0) => neighbors[9].get(x - 1, MAX - 1, MAX - 1),
                (1..=MAX, 0, 1..=MAX) => neighbors[10].get(x - 1, MAX - 1, z - 1),
                (1..=MAX, 0, BOUND) => neighbors[11].get(x - 1, MAX - 1, 0),
                (1..=MAX, 1..=MAX, 0) => neighbors[12].get(x - 1, y - 1, MAX - 1),
                (1..=MAX, 1..=MAX, 1..=MAX) => center.get(x - 1, y - 1, z - 1),
                (1..=MAX, 1..=MAX, BOUND) => neighbors[13].get(x - 1, y - 1, 0),
                (1..=MAX, BOUND, 0) => neighbors[14].get(x - 1, 0, MAX - 1),
                (1..=MAX, BOUND, 1..=MAX) => neighbors[15].get(x - 1, 0, z - 1),
                (1..=MAX, BOUND, BOUND) => neighbors[16].get(x - 1, 0, 0),
                (BOUND, 0, 0) => neighbors[17].get(0, MAX - 1, MAX - 1),
                (BOUND, 0, 1..=MAX) => neighbors[18].get(0, MAX - 1, z - 1),
                (BOUND, 0, BOUND) => neighbors[19].get(0, MAX - 1, 0),
                (BOUND, 1..=MAX, 0) => neighbors[20].get(0, y - 1, MAX - 1),
                (BOUND, 1..=MAX, 1..=MAX) => neighbors[21].get(0, y - 1, z - 1),
                (BOUND, 1..=MAX, BOUND) => neighbors[22].get(0, y - 1, 0),
                (BOUND, BOUND, 0) => neighbors[23].get(0, 0, MAX - 1),
                (BOUND, BOUND, 1..=MAX) => neighbors[24].get(0, 0, z - 1),
                (BOUND, BOUND, BOUND) => neighbors[25].get(0, 0, 0),

                (_, _, _) => Voxel::Empty,
            })
            .collect();

        Self { voxels }
    }

    pub fn voxels(&self) -> &[Voxel] {
        &self.voxels
    }

    pub const fn edge() -> usize {
        BOUNDARY_EDGE
    }

    pub const fn size() -> usize {
        BoundaryShape::SIZE
    }

    pub fn linearize(x: usize, y: usize, z: usize) -> usize {
        BoundaryShape::linearize([x, y, z])
    }

    pub fn delinearize(idx: usize) -> (usize, usize, usize) {
        let res = BoundaryShape::delinearize(idx);
        (res[0], res[1], res[2])
    }

    pub fn x_offset() -> usize {
        ChunkBoundary::linearize(1, 0, 0) - ChunkBoundary::linearize(0, 0, 0)
    }

    pub fn y_offset() -> usize {
        ChunkBoundary::linearize(0, 1, 0) - ChunkBoundary::linearize(0, 0, 0)
    }

    pub fn z_offset() -> usize {
        ChunkBoundary::linearize(0, 0, 1) - ChunkBoundary::linearize(0, 0, 0)
    }
}
