use std::path::{Path, PathBuf};

use bevy::{
    asset::{
        io::{Reader, Writer},
        saver::{AssetSaver, SavedAsset},
        AssetLoader, AssetPath, AsyncWriteExt, LoadContext, LoadDirectError,
    },
    prelude::*,
    render::texture::{ImageFormat, ImageFormatSetting, ImageLoaderSettings},
    sprite::TextureAtlasBuilderError,
};
use serde::{Deserialize, Serialize};
use thiserror::Error;

use super::{TextureAtlasAsset, TextureAtlasPaths};

/// Errors encountered by [`TextureAtlasLoader`].
#[derive(Debug, Error)]
pub enum LoaderError {
    #[error(transparent)]
    Io(#[from] std::io::Error),
    #[error(transparent)]
    LoadDirect(#[from] LoadDirectError),
}

/// Configures the [`TextureAtlasLoader`].
///
/// Note that `LoaderSettings` implements [`From<TextureAtlasLayout>`].
///
/// [`From<TextureAtlasLayout>`]: TextureAtlasLayout
#[derive(Default, Serialize, Deserialize)]
pub struct LoaderSettings {
    /// The format of the atlas image. If this is `None` the format will be auto-detected based on the image's file
    /// extension.
    pub format: Option<ImageFormat>,
    /// The list of textures in the atlas's [`TextureAtlasLayout`]. Textures with an associated [`AssetPath`] can be
    /// looked up in [`TextureAtlasAsset::paths`]
    pub textures: Vec<(Option<AssetPath<'static>>, URect)>,
}

impl LoaderSettings {
    /// Configure the loader to use the given [`ImageFormat`] when loading the atlas image.
    pub fn with_format(self, format: ImageFormat) -> Self {
        Self {
            format: Some(format),
            textures: self.textures,
        }
    }
}

impl From<TextureAtlasLayout> for LoaderSettings {
    fn from(layout: TextureAtlasLayout) -> Self {
        Self {
            format: None,
            textures: layout
                .textures
                .into_iter()
                .map(|rect| (None, rect))
                .collect(),
        }
    }
}

/// An [`AssetLoader`] that loads a [`TextureAtlasAsset`] from a single image file. The layout of the resulting asset
/// must be specified by the loader's [`LoaderSettings`].
pub struct TextureAtlasLoader;

impl AssetLoader for TextureAtlasLoader {
    type Asset = TextureAtlasAsset;
    type Settings = LoaderSettings;
    type Error = LoaderError;

    async fn load(
        &self,
        reader: &mut dyn Reader,
        settings: &Self::Settings,
        load_context: &mut LoadContext<'_>,
    ) -> Result<Self::Asset, Self::Error> {
        let asset_path = load_context.asset_path().clone();
        let internal_asset_path = AssetPath::from(
            asset_path.path().to_path_buf().with_extension(
                settings
                    .format
                    .map_or("bin", |format| format.to_file_extensions()[0]),
            ),
        );
        debug!("Loading atlas with texture: {asset_path}");
        trace!("Loading atlas texture with path: {internal_asset_path}");

        let format = settings.format;
        let texture = load_context
            .loader()
            .immediate()
            .with_reader(reader)
            .with_settings(move |image_settings: &mut ImageLoaderSettings| {
                if let Some(format) = format {
                    trace!("Override image format: {format:?}");
                    image_settings.format = ImageFormatSetting::Format(format);
                }
            })
            .load::<Image>(internal_asset_path)
            .await?;

        trace!(
            "Building texture atlas layout with {} textures",
            settings.textures.len()
        );
        let mut paths = TextureAtlasPaths::default();
        let mut textures = Vec::with_capacity(settings.textures.len());
        for (path, rect) in settings.textures.iter() {
            trace!(
                "Adding sub-texture to layout with index {}: {path:?}",
                paths.path_indices.len()
            );
            paths.add(path.clone());
            textures.push(*rect);
        }

        let layout = TextureAtlasLayout {
            size: texture.get().size(),
            textures,
        };

        debug!(
            "Loaded texture atlas containing {} sub-textures",
            paths.path_indices.len()
        );
        Ok(TextureAtlasAsset {
            layout: load_context.add_labeled_asset("layout".into(), layout),
            texture: load_context.add_loaded_labeled_asset("texture", texture),
            paths,
        })
    }
}

#[derive(Debug, Error)]
pub enum BuildLoaderError {
    #[error(transparent)]
    Io(#[from] std::io::Error),
    #[error(transparent)]
    RonSpannedError(#[from] ron::error::SpannedError),
    #[error(transparent)]
    LoadDirect(#[from] LoadDirectError),
    #[error(transparent)]
    TextureAtlasBuilder(#[from] TextureAtlasBuilderError),
}

pub struct TextureAtlasBuildLoader;

#[derive(Debug, Deserialize)]
struct BuildLoaderConfig {
    textures: Vec<PathBuf>,
}

impl BuildLoaderConfig {
    pub fn iter_paths(&self) -> impl Iterator<Item = &'_ Path> {
        self.textures.iter().map(PathBuf::as_path)
    }
}

impl AssetLoader for TextureAtlasBuildLoader {
    type Asset = TextureAtlasAsset;
    type Settings = ();
    type Error = BuildLoaderError;

    async fn load(
        &self,
        reader: &mut dyn Reader,
        &(): &Self::Settings,
        load_context: &mut LoadContext<'_>,
    ) -> Result<Self::Asset, Self::Error> {
        // Read the configuration .ron file
        debug!("Building texture atlas from {:?}", load_context.path());
        let config = {
            let mut bytes = Vec::new();
            reader.read_to_end(&mut bytes).await?;
            ron::de::from_bytes::<BuildLoaderConfig>(&bytes)?
        };
        trace!(
            "Building texture atlas with {} textures",
            config.textures.len()
        );

        let mut texture_assets = Vec::with_capacity(config.textures.len());
        for path in config.iter_paths() {
            trace!("Loading atlas sub-texture from: {path:?}");
            texture_assets.push(
                load_context
                    .loader()
                    .immediate()
                    .load::<Image>(path)
                    .await?,
            );
        }

        let mut paths = TextureAtlasPaths::default();
        let mut builder = TextureAtlasBuilder::default();
        for (path, texture) in config.iter_paths().zip(texture_assets.iter()) {
            trace!(
                "Adding sub-texture with index {}: {path:?}",
                paths.path_indices.len()
            );
            paths.add(Some(AssetPath::from(path).into_owned()));
            builder.add_texture(None, texture.get());
        }

        trace!("Finalizing atlas");
        let (layout, _, texture) = builder.build()?;

        debug!(
            "Built texture atlas containing {} sub-textures",
            paths.path_indices.len()
        );
        Ok(TextureAtlasAsset {
            layout: load_context.add_labeled_asset("layout".into(), layout),
            texture: load_context.add_labeled_asset("texture".into(), texture),
            paths,
        })
    }

    fn extensions(&self) -> &[&str] {
        &["atlas.ron"]
    }
}

#[derive(Debug, Error)]
pub enum SaverError {
    #[error(transparent)]
    Io(#[from] std::io::Error),
    #[error(transparent)]
    IntoDynamicImage(#[from] IntoDynamicImageError),
    #[error("Unable to save image with format: {0:?}")]
    InvalidImageFormat(ImageFormat),
    #[error(transparent)]
    Image(std::io::Error),
    #[error("Unable to get `TextureAtlasLayout` sub-asset.")]
    MissingLayout,
    #[error("Unable to get `Image` sub-asset.")]
    MissingTexture,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct SaverSettings {
    pub format: ImageFormat,
}

impl Default for SaverSettings {
    fn default() -> Self {
        Self {
            format: ImageFormat::Png,
        }
    }
}

pub struct TextureAtlasSaver;

impl AssetSaver for TextureAtlasSaver {
    type Asset = TextureAtlasAsset;
    type Settings = SaverSettings;
    type OutputLoader = TextureAtlasLoader;
    type Error = SaverError;

    async fn save(
        &self,
        writer: &mut Writer,
        asset: SavedAsset<'_, Self::Asset>,
        settings: &Self::Settings,
    ) -> Result<<Self::OutputLoader as AssetLoader>::Settings, Self::Error> {
        debug!("Exporting texture atlas");
        let paths = &asset.get().paths;
        let layout = asset
            .get_labeled::<TextureAtlasLayout, _>("layout")
            .ok_or(SaverError::MissingLayout)?;
        let texture = asset
            .get_labeled::<Image, _>("texture")
            .ok_or(SaverError::MissingTexture)?;

        trace!(
            "Writing atlas image to buffer ({:?} format)",
            settings.format
        );
        let mut png_buf = Vec::<u8>::new();
        let dyn_image = texture.get().clone().try_into_dynamic()?;
        dyn_image
            .write_to(
                &mut std::io::Cursor::new(&mut png_buf),
                settings
                    .format
                    .as_image_crate_format()
                    .ok_or(SaverError::InvalidImageFormat(settings.format))?,
            )
            .map_err(|err| {
                SaverError::Image(std::io::Error::new(std::io::ErrorKind::Other, err))
            })?;
        writer.write_all(&png_buf).await?;

        debug!("Exported texture atlas");
        Ok(LoaderSettings {
            format: Some(settings.format),
            textures: paths
                .path_indices
                .iter()
                .cloned()
                .zip(layout.textures.iter().cloned())
                .collect(),
        })
    }
}
