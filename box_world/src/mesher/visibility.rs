use serde::{Deserialize, Serialize};

#[derive(Debug, Copy, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
pub enum VoxelVisibility {
    #[default]
    Empty,
    Transparent,
    Opaque,
}
