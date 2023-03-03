use bitvec::prelude::*;
use block_mesh::ndshape::{ConstShape, ConstShape3u32};

use crate::voxel::Voxel;

pub const CHUNK_EDGE: u32 = 16;
pub type ChunkShape = ConstShape3u32<CHUNK_EDGE, CHUNK_EDGE, CHUNK_EDGE>;

#[derive(Clone)]
pub struct Chunk {
    voxels: Storage,
}

impl Default for Chunk {
    fn default() -> Self {
        Self {
            voxels: Storage::new(ChunkShape::USIZE),
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
    }

    pub fn size() -> u32 {
        ChunkShape::SIZE
    }

    pub fn linearize(x: u32, y: u32, z: u32) -> usize {
        ChunkShape::linearize([x, y, z]) as usize
    }

    pub fn delinearize(idx: u32) -> (u32, u32, u32) {
        let res = ChunkShape::delinearize(idx);
        (res[0], res[1], res[2])
    }
}

#[derive(Clone)]
struct Storage {
    /// Size of chunk storage, in voxels
    size: usize,
    data: BitBuffer,
    palette: Vec<PaletteEntry>,
    /// Palette capacity given size of indices
    /// Not necessarily equal to palette vector capacity
    palette_capacity: usize,
    /// Bit length of indices into the palette
    indices_length: usize,
}

impl Storage {
    pub fn new(size: usize) -> Self {
        let indices_length = 1;
        let initial_capacity = 2_usize.pow(indices_length as u32);
        Self {
            size,
            data: BitBuffer::new(size * indices_length),
            palette: Vec::with_capacity(initial_capacity),
            palette_capacity: initial_capacity,
            indices_length,
        }
    }

    pub fn set(&mut self, target_idx: usize, voxel: Voxel) {
        let palette_target_idx: usize = self
            .data
            .get(target_idx * self.indices_length, self.indices_length);
        if let Some(target) = self.palette.get_mut(palette_target_idx) {
            target.ref_count -= 1;
        }

        // Look for voxel palette entry
        let palette_entry_voxel = self.palette.iter().enumerate().find_map(|(idx, entry)| {
            if entry.voxel_type == voxel {
                Some(idx)
            } else {
                None
            }
        });

        // Voxel type already in palette
        if let Some(idx) = palette_entry_voxel {
            self.data
                .set(target_idx * self.indices_length, self.indices_length, idx);
            self.palette
                .get_mut(idx)
                .expect("Failed to get palette entry of target voxel")
                .ref_count += 1;

            return;
        }

        // Overwrite target palette entry
        if let Some(target) = self.palette.get_mut(palette_target_idx) {
            if target.ref_count == 0 {
                target.voxel_type = voxel;
                target.ref_count = 1;

                return;
            }
        }

        // Create new palette entry
        let new_entry_idx = if let Some((i, entry)) = self
            .palette
            .iter_mut()
            .enumerate()
            .find(|(_i, entry)| entry.ref_count == 0)
        {
            // Recycle a ref_count 0 entry if any exists
            entry.voxel_type = voxel;
            entry.ref_count = 1;

            i
        } else {
            // Create a new entry from scratch
            if self.palette.len() == self.palette_capacity {
                self.grow_palette();
            }

            self.palette.push(PaletteEntry {
                voxel_type: voxel,
                ref_count: 1,
            });

            self.palette.len() - 1
        };
        self.data.set(
            target_idx * self.indices_length,
            self.indices_length,
            new_entry_idx,
        );
    }

    fn get(&self, idx: usize) -> Voxel {
        let palette_idx: usize = self
            .data
            .get(idx * self.indices_length, self.indices_length);

        if let Some(entry) = self.palette.get(palette_idx) {
            entry.voxel_type
        } else {
            // If the cube is empty, return default Voxel
            Voxel::default()
        }
    }

    fn grow_palette(&mut self) {
        let mut indices: Vec<usize> = Vec::with_capacity(self.size);
        for i in 0..self.size {
            indices.push(self.data.get(i * self.indices_length, self.indices_length));
        }

        self.indices_length <<= 1;
        let new_capacity = 2usize.pow(self.indices_length as u32);
        self.palette.reserve(new_capacity - self.palette_capacity);
        self.palette_capacity = new_capacity;

        self.data = BitBuffer::new(self.size * self.indices_length);

        for (i, idx) in indices.into_iter().enumerate() {
            self.data
                .set(i * self.indices_length, self.indices_length, idx);
        }
    }
}

#[derive(Clone)]
struct PaletteEntry {
    voxel_type: Voxel,
    ref_count: u32,
}

#[derive(Clone)]
struct BitBuffer {
    bytes: BitVec<u8, Lsb0>,
}

impl BitBuffer {
    /// Create a new BitBuffer. size is specified in bits, not bytes.
    fn new(size: usize) -> Self {
        Self {
            bytes: BitVec::repeat(false, size),
        }
    }

    fn set(&mut self, idx: usize, bit_length: usize, bits: usize) {
        self.bytes[idx..idx + bit_length].store_le::<usize>(bits);
    }

    fn get(&self, idx: usize, bit_length: usize) -> usize {
        self.bytes[idx..idx + bit_length].load_le::<usize>()
    }
}
