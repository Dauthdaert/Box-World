use serde::{Deserialize, Serialize};
use serde_with::{serde_as, Bytes};

use super::ChunkData;

#[inline]
pub fn to_torchlight(value: u8) -> u8 {
    value & 0xFu8
}

#[inline]
pub fn to_sunlight(value: u8) -> u8 {
    (value >> 4) & 0xFu8
}

#[serde_as]
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct LightStorage {
    #[serde_as(as = "Bytes")]
    lights: Box<[u8; ChunkData::usize()]>,
}

impl LightStorage {
    pub fn new() -> Self {
        Self {
            lights: Box::new([0; ChunkData::usize()]),
        }
    }

    /// Output contains both torch and sun light
    pub fn get_light(&self, idx: usize) -> u8 {
        self.lights[idx]
    }

    /// Output is bounded between 0 and 15
    pub fn get_torchlight(&self, idx: usize) -> u8 {
        to_torchlight(self.lights[idx])
    }

    /// Input is bounded between 0 and 15
    pub fn set_torchlight(&mut self, idx: usize, value: u8) {
        debug_assert!(value < 16);

        self.lights[idx] = (self.lights[idx] & 0xF0u8) | value;
    }

    /// Output is bounded between 0 and 15
    pub fn get_sunlight(&self, idx: usize) -> u8 {
        to_sunlight(self.lights[idx])
    }

    /// Input is bounded between 0 and 15
    pub fn set_sunlight(&mut self, idx: usize, value: u8) {
        debug_assert!(value < 16);

        self.lights[idx] = (self.lights[idx] & 0xFu8) | (value << 4);
    }
}
