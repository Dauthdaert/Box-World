use bevy::{
    pbr::wireframe::{Wireframe, WireframeConfig, WireframePlugin},
    prelude::*,
    render::{
        render_resource::{Extent3d, TextureDimension, TextureFormat},
        settings::{WgpuFeatures, WgpuSettings},
    },
};
use bevy_flycam::{FlyCam, NoCameraPlayerPlugin};
use bevy_inspector_egui::*;
use block_mesh::ndshape::ConstShape;
use chunk::{Chunk, ChunkBoundary};
use mesher::generate_mesh;
use voxel::Voxel;

mod chunk;
mod mesher;
mod voxel;

pub fn app() -> App {
    let mut app = App::new();

    app.add_plugins(DefaultPlugins.set(ImagePlugin::default_nearest()))
        .add_plugin(NoCameraPlayerPlugin);

    #[cfg(debug_assertions)]
    {
        app.insert_resource(WorldInspectorParams {
            enabled: true,
            ..default()
        })
        .add_plugin(WorldInspectorPlugin::new())
        .insert_resource(WgpuSettings {
            features: WgpuFeatures::POLYGON_MODE_LINE,
            ..default()
        })
        .add_plugin(WireframePlugin);
    }

    app.add_startup_system(setup);

    //app.add_system(rotate);

    app
}

#[derive(Component)]
struct Shape;

fn setup(
    mut commands: Commands,
    mut wireframe_config: ResMut<WireframeConfig>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut images: ResMut<Assets<Image>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    wireframe_config.global = false;

    let debug_material = materials.add(StandardMaterial {
        base_color_texture: Some(images.add(uv_debug_texture())),
        ..default()
    });

    let mut chunks = Vec::new();

    for _ in 0..10 {
        let mut chunk = Chunk::default();

        for i in 0..Chunk::size() {
            let (x, y, _z) = Chunk::delinearize(i);

            let voxel = if ((y * x) as f32).sqrt() < 10.0 {
                Voxel::Opaque(1)
            } else {
                Voxel::Empty
            };

            chunk.voxels[i as usize] = voxel;
        }

        chunks.push(chunk);
    }

    let mut boundaries = Vec::new();
    for (i, chunk) in chunks.iter().enumerate() {
        boundaries.push(ChunkBoundary::new(
            chunk.clone(),
            [
                chunks.get(i + 1).cloned().unwrap_or_default(),
                chunks.get(i.wrapping_sub(1)).cloned().unwrap_or_default(),
                Chunk::default(),
                Chunk::default(),
                Chunk::default(),
                Chunk::default(),
            ],
        ));
    }

    for (i, shape) in boundaries.into_iter().enumerate() {
        commands.spawn((
            PbrBundle {
                mesh: meshes.add(generate_mesh(&shape)),
                material: debug_material.clone(),
                transform: Transform::from_xyz(
                    (i * chunk::ChunkShape::ARRAY[0] as usize) as f32,
                    2.0,
                    0.0,
                ),
                ..default()
            },
            Shape,
            Wireframe,
        ));
    }

    commands.spawn(PointLightBundle {
        point_light: PointLight {
            intensity: 9000.0,
            range: 100.,
            shadows_enabled: true,
            ..default()
        },
        transform: Transform::from_xyz(8.0, 16.0, 8.0),
        ..default()
    });

    // ground plane
    commands.spawn(PbrBundle {
        mesh: meshes.add(shape::Plane { size: 50. }.into()),
        material: materials.add(Color::SILVER.into()),
        ..default()
    });

    commands.spawn((
        Camera3dBundle {
            transform: Transform::from_xyz(0.0, 6., 12.0)
                .looking_at(Vec3::new(0., 1., 0.), Vec3::Y),
            ..default()
        },
        FlyCam,
    ));
}

/// Creates a colorful test pattern
fn uv_debug_texture() -> Image {
    const TEXTURE_SIZE: usize = 8;

    let mut palette: [u8; 32] = [
        255, 102, 159, 255, 255, 159, 102, 255, 236, 255, 102, 255, 121, 255, 102, 255, 102, 255,
        198, 255, 102, 198, 255, 255, 121, 102, 255, 255, 236, 102, 255, 255,
    ];

    let mut texture_data = [0; TEXTURE_SIZE * TEXTURE_SIZE * 4];
    for y in 0..TEXTURE_SIZE {
        let offset = TEXTURE_SIZE * y * 4;
        texture_data[offset..(offset + TEXTURE_SIZE * 4)].copy_from_slice(&palette);
        palette.rotate_right(4);
    }

    Image::new_fill(
        Extent3d {
            width: TEXTURE_SIZE as u32,
            height: TEXTURE_SIZE as u32,
            depth_or_array_layers: 1,
        },
        TextureDimension::D2,
        &texture_data,
        TextureFormat::Rgba8UnormSrgb,
    )
}
