//! A minimal example that produces a stack overflow. Overflow only occurs on a dry run (no assets processed) when the
//! `file_watcher` feature is enabled, and the loaded asset `Handle` is not dropped.

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
    let handle = assets.load("my_atlas.atlas.ron");
    // Commenting out the following line (and allowing the handle to drop) prevents the stack overflow
    commands.insert_resource(TileAtlas(handle));
}
