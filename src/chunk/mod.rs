use block_mesh::ndshape::{ConstShape, ConstShape3u32};

use crate::voxel::Voxel;

use storage::Storage;

mod position;
mod storage;

pub use position::ChunkPos;

const CHUNK_EDGE: u32 = 16;
type ChunkShape = ConstShape3u32<CHUNK_EDGE, CHUNK_EDGE, CHUNK_EDGE>;

#[derive(Clone)]
pub struct Chunk {
    voxels: Storage,
    change_count: u16,
}

impl Default for Chunk {
    fn default() -> Self {
        Self {
            voxels: Storage::new(ChunkShape::USIZE),
            change_count: 0,
        }
    }
}

#[allow(dead_code)]
impl Chunk {
    pub fn get(&self, x: u32, y: u32, z: u32) -> Voxel {
        self.voxels.get(Self::linearize(x, y, z))
    }

    pub fn set(&mut self, x: u32, y: u32, z: u32, voxel: Voxel) {
        self.voxels.set(Self::linearize(x, y, z), voxel);
        self.change_count += 1;

        if self.change_count > 500 {
            self.voxels.trim();
            self.change_count = 0;
        }
    }

    pub const fn size() -> u32 {
        ChunkShape::SIZE
    }

    pub const fn edge() -> u32 {
        CHUNK_EDGE
    }

    pub fn linearize(x: u32, y: u32, z: u32) -> usize {
        ChunkShape::linearize([x, y, z]) as usize
    }

    pub fn delinearize(idx: u32) -> (u32, u32, u32) {
        let res = ChunkShape::delinearize(idx);
        (res[0], res[1], res[2])
    }
}
