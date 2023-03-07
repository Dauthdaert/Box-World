use std::ops::Deref;

use crate::voxel::Voxel;

use super::{
    chunk_boundary::ChunkBoundary,
    quads::Quad,
    side::{Axis, Side},
    VoxelVisibility,
};

pub struct Face<'a> {
    side: Side,
    quad: &'a Quad,
}

#[allow(dead_code)]
impl<'a> Face<'a> {
    pub fn new(side: Side, quad: &'a Quad) -> Self {
        Self { side, quad }
    }

    pub fn side(&self) -> Side {
        self.side
    }

    pub fn indices(&self, start: u32) -> [u32; 6] {
        [start, start + 2, start + 1, start + 1, start + 2, start + 3]
    }

    pub fn positions(&self, voxel_size: f32) -> [[f32; 3]; 4] {
        let positions = match (&self.side.axis, &self.side.positive) {
            (Axis::X, false) => [
                [0.0, 0.0, 1.0],
                [0.0, 0.0, 0.0],
                [0.0, 1.0, 1.0],
                [0.0, 1.0, 0.0],
            ],
            (Axis::X, true) => [
                [1.0, 0.0, 0.0],
                [1.0, 0.0, 1.0],
                [1.0, 1.0, 0.0],
                [1.0, 1.0, 1.0],
            ],
            (Axis::Y, false) => [
                [0.0, 0.0, 1.0],
                [1.0, 0.0, 1.0],
                [0.0, 0.0, 0.0],
                [1.0, 0.0, 0.0],
            ],
            (Axis::Y, true) => [
                [0.0, 1.0, 1.0],
                [0.0, 1.0, 0.0],
                [1.0, 1.0, 1.0],
                [1.0, 1.0, 0.0],
            ],
            (Axis::Z, false) => [
                [0.0, 0.0, 0.0],
                [1.0, 0.0, 0.0],
                [0.0, 1.0, 0.0],
                [1.0, 1.0, 0.0],
            ],
            (Axis::Z, true) => [
                [1.0, 0.0, 1.0],
                [0.0, 0.0, 1.0],
                [1.0, 1.0, 1.0],
                [0.0, 1.0, 1.0],
            ],
        };

        let (x, y, z) = (
            (self.quad.voxel[0] - 1) as f32,
            (self.quad.voxel[1] - 1) as f32,
            (self.quad.voxel[2] - 1) as f32,
        );

        [
            [
                x * voxel_size + positions[0][0] * voxel_size,
                y * voxel_size + positions[0][1] * voxel_size,
                z * voxel_size + positions[0][2] * voxel_size,
            ],
            [
                x * voxel_size + positions[1][0] * voxel_size,
                y * voxel_size + positions[1][1] * voxel_size,
                z * voxel_size + positions[1][2] * voxel_size,
            ],
            [
                x * voxel_size + positions[2][0] * voxel_size,
                y * voxel_size + positions[2][1] * voxel_size,
                z * voxel_size + positions[2][2] * voxel_size,
            ],
            [
                x * voxel_size + positions[3][0] * voxel_size,
                y * voxel_size + positions[3][1] * voxel_size,
                z * voxel_size + positions[3][2] * voxel_size,
            ],
        ]
    }

    pub fn normals(&self) -> [[f32; 3]; 4] {
        self.side.normals()
    }

    pub fn uvs(&self, flip_u: bool, flip_v: bool) -> [[f32; 2]; 4] {
        match (flip_u, flip_v) {
            (true, true) => [[1.0, 1.0], [0.0, 1.0], [1.0, 0.0], [0.0, 0.0]],
            (true, false) => [[1.0, 0.0], [0.0, 0.0], [1.0, 1.0], [0.0, 1.0]],
            (false, true) => [[0.0, 1.0], [1.0, 1.0], [0.0, 0.0], [1.0, 0.0]],
            (false, false) => [[0.0, 0.0], [1.0, 0.0], [0.0, 1.0], [1.0, 1.0]],
        }
    }

    pub fn voxel(&self) -> [usize; 3] {
        self.quad.voxel
    }
}

pub struct FaceWithAO<'a> {
    face: Face<'a>,
    aos: [u32; 4],
}

impl<'a> Deref for FaceWithAO<'a> {
    type Target = Face<'a>;

    fn deref(&self) -> &Self::Target {
        &self.face
    }
}

impl<'a> FaceWithAO<'a> {
    pub fn new(face: Face<'a>, chunk: &ChunkBoundary) -> Self {
        let aos = face_aos(&face, chunk);
        Self { face, aos }
    }

    pub fn indices(&self, start: u32) -> [u32; 6] {
        let aos = self.aos();

        if aos[1] + aos[2] > aos[0] + aos[3] {
            [start, start + 2, start + 1, start + 1, start + 2, start + 3]
        } else {
            [start, start + 3, start + 1, start, start + 2, start + 3]
        }
    }

