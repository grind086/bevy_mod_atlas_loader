# bevy_mod_atlas_loader

This crate provides loaders for [`bevy_sprite`]'s texture atlases that can either load from a single image, or from a `.atlas.ron` file with a list of textures. The plugin also provides an optional asset processor that will automatically compress atlases defined in `.atlas.ron` files to a single image.

## Example

```rust
use bevy::prelude::*;
use bevy_mod_atlas_loader::AtlasLoaderPlugin;

fn main() {
    App::new()
        .add_plugins((
            DefaultPlugins,
            // Add the plugin
            AtlasLoaderPlugin,
        ))
        .add_systems(Startup, startup)
        .run();
}

fn startup(mut commands: Commands, assets: Res<AssetServer>) {
    // Create a camera
    commands.spawn(Camera2d);

    // Get the atlas texture and layout
    let atlas_texture = assets.load::<Image>("my_atlas.atlas.ron#texture");
    let atlas_layout = assets.load::<TextureAtlasLayout>("my_atlas.atlas.ron#layout");

    // And spawn a sprite
    commands.spawn(Sprite::from_atlas_image(
        atlas_texture,
        TextureAtlas {
            index: 0,
            layout: atlas_layout,
        },
    ));
}
```

## Asset Processing

To enable asset processing enable the [`bevy`] feature `asset_processor` and ensure that your [`AssetPlugin::mode`] is `AssetMode::Processed`. If `file_watcher` is also enabled, changes to assets will be hot reloaded. Note that asset processing currently requires a multi-threaded environment.

[`bevy_sprite`]: bevy::sprite
[`AssetPlugin::mode`]: bevy::asset::AssetPlugin::mode
