use std::path::Path;
use std::{collections::HashMap, path::PathBuf, sync::Arc};

use sha256::TrySha256Digest;

use crate::runtime::{ColorSpace, Image, Toolbox};
use crate::runtime::{ImageFormat, Runtime};
use crate::session::Session;

#[derive(PartialEq, Eq, Hash, Clone, Debug, serde::Deserialize, serde::Serialize)]
pub enum LibraryImageIdentifier {
    Temp(usize), // images that we no longer have access to after the application closes
    Path(PathBuf),
}

struct LibraryItem {
    image: Option<Arc<Image>>,
    thumbnail: Option<Arc<Image>>,
}

pub struct Library {
    items: HashMap<LibraryImageIdentifier, LibraryItem>,
    items_order: Vec<LibraryImageIdentifier>,
    num_temp_images: usize,
    runtime: Arc<Runtime>,
    toolbox: Arc<Toolbox>,
}

#[derive(Clone, serde::Deserialize, serde::Serialize)]
pub struct LibraryPersistentState {
    pub paths: Vec<PathBuf>,
}

impl LibraryPersistentState {
    pub fn new() -> Self {
        Self { paths: Vec::new() }
    }
}

impl Library {
    pub fn new(runtime: Arc<Runtime>, toolbox: Arc<Toolbox>) -> Self {
        Self {
            items: HashMap::new(),
            items_order: Vec::new(),
            num_temp_images: 0,
            runtime,
            toolbox,
        }
    }

    pub fn num_images_total(&self) -> usize {
        self.items.len() as usize
    }

    pub fn num_temp_images(&self) -> usize {
        self.num_temp_images
    }

    fn add_item(&mut self, item: LibraryItem, identifier: LibraryImageIdentifier) {
        let old_item = self.items.insert(identifier.clone(), item);
        if old_item.is_none() {
            self.items_order.push(identifier);
        }
    }

    fn add_image(&mut self, image: Arc<Image>, identifier: LibraryImageIdentifier) {
        let thumbnail = self.compute_thumbnail(image.clone());
        let image = self
            .toolbox
            .convert_image_format(image, ImageFormat::Rgba16Float);
        let image = self
            .toolbox
            .convert_color_space(image, ColorSpace::LinearRGB);
        let library_item = LibraryItem {
            image: Some(image),
            thumbnail: Some(thumbnail),
        };
        self.add_item(library_item, identifier)
    }

    pub fn add_image_temp(&mut self, image: Arc<Image>) -> LibraryImageIdentifier {
        let temp_image_id = LibraryImageIdentifier::Temp(self.num_temp_images);
        self.num_temp_images += 1;
        self.add_image(image, temp_image_id.clone());
        temp_image_id
    }

    pub fn add_item_from_path(&mut self, path: PathBuf) -> LibraryImageIdentifier {
        let id = LibraryImageIdentifier::Path(path);
        let item = LibraryItem {
            image: None,
            thumbnail: None,
        };
        self.add_item(item, id.clone());
        id
    }

    pub fn get_identifier_at_index(&self, index: usize) -> &LibraryImageIdentifier {
        &self.items_order[index]
    }

    pub fn get_image_at_index(&mut self, index: usize) -> Arc<Image> {
        let identifier = &self.items_order[index];
        self.get_image_from_identifier(&identifier.clone())
    }

    pub fn get_thumbnail_at_index(&mut self, index: usize) -> Arc<Image> {
        let identifier = &self.items_order[index];
        self.get_thumbnail_from_identifier(&identifier.clone())
    }

    fn ensure_loaded(&mut self, identifier: &LibraryImageIdentifier) -> &LibraryItem {
        if self.items[identifier].image.is_none() {
            if let LibraryImageIdentifier::Path(ref path) = identifier {
                let image = self
                    .runtime
                    .create_image_from_path(&path)
                    .expect("failed to create image from path");
                let image = Arc::new(image);
                let image = self
                    .toolbox
                    .convert_image_format(image, ImageFormat::Rgba16Float);
                let image = self
                    .toolbox
                    .convert_color_space(image, ColorSpace::LinearRGB);
                self.items.get_mut(identifier).unwrap().image = Some(image);
            } else {
                panic!("cannot load from a non-path identifier {:?}", identifier);
            }
        }

        if self.items[identifier].thumbnail.is_none() {
            let thumbnail =
                self.compute_thumbnail(self.items[identifier].image.as_ref().unwrap().clone());
            self.items.get_mut(identifier).unwrap().thumbnail = Some(thumbnail);
        }

        &self.items[identifier]
    }

    pub fn get_image_from_identifier(&mut self, identifier: &LibraryImageIdentifier) -> Arc<Image> {
        self.ensure_loaded(identifier);
        self.items[identifier].image.as_ref().unwrap().clone()
    }

    pub fn get_thumbnail_from_identifier(
        &mut self,
        identifier: &LibraryImageIdentifier,
    ) -> Arc<Image> {
        self.ensure_loaded(identifier);
        self.items[identifier].thumbnail.as_ref().unwrap().clone()
    }

    pub fn get_persistent_state(&self) -> LibraryPersistentState {
        let mut paths = Vec::new();
        for pair in self.items.iter() {
            if let LibraryImageIdentifier::Path(ref path) = pair.0 {
                paths.push(path.clone())
            }
        }
        LibraryPersistentState { paths }
    }

    pub fn load_persistent_state(&mut self, state: LibraryPersistentState) {
        for path in state.paths {
            self.add_item_from_path(path);
        }
    }

    fn compute_thumbnail(&self, image: Arc<Image>) -> Arc<Image> {
        let thumbnail_min_dimension_size = 400.0 as f32;
        let factor = thumbnail_min_dimension_size
            / (image.properties.dimensions.0).min(image.properties.dimensions.1) as f32;
        if factor < 0.5 {
            self.toolbox.generate_mipmap(&image);
            let thumbnail = self.toolbox.resize_image(image, factor);
            self.toolbox.generate_mipmap(&thumbnail);
            thumbnail
        } else {
            image
        }
    }

    fn get_thumbnail_path_for_image_path(&self, image_path: &PathBuf) -> Option<PathBuf> {
        if let Ok(digest_str) = image_path.digest() {
            if let Some(storage_dir) = Session::get_persistent_storage_dir() {
                let file_name = digest_str + ".jpg";
                let full_path = storage_dir.join("thumbnails").join(file_name);
                return Some(full_path);
            }
        }
        None
    }
}
