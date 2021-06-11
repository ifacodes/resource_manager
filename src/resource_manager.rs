use crate::resource::{File, Resource, Texture};
use anyhow::*;
use notify::{watcher, DebouncedEvent, ReadDirectoryChangesWatcher, RecursiveMode, Watcher};
use std::{
    any::{Any, TypeId},
    collections::HashMap,
    path::PathBuf,
    sync::Arc,
    time::Duration,
};
pub struct ResourceManager {
    root: String,
    event_recv: std::sync::mpsc::Receiver<DebouncedEvent>,
    _watcher: ReadDirectoryChangesWatcher,
    resources: HashMap<String, Arc<dyn Resource>>,
}

impl ResourceManager {
    pub fn new(root: &str) -> Self {
        let (tx, rx) = std::sync::mpsc::channel();
        let mut _watcher = watcher(tx, Duration::from_millis(2000)).unwrap();
        _watcher
            .watch("textures", RecursiveMode::Recursive)
            .unwrap();
        Self {
            root: root.to_string(),
            event_recv: rx,
            _watcher,
            resources: HashMap::new(),
        }
    }
    #[allow(dead_code)]
    fn init_generics(&mut self) -> Result<()> {
        Ok(())
    }
    pub fn check_files(&mut self) -> Result<()> {
        match self.event_recv.recv() {
            Ok(event) => match event {
                DebouncedEvent::Write(src) => {
                    println!("{:#?} was edited!", src);
                    // update file and dependencies
                    if self
                        .resources
                        .contains_key(src.file_stem().unwrap().to_str().unwrap())
                    {
                        println!("Contains: {:#?}!", src.file_stem().unwrap());
                        let _ = self.get_texture(src.file_stem().unwrap().to_str().unwrap());
                    }
                }
                DebouncedEvent::Remove(src) => {
                    println!("{:#?} was deleted", src)
                    // remove all from hashmap
                }
                DebouncedEvent::Rename(src, dest) => {
                    println!("{:#?} was renamed {:#?}", src, dest)
                }
                _ => {}
            },
            Err(e) => eprintln!("watch error {:#?}", e),
        }
        /*if TypeId::of::<File>() == root_resource.type_id() {
            //do a thing
        }
        if TypeId::of::<Texture>() == root_resource.type_id() {
            //do a thing
        }*/
        Ok(())
    }
    pub fn get_texture(&mut self, filename: &str) -> Option<Arc<Texture>> {
        // move this to some form of dictionary
        let path_to_file = &format!("{}/textures/{}.png", self.root, filename);
        // 1: is the file loaded?
        if !self.resources.contains_key(path_to_file) {
            // 2: Load the file and create the texture
            let raw_bytes = std::fs::read(path_to_file)
                .map_err(|_| eprintln!("couldn't open file: {}", filename))
                .ok()?;
            let image = image::load_from_memory(&raw_bytes[..])
                .map_err(|e| eprintln!("{}", e))
                .ok()?;
            let buf_image = image
                .as_rgba8()
                .ok_or(anyhow!("unable to get rgba data"))
                .map_err(|e| eprintln!("{}", e))
                .ok()?
                .clone();
            let texture: Arc<dyn Resource> = Arc::new(Texture { diffuse: buf_image });
            let file = File {
                dependency: texture.clone(),
                raw_file: raw_bytes.clone(),
            };
            self.resources
                .insert(format!("file_{}", filename), Arc::new(file));
            self.resources
                .insert(format!("texture_{}", filename), texture);
        }
        // TODO: instead of ok replace the return with a generic texture if not found
        Some(
            self.resources
                .get(&format!("texture_{}", filename))
                .ok_or(anyhow!("no such texture resource!"))
                .map_err(|e| eprintln!("{}", e))
                .ok()?
                .clone()
                .downcast_arc::<Texture>()
                .map_err(|_| "This shouldn't happen...")
                .unwrap(),
        )
    }
}
