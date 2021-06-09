use crate::resource::{File, Resource, Texture};
use anyhow::*;
use std::{collections::HashMap, path::PathBuf, str::FromStr, sync::Arc};
use uuid::Uuid;
pub struct ResourceManager {
    resources: HashMap<String, Arc<dyn Resource>>,
}

impl ResourceManager {
    pub fn new() -> Self {
        Self {
            resources: HashMap::new(),
        }
    }
    pub fn get_texture(&mut self, path_to_file: &str) -> Result<Arc<Texture>> {
        // 1: is the file loaded?
        if !self.resources.contains_key(path_to_file) {
            // 2: Load the file and create the texture
            let raw_bytes = std::fs::read(path_to_file).unwrap();
            let file = File {
                path_to_file: PathBuf::from_str(path_to_file).unwrap(),
                raw_file: raw_bytes.clone(),
            };
            let id = Uuid::new_v4();
            self.resources.insert(id.to_string(), Arc::new(file));
            let image = image::load_from_memory(&raw_bytes[..]).unwrap();
            let buf_image = image.as_rgba8().unwrap().clone();
            let texture: Arc<dyn Resource> = Arc::new(Texture {
                diffuse: buf_image,
                file: id,
            });
            self.resources.insert(path_to_file.to_string(), texture);
        }
        Ok(self
            .resources
            .get(path_to_file)
            .unwrap()
            .clone()
            .downcast_arc::<Texture>()
            .map_err(|_| "This shouldn't happen...")
            .unwrap())
    }
    #[allow(dead_code)]
    pub fn exist(&self, name: &str) {
        println!("{}", self.resources.contains_key(name));
    }
}
