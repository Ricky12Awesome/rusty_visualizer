use std::cell::RefCell;
use std::collections::HashMap;
use std::fs::{create_dir, create_dir_all, File};
use std::io::BufReader;
use std::ops::Index;
use std::path::Path;
use std::sync::RwLock;
use image::imageops::FilterType;

use image::{DynamicImage, ImageFormat, RgbaImage};
use macroquad::prelude::Texture2D;

#[derive(Debug, Default)]
pub struct ImageCache {
  /// Holds the base image unmodified
  base_cache: HashMap<ImageCacheType, HashMap<String, DynamicImage>>,
  /// This will replace the image if the size changed using Self::base_cache as the base
  resize_cache: HashMap<ImageCacheType, HashMap<String, RgbaImage>>,
}

#[derive(Debug, Eq, PartialEq, Ord, PartialOrd, Copy, Clone, Hash)]
pub enum ImageCacheType {
  Cover,
  Background,
  Custom(u64),
}

impl ImageCache {
  pub fn set_texture(
    &mut self,
    texture: &mut Texture2D,
    id: String,
    url: &str,
    force: bool,
    cache_type: ImageCacheType,
    (width, height): (Option<u32>, Option<u32>),
  ) {
    match self.get(id, url, cache_type, (width, height)) {
      Some(image) => {
        let width = width.unwrap_or_else(|| image.width());
        let height = height.unwrap_or_else(|| image.height());

        if force || (texture.width() as u32 != width || texture.height() as u32 != height) {
          texture.delete();
          *texture = Texture2D::from_rgba8(width as u16, height as u16, image.as_raw());
        }
      },
      None => {
        if force {
          texture.delete();
          *texture = Texture2D::empty();
        }
      }
    }
  }

  pub fn get_image(id: &str, url: &str, cache_type: ImageCacheType) -> Option<DynamicImage> {
    let path = match cache_type {
      ImageCacheType::Cover => "cover".to_string(),
      ImageCacheType::Background => "background".to_string(),
      ImageCacheType::Custom(id) => format!("custom_{}", id)
    };

    let path = Path::new("./.cache").join(path).join(&format!("{}.jpg", id));

    if path.exists() {
      let file = File::open(path).ok()?;
      let image = image::load(BufReader::new(file), ImageFormat::Jpeg).ok()?;

      Some(image)
    } else {
      let bytes = reqwest::blocking::get(url).ok()?.bytes().ok()?;
      let image = image::load_from_memory(&bytes).ok()?;

      if !path.parent().unwrap().exists() {
        create_dir_all(path.parent().unwrap()).ok()?;
      }

      println!("Saving {}.jpg to {:?}", id, path.parent().unwrap());
      image.save_with_format(path, ImageFormat::Jpeg);

      Some(image)
    }
  }

  pub fn get(
    &mut self,
    id: String,
    url: &str,
    cache_type: ImageCacheType,
    (width, height): (Option<u32>, Option<u32>),
  ) -> Option<&RgbaImage> {
    let base_cache = self.base_cache.entry(cache_type).or_insert_with(HashMap::new);
    let resize_cache = self.resize_cache.entry(cache_type).or_insert_with(HashMap::new);

    // TODO: File caching
    if !base_cache.contains_key(&id) {
      let image = Self::get_image(&id, url, cache_type)?;

      base_cache.insert(id.clone(), image.clone());
      resize_cache.insert(id.clone(), image.to_rgba8());
    }

    let base_image = base_cache.index(&id);
    let resize_image = resize_cache.get_mut(&id).unwrap();

    let width = width.unwrap_or_else(|| resize_image.width());
    let height = height.unwrap_or_else(|| resize_image.height());

    if resize_image.width() != width || resize_image.height() != height {
      *resize_image = base_image.resize_to_fill(width, height, FilterType::Nearest).to_rgba8();
    }

    resize_cache.get(&id)
  }
}