use anyhow::*;
use std::{collections::HashMap, path::PathBuf, sync::Arc};

use image::RgbaImage;

struct ResourceManager<T> {
    resource_dir: PathBuf,
    resources: HashMap<String, Arc<T>>,
}

impl<T> ResourceManager<T>
where
    T: Resource,
{
    fn new(dir: &str) -> Self {
        Self {
            resource_dir: PathBuf::from(dir),
            resources: HashMap::new(),
        }
    }
    fn get(&self, resource: &str) -> Option<&Arc<T>> {
        self.resources.get(resource).clone()
    }

    fn load_dir(&mut self, dir: String) -> Result<()> {
        let dir = std::fs::read_dir(dir)?;
        for entry in dir {
            let path = entry?.path();
            let path_str = path.to_string_lossy().to_string();
            if path.is_dir() {
                self.load_dir(path_str)?;
            } else if path.is_file() {
                let id = path
                    .file_stem()
                    .ok_or(anyhow!("Unable to retrieve file name as string"))?
                    .to_string_lossy()
                    .to_string();
                let item: Arc<T> = Arc::new(T::load_resource(&path_str)?);
                self.resources.insert(id, item);
            }
        }
        Ok(())
    }
    fn load_resources(&mut self) -> Result<()> {
        self.load_dir(self.resource_dir.to_string_lossy().to_string())
    }
}

trait Resource {
    fn load_resource(res_path: &str) -> Result<Self>
    where
        Self: Sized;
}

struct Texture {
    texture: RgbaImage,
}

impl Resource for Texture {
    fn load_resource(res_path: &str) -> Result<Self> {
        let bytes = std::fs::read(res_path).unwrap();
        let image = image::load_from_memory(&bytes[..]).unwrap();
        let texture = image.as_rgba8().unwrap().clone();
        Ok(Texture { texture })
    }
}

fn main() {
    let mut rm: ResourceManager<Texture> = ResourceManager::new("textures");
    println!("{:#?}", rm.resource_dir);
    rm.load_resources().unwrap();
    let bytes = std::fs::read(".\\textures\\happy-tree.png").unwrap();
    let image = image::load_from_memory(&bytes[..]).unwrap();
    let texture = image.as_rgba8().unwrap().clone();
    let item = rm.get("happy-tree").unwrap();
    assert_eq!(texture, item.texture);
}
