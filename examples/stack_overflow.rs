use bevy::{
    log::{Level, LogPlugin},
    prelude::*,
};
use bevy_mod_atlas_loader::{AtlasLoaderPlugin, TextureAtlasAsset};

fn main() {
    App::new()
        .add_plugins((
            DefaultPlugins
                .set(LogPlugin {
                    level: Level::TRACE,
                    ..default()
                })
                .set(AssetPlugin {
                    mode: AssetMode::Processed,
                    ..default()
                }),
            AtlasLoaderPlugin,
        ))
        .add_systems(Startup, startup)
        .run();
}

#[derive(Resource, Deref)]
struct TileAtlas(Handle<TextureAtlasAsset>);

fn startup(mut commands: Commands, assets: Res<AssetServer>) {
    commands.insert_resource(TileAtlas(assets.load("my_atlas.atlas.ron")));
}
