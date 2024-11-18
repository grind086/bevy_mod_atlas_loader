use bevy::prelude::*;
use bevy_mod_atlas_loader::AtlasLoaderPlugin;

fn main() {
    App::new()
        .add_plugins((
            DefaultPlugins.set(AssetPlugin {
                mode: AssetMode::Processed,
                ..default()
            }),
            AtlasLoaderPlugin,
        ))
        .add_systems(Startup, startup)
        .run();
}

fn startup(mut commands: Commands, assets: Res<AssetServer>) {
    commands.spawn(Camera2d);

    let atlas_texture = assets.load::<Image>("my_atlas.atlas.ron#texture");
    let atlas_layout = assets.load::<TextureAtlasLayout>("my_atlas.atlas.ron#layout");
    commands.spawn(Sprite::from_atlas_image(
        atlas_texture,
        TextureAtlas {
            index: 0,
            layout: atlas_layout,
        },
    ));
}
