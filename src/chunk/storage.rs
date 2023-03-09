use bitvec::prelude::*;

use crate::voxel::Voxel;

#[derive(Clone, Debug)]
pub enum Storage {
    Single(SingleStorage),
    Multi(MultiStorage),
}

#[derive(Clone, Debug)]
pub struct SingleStorage {
    size: usize,
    voxel: Voxel,
}

#[derive(Clone, Debug)]
pub struct MultiStorage {
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

impl MultiStorage {
    fn new(size: usize, initial_voxel: Voxel) -> Self {
        // Indices_length of 2 since this is only used for multiple voxel types
        let indices_length = 2;
        let initial_capacity = 2_usize.pow(indices_length as u32);
        let mut palette = Vec::with_capacity(initial_capacity);
        palette.push(PaletteEntry {
            voxel_type: initial_voxel,
            ref_count: size,
        });

        Self {
            size,
            data: BitBuffer::new(size * indices_length),
            palette,
            palette_capacity: initial_capacity,
            indices_length,
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

impl Storage {
    pub fn new(size: usize) -> Self {
        Self::Single(SingleStorage {
            size,
            voxel: Voxel::Empty,
        })
    }

    fn toggle_storage_type(&mut self) {
        *self = match self {
            Storage::Single(storage) => {
                Storage::Multi(MultiStorage::new(storage.size, storage.voxel))
            }
            Storage::Multi(storage) => {
                assert!(storage.palette.len() == 1);
                Storage::Single(SingleStorage {
                    size: storage.size,
                    voxel: storage.palette[0].voxel_type,
                })
            }
        };
    }

    pub fn set(&mut self, target_idx: usize, voxel: Voxel) {
        match self {
            Storage::Single(storage) => {
                if storage.voxel != voxel {
                    self.toggle_storage_type();
                    self.set(target_idx, voxel);
                }
            }
            Storage::Multi(storage) => {
                let palette_target_idx: usize = storage
                    .data
                    .get(target_idx * storage.indices_length, storage.indices_length);
                if let Some(target) = storage.palette.get_mut(palette_target_idx) {
                    target.ref_count -= 1;
                }

                // Look for voxel palette entry
                let palette_entry_voxel =
                    storage.palette.iter().enumerate().find_map(|(idx, entry)| {
                        if entry.voxel_type == voxel {
                            Some(idx)
                        } else {
                            None
                        }
                    });

                // Voxel type already in palette
                if let Some(idx) = palette_entry_voxel {
                    storage.data.set(
                        target_idx * storage.indices_length,
                        storage.indices_length,
                        idx,
                    );
                    storage
                        .palette
                        .get_mut(idx)
                        .expect("Failed to get palette entry of target voxel")
                        .ref_count += 1;

                    return;
                }

                // Overwrite target palette entry
                if let Some(target) = storage.palette.get_mut(palette_target_idx) {
                    if target.ref_count == 0 {
                        target.voxel_type = voxel;
                        target.ref_count = 1;

                        return;
                    }
                }

                // Create new palette entry
                //bevy::prelude::info!("Creating new voxel entry for {:?}", voxel);
                let new_entry_idx = if let Some((i, entry)) = storage
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
                    if storage.palette.len() == storage.palette_capacity {
                        storage.grow_palette();
                    }

                    storage.palette.push(PaletteEntry {
                        voxel_type: voxel,
                        ref_count: 1,
                    });

                    storage.palette.len() - 1
                };
                storage.data.set(
                    target_idx * storage.indices_length,
                    storage.indices_length,
                    new_entry_idx,
                );
            }
        }
    }

    pub fn get(&self, idx: usize) -> Voxel {
        match self {
            Storage::Single(storage) => storage.voxel,
            Storage::Multi(storage) => {
                let palette_idx: usize = storage
                    .data
                    .get(idx * storage.indices_length, storage.indices_length);

                storage
                    .palette
                    .get(palette_idx)
                    .expect("Failed to get palette entry in voxel get")
                    .voxel_type
            }
        }
    }

    pub fn trim(&mut self) {
        match self {
            Storage::Single(_) => (),
            Storage::Multi(storage) => {
                if storage.palette.len() == 1 {
                    self.toggle_storage_type();
                }
            }
        }
    }
}

#[derive(Clone, Debug)]
struct PaletteEntry {
    voxel_type: Voxel,
    ref_count: usize,
}

#[derive(Clone, Debug)]
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
