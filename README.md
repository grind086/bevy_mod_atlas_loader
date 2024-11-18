# bevy_mod_atlas_loader

This crate provides loaders for [`bevy::sprite`]'s texture atlases that can either load from a single image, or from a `.atlas.ron` file with a list of textures. The plugin also provides an optional asset processor that will automatically compress atlases defined in `.atlas.ron` files to a single image.

## Asset Processing

To enable asset processing enable the [`bevy`] feature `asset_processor` and ensure that your [`AssetPlugin::mode`] is `AssetMode::Processed`. If `file_watcher` is also enabled, changes to assets will be hot reloaded. Note that asset processing currently requires a multi-threaded environment.

[`AssetPlugin::mode`]: bevy::asset::AssetPlugin::mode
