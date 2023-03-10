cuttlefish --input-list array textures.txt -m -f BC7 -o terrain_texture.ktx
ktx2ktx2 -f -o terrain_texture.ktx2 terrain_texture.ktx
del terrain_texture.ktx
