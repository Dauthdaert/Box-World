use bevy::prelude::*;
use bevy_asset_loader::prelude::*;

#[derive(AssetCollection, Resource)]
pub struct HightlightTexture {
    #[asset(path = "textures/outline.png")]
    pub handle: Handle<Image>,
}

#[derive(Component, Clone, Copy)]
pub struct HighlightCube;

pub fn spawn_highlight(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    texture: Res<HightlightTexture>,
) {
    commands.spawn((
        PbrBundle {
            mesh: meshes.add(shape::Cube::new(1.02).into()),
            material: materials.add(StandardMaterial {
                base_color_texture: Some(texture.handle.clone_weak()),
                unlit: true,
                alpha_mode: AlphaMode::Mask(0.5),
                ..default()
            }),
            ..default()
        },
        Name::new("Highlight"),
        HighlightCube,
    ));
}
