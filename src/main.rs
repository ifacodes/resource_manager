use anyhow::*;
use std::{collections::HashMap, sync::Arc};

use image::RgbaImage;

struct ResourceManager<T> {
    resources: HashMap<String, Arc<T>>,
}

impl<T> ResourceManager<T> {
    fn new() -> Self {
        Self {
            resources: HashMap::new(),
        }
    }
    fn get(&self, resource: &str) -> Option<&Arc<T>> {
        self.resources.get(resource)
    }
}

struct Texture {
    texture: RgbaImage,
}

fn load_resources<T, F>(
    manager: &mut ResourceManager<T>,
    dir: &str,
    load_resource: &F,
) -> Result<()>
where
    F: Fn(&str) -> Result<T>,
{
    let dir = std::fs::read_dir(dir)?;
    for entry in dir {
        let path = entry?.path();
        let path_str = path.to_string_lossy().to_string();
        if path.is_dir() {
            load_resources(manager, &path_str, load_resource)?;
        } else if path.is_file() {
            let id = path
                .file_stem()
                .ok_or(anyhow!("Unable to retrieve file name as string"))?
                .to_string_lossy()
                .to_string();
            let item: Arc<T> = Arc::new(load_resource(&path_str)?);
            manager.resources.insert(id, item);
        }
    }
    Ok(())
}
fn main() {
    let pwd = std::env::current_dir().unwrap();
    println!("{}", pwd.display());
    let mut rm: ResourceManager<Texture> = ResourceManager::new();
    load_resources(&mut rm, ".\\textures", &|file_path| {
        let bytes = std::fs::read(file_path).unwrap();
        let image = image::load_from_memory(&bytes[..]).unwrap();
        let texture = image.as_rgba8().unwrap().clone();
        Ok(Texture { texture })
    })
    .unwrap();
    let bytes = std::fs::read(".\\textures\\happy-tree.png").unwrap();
    let image = image::load_from_memory(&bytes[..]).unwrap();
    let texture = image.as_rgba8().unwrap().clone();
    let item = rm.get("happy-tree").unwrap();
    assert_eq!(texture, item.texture);
}
