use bevy::prelude::*;
use bevy_mod_atlas_loader::AtlasLoaderPlugin;

fn main() {
    App::new()
        .add_plugins((
            DefaultPlugins.set(AssetPlugin {
                // Enable asset processing
                mode: AssetMode::Processed,
                ..default()
            }),
            // Add the plugin
            AtlasLoaderPlugin,
        ))
        .add_systems(Startup, startup)
        .run();
}

// The startup function is unchanged from the basic example
fn startup(mut commands: Commands, assets: Res<AssetServer>) {
    // Create a camera
    commands.spawn(Camera2d);

    // Get the atlas texture and layout
    let atlas_texture = assets.load::<Image>("processed.atlas.ron#texture");
    let atlas_layout = assets.load::<TextureAtlasLayout>("processed.atlas.ron#layout");

    // And spawn a sprite
    commands.spawn(Sprite::from_atlas_image(
        atlas_texture,
        TextureAtlas {
            index: 0,
            layout: atlas_layout,
        },
    ));
}