    pub fn aos(&self) -> [u32; 4] {
        self.aos
    }
}

fn face_aos(face: &Face, chunk: &ChunkBoundary) -> [u32; 4] {
    let [x, y, z] = face.voxel();
    let idx = ChunkBoundary::linearize(x, y, z);

    let x_offset = ChunkBoundary::x_offset();
    let y_offset = ChunkBoundary::y_offset();
    let z_offset = ChunkBoundary::z_offset();

    let voxels = chunk.voxels();
    match face.side() {
        Side::X_NEG => side_aos([
            voxels[idx - x_offset + z_offset],
            voxels[idx - x_offset - y_offset + z_offset],
            voxels[idx - x_offset - y_offset],
            voxels[idx - x_offset - y_offset - z_offset],
            voxels[idx - x_offset - z_offset],
            voxels[idx - x_offset + y_offset - z_offset],
            voxels[idx - x_offset + y_offset],
            voxels[idx - x_offset + y_offset + z_offset],
        ]),
        Side::X_POS => side_aos([
            voxels[idx + x_offset - z_offset],
            voxels[idx + x_offset - y_offset - z_offset],
            voxels[idx + x_offset - y_offset],
            voxels[idx + x_offset - y_offset + z_offset],
            voxels[idx + x_offset + z_offset],
            voxels[idx + x_offset + y_offset + z_offset],
            voxels[idx + x_offset + y_offset],
            voxels[idx + x_offset + y_offset - z_offset],
        ]),
        Side::Y_NEG => side_aos([
            voxels[idx - x_offset - y_offset],
            voxels[idx - x_offset - y_offset + z_offset],
            voxels[idx - y_offset + z_offset],
            voxels[idx + x_offset - y_offset + z_offset],
            voxels[idx + x_offset - y_offset],
            voxels[idx + x_offset - y_offset - z_offset],
            voxels[idx - y_offset - z_offset],
            voxels[idx - x_offset - y_offset - z_offset],
        ]),
        Side::Y_POS => side_aos([
            voxels[idx + y_offset + z_offset],
            voxels[idx - x_offset + y_offset + z_offset],
            voxels[idx - x_offset + y_offset],
            voxels[idx - x_offset + y_offset - z_offset],
            voxels[idx + y_offset - z_offset],
            voxels[idx + x_offset + y_offset - z_offset],
            voxels[idx + x_offset + y_offset],
            voxels[idx + x_offset + y_offset + z_offset],
        ]),
        Side::Z_NEG => side_aos([
            voxels[idx - x_offset - z_offset],
            voxels[idx - x_offset - y_offset - z_offset],
            voxels[idx - y_offset - z_offset],
            voxels[idx + x_offset - y_offset - z_offset],
            voxels[idx + x_offset - z_offset],
            voxels[idx + x_offset + y_offset - z_offset],
            voxels[idx + y_offset - z_offset],
            voxels[idx - x_offset + y_offset - z_offset],
        ]),
        Side::Z_POS => side_aos([
            voxels[idx + x_offset + z_offset],
            voxels[idx + x_offset - y_offset + z_offset],
            voxels[idx - y_offset + z_offset],
            voxels[idx - x_offset - y_offset + z_offset],
            voxels[idx - x_offset + z_offset],
            voxels[idx - x_offset + y_offset + z_offset],
            voxels[idx + y_offset + z_offset],
            voxels[idx + x_offset + y_offset + z_offset],
        ]),
    }
}

fn side_aos(neighbors: [Voxel; 8]) -> [u32; 4] {
    let ns = [
        neighbors[0].visibility() == VoxelVisibility::Opaque,
        neighbors[1].visibility() == VoxelVisibility::Opaque,
        neighbors[2].visibility() == VoxelVisibility::Opaque,
        neighbors[3].visibility() == VoxelVisibility::Opaque,
        neighbors[4].visibility() == VoxelVisibility::Opaque,
        neighbors[5].visibility() == VoxelVisibility::Opaque,
        neighbors[6].visibility() == VoxelVisibility::Opaque,
        neighbors[7].visibility() == VoxelVisibility::Opaque,
    ];

    [
        ao_value(ns[0], ns[1], ns[2]),
        ao_value(ns[2], ns[3], ns[4]),
        ao_value(ns[6], ns[7], ns[0]),
        ao_value(ns[4], ns[5], ns[6]),
    ]
}

// true if OPAQUE, otherwise false
fn ao_value(side1: bool, corner: bool, side2: bool) -> u32 {
    match (side1, corner, side2) {
        (true, _, true) => 0,
        (true, true, false) | (false, true, true) => 1,
        (false, false, false) => 3,
        _ => 2,
    }
}
