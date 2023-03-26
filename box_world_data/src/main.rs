use std::{collections::HashMap, fs, process::Command};

use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize, Clone, Copy)]
enum VoxelType {
    Empty,
    Opaque,
    Transparent,
}

#[derive(Debug, Deserialize, Clone, Copy)]
struct VoxelData {
    pub voxel_type: VoxelType,
    pub texture_id: u32,
    pub emissiveness: Option<u8>,
}

#[derive(Debug, Serialize, Clone, Copy)]
struct FinalVoxelType {
    pub visibility: VoxelType,
    pub texture_id: u16,
    pub emissiveness: u8,
}

impl FinalVoxelType {
    pub fn from_voxel_data(data: VoxelData) -> Self {
        Self {
            visibility: data.voxel_type,
            texture_id: data.texture_id.try_into().unwrap(),
            emissiveness: data.emissiveness.unwrap_or(0),
        }
    }
}

fn main() {
    let mut blocks = HashMap::new();
    for path in fs::read_dir("./assets/data/blocks/").unwrap() {
        let Ok(path) = path else { panic!(); };

        let file_name = path.file_name().into_string().unwrap();
        let voxel_name = file_name.split_once('.').unwrap().0.to_string();

        let content = fs::read_to_string(path.path()).unwrap();
        let voxel_data: VoxelData = ron::from_str(&content).unwrap();

        blocks.insert(voxel_name, voxel_data);
    }

    generate_final_voxel_data(&blocks);
    generate_texture_list(&blocks);
    generate_texture_array();
}

fn generate_final_voxel_data(blocks: &HashMap<String, VoxelData>) {
    for (voxel_name, voxel_data) in blocks.iter() {
        let final_data = FinalVoxelType::from_voxel_data(*voxel_data);
        let mut final_string = ron::to_string(&final_data).unwrap();
        if cfg!(windows) {
            final_string.push_str("\r\n");
        } else {
            final_string.push('\n');
        }

        let final_path = format!("../box_world/assets/data/blocks/{}.voxel.ron", voxel_name);
        fs::write(final_path, final_string).unwrap();
    }
}

fn generate_texture_list(blocks: &HashMap<String, VoxelData>) {
    let mut blocks: Vec<_> = blocks.iter().collect();
    blocks.sort_by_key(|(_name, data)| data.texture_id);

    let mut content = String::new();
    for (voxel_name, _voxel_data) in blocks.into_iter() {
        content.push_str(voxel_name);
        content.push_str(".png");
        if cfg!(windows) {
            content.push_str("\r\n");
        } else {
            content.push('\n');
        }
    }
    fs::write("./assets/textures/textures.txt", content).unwrap();
}

fn generate_texture_array() {
    Command::new("cuttlefish")
        .args([
            "--input-list",
            "array",
            "textures.txt",
            "-m",
            "-f",
            "BC7",
            "-o",
            "terrain_texture.ktx",
        ])
        .current_dir("./assets/textures")
        .spawn()
        .unwrap()
        .wait()
        .unwrap();

    Command::new("ktx2ktx2")
        .args(["-f", "-o", "terrain_texture.ktx2", "terrain_texture.ktx"])
        .current_dir("./assets/textures")
        .spawn()
        .unwrap()
        .wait()
        .unwrap();

    fs::remove_file("./assets/textures/terrain_texture.ktx").unwrap();
    fs::remove_file("./assets/textures/textures.txt").unwrap();

    fs::copy(
        "./assets/textures/terrain_texture.ktx2",
        "../box_world/assets/textures/terrain_texture.ktx2",
    )
    .unwrap();
    fs::remove_file("./assets/textures/terrain_texture.ktx2").unwrap();
}
