use block_mesh::ndshape::{ConstShape, ConstShape3u32};

use crate::voxel::Voxel;

pub const CHUNK_EDGE: u32 = 16;
pub type ChunkShape = ConstShape3u32<CHUNK_EDGE, CHUNK_EDGE, CHUNK_EDGE>;

const BOUNDARY_EDGE: u32 = CHUNK_EDGE + 2;
pub type BoundaryShape = ConstShape3u32<BOUNDARY_EDGE, BOUNDARY_EDGE, BOUNDARY_EDGE>;

#[derive(Clone)]
pub struct Chunk {
    pub voxels: Box<[Voxel; ChunkShape::USIZE]>,
}

impl Default for Chunk {
    fn default() -> Self {
        Self {
            voxels: Box::new([Voxel::default(); ChunkShape::USIZE]),
        }
    }
}

#[allow(dead_code)]
impl Chunk {
    pub fn get(&self, x: u32, y: u32, z: u32) -> Voxel {
        self.voxels[ChunkShape::linearize([x, y, z]) as usize]
    }

    pub fn size() -> u32 {
        ChunkShape::SIZE
    }

    pub fn linearize(x: u32, y: u32, z: u32) -> u32 {
        ChunkShape::linearize([x, y, z])
    }

    pub fn delinearize(idx: u32) -> (u32, u32, u32) {
        let res = ChunkShape::delinearize(idx);
        (res[0], res[1], res[2])
    }
}

pub struct ChunkBoundary {
    voxels: Box<[Voxel; BoundaryShape::USIZE]>,
}

#[allow(dead_code)]
impl ChunkBoundary {
    pub fn new(center: Chunk, neighbors: [Chunk; 6]) -> Self {
        const MAX: u32 = CHUNK_EDGE;
        const BOUND: u32 = MAX + 1;

        let voxels: [Voxel; BoundaryShape::USIZE] = (0..BoundaryShape::SIZE)
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
            .collect::<Vec<Voxel>>()
            .try_into()
            .unwrap();

        Self {
            voxels: Box::new(voxels),
        }
    }

    pub fn voxels(&self) -> &[Voxel] {
        self.voxels.as_slice()
    }

    pub fn size() -> u32 {
        BoundaryShape::SIZE
    }

    pub fn linearize(x: u32, y: u32, z: u32) -> u32 {
        BoundaryShape::linearize([x, y, z])
    }

    pub fn delinearize(idx: u32) -> (u32, u32, u32) {
        let res = BoundaryShape::delinearize(idx);
        (res[0], res[1], res[2])
    }
}
