use std::sync::Arc;

use crate::runtime::{Runtime, Toolbox};

use super::thumbnail_generator::ThumbnailGeneratorService;

pub struct Services {
    pub thumbnail_generator: ThumbnailGeneratorService,
}

impl Services {
    pub fn new(runtime: Arc<Runtime>, toolbox: Arc<Toolbox>) -> Self {
        Self {
            thumbnail_generator: ThumbnailGeneratorService::new(runtime, toolbox),
        }
    }
}
