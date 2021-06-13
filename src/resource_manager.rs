use crate::resource::{File, Resource, ResourceKey, Texture};
use anyhow::*;
use log::{debug, error};
use notify::{watcher, DebouncedEvent, ReadDirectoryChangesWatcher, RecursiveMode, Watcher};
use std::{
    collections::HashMap,
    path::{self, Path, PathBuf},
    sync::Arc,
    time::Duration,
};

pub struct ResourceManager {
    root: PathBuf,
    event_recv: std::sync::mpsc::Receiver<DebouncedEvent>,
    _watcher: ReadDirectoryChangesWatcher,
    file_map: HashMap<String, PathBuf>,
    resources: HashMap<ResourceKey, Arc<dyn Resource>>,
}

impl ResourceManager {
    //
    //  This section is for initializing the ResourceManager
    //
    pub fn new() -> Self {
        let (tx, rx) = std::sync::mpsc::channel();
        let mut _watcher = watcher(tx, Duration::from_millis(2000)).unwrap();
        _watcher
            .watch("textures", RecursiveMode::Recursive)
            .unwrap();
        let file = std::fs::File::open("./resources.json").unwrap();
        let buffer = std::io::BufReader::new(file);
        let file_map: HashMap<String, PathBuf> = serde_json::from_reader(buffer).unwrap();
        Self {
            root: std::env::current_dir().unwrap(),
            event_recv: rx,
            _watcher,
            file_map,
            resources: HashMap::new(),
        }
    }
    #[allow(dead_code)]
    fn init_generics(&mut self) -> Result<()> {
        Ok(())
    }
    //
    //  This section is for handling updates to the Resources
    //
    pub fn check_files(&mut self) -> Result<()> {
        match self.event_recv.recv() {
            Ok(event) => match event {
                DebouncedEvent::Write(src) => {
                    let relative_path = src.strip_prefix(self.root.clone())?;
                    println!("{:#?} was edited!", relative_path);
                    // update file and dependencies
                }
                DebouncedEvent::Remove(src) => {
                    println!("{:#?} was deleted", src);
                }
                DebouncedEvent::Rename(src, dest) => {
                    println!("{:#?} was renamed {:#?}", src, dest)
                    // this won't be handled for now
                }
                _ => {}
            },
            Err(e) => eprintln!("watch error {:#?}", e),
        }
        Ok(())
    }
    //
    //  This section is for getting Resources from the Manager
    //
    pub fn get_texture(&mut self, filename: &str) -> Result<Arc<Texture>> {
        let mut filename = filename;

        if !self.file_map.contains_key(filename) {
            filename = "error-texture";
        }
        let path_to_file = self.get_abs_file_path(filename);

        if !self
            .resources
            .contains_key(&ResourceKey::Texture(filename.to_string()))
        {
            // 2: Load the file and create the texture
            println!("{:#?}", path_to_file);
            let mut read_bytes = std::fs::read(path_to_file.clone());
            if read_bytes.is_err() {
                error!("unable to find file: {:#?}", path_to_file);
                read_bytes = std::fs::read(self.get_abs_file_path("error-texture"));
                debug!(
                    "loaded file: {:#?}",
                    self.get_abs_file_path("error-texture")
                );
            } else {
                debug!("loaded file: {:#?}", path_to_file);
            }
            let mut file = File {
                dependency: None,
                data: read_bytes.unwrap(),
            };
            let texture: Arc<dyn Resource> =
                Arc::new(self.load_texture_from_raw(file.data.clone())?);
            file.dependency = Some(filename.to_string());
            self.resources
                .insert(ResourceKey::File(filename.to_string()), Arc::new(file));
            debug!("stored file resource for: {:#?}", filename);
            self.resources
                .insert(ResourceKey::Texture(filename.to_string()), texture);
            debug!("stored texture resource for: {:#?}", filename);
        }
        Ok(self
            .resources
            .get(&ResourceKey::Texture(filename.to_string()))
            .ok_or(anyhow!("no such texture resource!"))?
            .clone()
            .downcast_arc::<Texture>()
            .map_err(|_| "This shouldn't happen...")
            .unwrap())
    }
    //
    //  This section is for helper functions and things that shouldn't be public
    //
    #[allow(dead_code)]
    fn update_dep_chain(&mut self, res: String, prev_res: Arc<dyn Resource>) -> Result<()> {
        Ok(())
    }

    fn load_texture_from_raw(&mut self, raw_bytes: Vec<u8>) -> Result<Texture> {
        let image = image::load_from_memory(&raw_bytes[..])?;
        let buf_image = image
            .as_rgba8()
            .ok_or(anyhow!("unable to get rgba data"))?
            .clone();
        Ok(Texture { data: buf_image })
    }
    /// Update A File Resource
    fn update_file() {}
    /// Update A Texture Resource
    fn update_texture() {}

    fn get_abs_file_path(&self, filename: &str) -> PathBuf {
        let mut abs = self.root.clone();
        abs.push(self.file_map.get(filename).unwrap());
        abs
    }
}
