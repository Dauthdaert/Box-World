use super::{
    chunk_boundary::ChunkBoundary,
    face::{Face, FaceWithAO},
    VoxelVisibility,
};

#[derive(Copy, Clone, Debug)]
pub struct Quad {
    pub voxel: [usize; 3],
    pub texture_indice: u32,
    pub width: u32,
    pub height: u32,
}

#[derive(Default)]
pub struct QuadGroups {
    pub groups: [Vec<Quad>; 6],
}

impl QuadGroups {
    pub fn iter(&self) -> impl Iterator<Item = Face> {
        self.groups
            .iter()
            .enumerate()
            .flat_map(|(index, quads)| quads.iter().map(move |quad| (index, quad)))
            .map(|(index, quad)| Face::new(index.into(), quad))
    }

    pub fn iter_with_ao<'a>(
        &'a self,
        chunk: &'a ChunkBoundary,
    ) -> impl Iterator<Item = FaceWithAO<'a>> {
        self.iter().map(|face| FaceWithAO::new(face, chunk))
    }

    /// Returns the total count of quads across all groups.
    pub fn num_quads(&self) -> usize {
        let mut sum = 0;
        for group in self.groups.iter() {
            sum += group.len();
        }
        sum
    }

    pub fn clear(&mut self) {
        self.groups.iter_mut().for_each(|g| g.clear());
    }
}

pub fn generate_quads_with_buffer(
    solid_pass: bool,
    chunk_boundary: &ChunkBoundary,
    buffer: &mut QuadGroups,
) {
    buffer.clear();

    let x_offset = ChunkBoundary::x_offset();
    let y_offset = ChunkBoundary::y_offset();
    let z_offset = ChunkBoundary::z_offset();

    let voxels = chunk_boundary.voxels();
    for z in 1..ChunkBoundary::edge() - 1 {
        for y in 1..ChunkBoundary::edge() - 1 {
            for x in 1..ChunkBoundary::edge() - 1 {
                let idx = ChunkBoundary::linearize(x, y, z);
                let voxel = voxels[idx];

                match voxel.visibility() {
                    VoxelVisibility::Empty => continue,
                    visibility => {
                        let neighbors = [
                            voxels[idx - x_offset],
                            voxels[idx + x_offset],
                            voxels[idx - y_offset],
                            voxels[idx + y_offset],
                            voxels[idx - z_offset],
                            voxels[idx + z_offset],
                        ];

                        for (i, neighbor) in neighbors.into_iter().enumerate() {
                            let other = neighbor.visibility();

                            let generate = if solid_pass {
                                match (visibility, other) {
                                    (VoxelVisibility::Opaque, VoxelVisibility::Empty)
                                    | (VoxelVisibility::Opaque, VoxelVisibility::Transparent) => {
                                        true
                                    }

                                    (
                                        VoxelVisibility::Transparent,
                                        VoxelVisibility::Transparent,
                                    ) => voxel != neighbor,
                                    (_, _) => false,
                                }
                            } else {
                                match (visibility, other) {
                                    (VoxelVisibility::Transparent, VoxelVisibility::Empty) => true,
                                    (
                                        VoxelVisibility::Transparent,
                                        VoxelVisibility::Transparent,
                                    ) => voxel != neighbor,
                                    (_, _) => false,
                                }
                            };

                            if generate {
                                buffer.groups[i].push(Quad {
                                    voxel: [x, y, z],
                                    texture_indice: voxels[idx].indice(),
                                    width: 1,
                                    height: 1,
                                });
                            }
                        }
                    }
                }
            }
        }
    }
}
