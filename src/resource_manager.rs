use crate::resource::{File, InnerFile, Resource, ResourceKey, Texture};
use anyhow::*;
use log::{debug, error};
use notify::{watcher, DebouncedEvent, ReadDirectoryChangesWatcher, RecursiveMode, Watcher};
use std::{
    collections::HashMap,
    path::PathBuf,
    sync::{Arc, RwLock},
    time::Duration,
};

pub struct ResourceManager {
    root: PathBuf,
    event_recv: std::sync::mpsc::Receiver<DebouncedEvent>,
    _watcher: ReadDirectoryChangesWatcher,
    file_map: HashMap<String, PathBuf>,
    resources: HashMap<ResourceKey, Arc<dyn Resource>>,
}

#[allow(dead_code)]
impl ResourceManager {
    //
    //  This section is for initializing the ResourceManager
    //
    /// Create a new ResourceManager
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

    ///Load in the Generic Texture
    fn gen_texture_load(&mut self) {
        // This shouldn't fail! so we will panic if anything in this fails.
        let key = String::from("error-texture");
        let generic_data = std::fs::read(self.get_abs_file_path("error-texture")).unwrap();
        let texture = Texture::new(&generic_data[..]);
        let file = File::new(generic_data);
        file.set_dependency(Some(ResourceKey::File(key.clone())));
        self.resources
            .insert(ResourceKey::File(key.clone()), Arc::new(file));
        self.resources
            .insert(ResourceKey::Texture(key), Arc::new(texture));
    }

    //
    //  This section is for handling updates to the Resources
    //
    pub fn check_files(&mut self) -> Result<()> {
        match self.event_recv.recv() {
            Ok(event) => {
                match event {
                    DebouncedEvent::Write(src) => {
                        let filename = src
                            .clone()
                            .file_stem()
                            .unwrap()
                            .to_string_lossy()
                            .to_string();
                        let folder = src.parent().iter().last().unwrap().to_str().unwrap();
                        let filepath = self.get_abs_file_path(&filename);
                        println!("{:#?} was edited!", filename);
                        // Is the File Resource loaded?
                        let old_file = self.resources.get(&ResourceKey::File(filename));
                        if old_file.is_some() {
                            let old_file = old_file
                                .unwrap()
                                .clone()
                                .downcast_arc::<File>()
                                .map_err(|_| error!("this shouldn't happen..."))
                                .unwrap();
                            // Reload the File Resource
                            let mut read_bytes = std::fs::read(filepath.clone());
                            if read_bytes.is_err() {
                                error!("unable to find file: {:#?}", filepath);
                                match folder {
                                    "textures" => {
                                        read_bytes =
                                            std::fs::read(self.get_abs_file_path("error-texture"));
                                        debug!(
                                            "loaded file: {:#?}",
                                            self.get_abs_file_path("error-texture")
                                        );
                                    }
                                    _ => {}
                                }
                            } else {
                                debug!("loaded file: {:#?}", filepath);
                            }
                            let mut guard = old_file.data.write().unwrap();
                            guard.data = read_bytes.unwrap();
                            drop(guard);
                            let guard = old_file.data.read().unwrap();
                            let dependency = guard.dependency.clone();
                            if dependency.is_some() {
                                //  update the dependency!
                                match dependency.unwrap() {
                                    ResourceKey::Texture(key) => {
                                        let new_texture = Texture::new_inner(&guard.data[..]);
                                        let old_texture = self
                                            .resources
                                            .get(&ResourceKey::Texture(key.clone()))
                                            .unwrap()
                                            .clone()
                                            .downcast_arc::<Texture>()
                                            .map_err(|_| error!("this shouldn't happen..."))
                                            .unwrap();
                                        let mut tguard = old_texture.data.write().unwrap();
                                        tguard.data = new_texture.data;
                                        debug!("texture: {:#?} updated", key);
                                    }
                                    _ => {}
                                }
                            }
                        }
                    }
                    DebouncedEvent::Remove(src) => {
                        println!("{:#?} was deleted", src);
                    }
                    DebouncedEvent::Rename(src, dest) => {
                        println!("{:#?} was renamed {:#?}", src, dest)
                        // this won't be handled for now
                    }
                    _ => {}
                }
            }
            Err(e) => eprintln!("watch error {:#?}", e),
        }
        Ok(())
    }
    //
    //  This section is for getting Resources from the Manager
    //
    pub fn get_texture(&mut self, filename: &str) -> Result<Arc<Texture>> {
        // if the texture isn't in the resource.json file, provide a generic texture
        if !self.file_map.contains_key(filename) {
            return Ok(self
                .resources
                .get(&&ResourceKey::Texture(String::from("error-texture")))
                .unwrap()
                .clone()
                .downcast_arc::<Texture>()
                .map_err(|_| "This shouldn't happen...")
                .unwrap());
        }
        let path_to_file = self.get_abs_file_path(filename);

        // check to see if the texture is already loaded
        if self
            .resources
            .contains_key(&ResourceKey::Texture(filename.to_string()))
        {
            debug!("Texture: {:#?} not already loaded.", filename);
            // check to see if the file has already been loaded
            if self
                .resources
                .contains_key(&ResourceKey::File(filename.to_string()))
            {
                debug!("File: {:#?} not already loaded.", filename);
                // load the raw file
                let read_bytes = std::fs::read(path_to_file.clone());
                // if the file isn't found return the generic texture file
                if read_bytes.is_err() {
                    error!(
                        "Unable to find file: {:#?}\nReturning generic texture.",
                        path_to_file
                    );
                    return Ok(self
                        .resources
                        .get(&&ResourceKey::Texture(String::from("error-texture")))
                        .unwrap()
                        .clone()
                        .downcast_arc::<Texture>()
                        .map_err(|_| "This shouldn't happen...")
                        .unwrap());
                }
                self.resources.insert(
                    ResourceKey::File(filename.to_string()),
                    Arc::new(File {
                        data: RwLock::new(InnerFile {
                            data: read_bytes.unwrap(),
                            dependency: None,
                        }),
                    }),
                );
            }
            let file_guard = self
                .resources
                .get(&ResourceKey::File(filename.to_string()))
                .unwrap()
                .clone()
                .downcast_arc::<File>()
                .map_err(|_| "This shouldn't happen...")
                .unwrap();
            // create texture
            let texture: Arc<dyn Resource> =
                Arc::new(self.load_texture_from_raw(file_guard.data.read().unwrap().data.clone())?);
            file_guard.data.write().unwrap().dependency =
                Some(ResourceKey::Texture(filename.to_string()));
            self.resources
                .insert(ResourceKey::Texture(filename.to_string()), texture);
            debug!("stored texture resource for: {:#?}", filename);
        } else {
            debug!("Texture: {:#?} already loaded.", filename);
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
    fn load_texture_from_raw(&mut self, raw_bytes: Vec<u8>) -> Result<Texture> {
        Ok(Texture::new(&&raw_bytes[..]))
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
