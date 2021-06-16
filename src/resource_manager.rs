use crate::resource::{File, Resource, ResourceKey, Text, Texture};
use anyhow::*;
use log::{debug, error};
use notify::{watcher, DebouncedEvent, ReadDirectoryChangesWatcher, RecursiveMode, Watcher};
use std::{collections::HashMap, path::PathBuf, sync::Arc, time::Duration};

pub struct ResourceManager {
    root: String,
    event_recv: std::sync::mpsc::Receiver<DebouncedEvent>,
    _watcher: ReadDirectoryChangesWatcher,
    file_map: HashMap<String, PathBuf>,
    resources: HashMap<ResourceKey, Arc<dyn Resource>>,
}

impl ResourceManager {
    //
    //  This section is for initializing the ResourceManager
    //
    /// Create a new ResourceManager
    pub fn new() -> Self {
        let (tx, rx) = std::sync::mpsc::channel();
        let mut _watcher = watcher(tx, Duration::from_millis(2000)).unwrap();
        _watcher
            .watch("resources", RecursiveMode::Recursive)
            .unwrap();
        let file = std::fs::File::open("resources/resources.json").unwrap();
        let buffer = std::io::BufReader::new(file);
        let file_map: HashMap<String, PathBuf> = serde_json::from_reader(buffer).unwrap();
        let root = std::env::current_dir()
            .unwrap()
            .to_string_lossy()
            .to_string()
            + "\\resources\\";
        debug!("{:#?}", root);
        Self {
            root,
            event_recv: rx,
            _watcher,
            file_map,
            resources: HashMap::new(),
        }
    }

    ///Load in the Generic Texture
    pub fn init(&mut self) {
        // This shouldn't fail! so we will panic if anything in this fails.
        let key = String::from("error-texture");
        let generic_texture = std::fs::read(self.get_abs_file_path("error-texture")).unwrap();
        let texture = Texture::new(&generic_texture[..]);
        let file = File::new(generic_texture);
        file.set_dependency(Some(ResourceKey::Texture(key.clone())));
        self.resources
            .insert(ResourceKey::File(key.clone()), Arc::new(file));
        self.resources
            .insert(ResourceKey::Texture(key), Arc::new(texture));
        let key = String::from("error-text");
        let generic_text = std::fs::read(self.get_abs_file_path("error-text")).unwrap();
        let text = Text::new(generic_text.clone());
        let file = File::new(generic_text);
        file.set_dependency(Some(ResourceKey::Text(key.clone())));
        self.resources
            .insert(ResourceKey::File(key.clone()), Arc::new(file));
        self.resources
            .insert(ResourceKey::Text(key), Arc::new(text));
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
                        //let folder = src.parent().iter().last().unwrap().to_str().unwrap();
                        let filepath = self.get_abs_file_path(&filename);
                        println!("{:#?} was edited!", filename);
                        // Is the File Resource loaded?
                        let old_file = self.get_file(&filename);
                        if old_file.is_err() {
                            // here is where we replace the current resource with the generic!!
                        }
                        // okay cool so we have a new file now, we just need to update it as well as the resource
                        let old_file = old_file.unwrap();
                        let read_bytes = std::fs::read(filepath.clone())?;
                        // TODO: If this fails, copy the generic into this resource.
                        debug!("loaded file: {:#?}", filepath.clone());
                        // inner block so the guard is dropped after we update the data
                        {
                            let mut write_guard = old_file.data.write().unwrap();
                            write_guard.data = read_bytes;
                        }
                        let read_guard = old_file.data.read().unwrap();

                        // we don't need to test if the File Resource has a dependency because
                        // a File can't be loaded in without gaining a dependency,
                        // and if a File isn't loaded we don't handle the update.
                        let d = read_guard.dependency.clone().unwrap();

                        match d {
                            ResourceKey::Texture(key) => {
                                let inner = Texture::new_inner(&read_guard.data[..]);
                                let texture = self
                                    .resources
                                    .get(&ResourceKey::Texture(key.clone()))
                                    .unwrap()
                                    .clone()
                                    .downcast_arc::<Texture>()
                                    .map_err(|_| "unable to downcast to Texture...")
                                    .unwrap();
                                let mut texture_guard = texture.data.write().unwrap();
                                texture_guard.data = inner.data;
                                debug!("texture: {:#?} updated", key);
                            }
                            ResourceKey::Text(key) => {
                                let inner = Text::new_inner(read_guard.data.clone());
                                let text = self
                                    .resources
                                    .get(&ResourceKey::Text(key.clone()))
                                    .unwrap()
                                    .clone()
                                    .downcast_arc::<Text>()
                                    .map_err(|_| "unable to downcast to Text...")
                                    .unwrap();
                                let mut text_guard = text.data.write().unwrap();
                                text_guard.data = inner.data;
                                debug!("text: {:#?} updated", key);
                            }
                            ResourceKey::Item(key) => {
                                debug!("item: {:#?} updated", key);
                            }
                            _ => {}
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
    fn get_file(&mut self, filename: &str) -> Result<Arc<File>> {
        // We don't need to check if the filename is correct here, because we should NEVER be able to reach that point.
        let path_to_file = self.get_abs_file_path(filename);

        // we still need to check if we need to load it in however
        if !self
            .resources
            .contains_key(&ResourceKey::File(filename.to_string()))
        {
            debug!("File: {:#?} not already loaded.", filename);
            // load the raw file
            let read_bytes = std::fs::read(path_to_file.clone())?;

            self.resources.insert(
                ResourceKey::File(filename.to_string()),
                Arc::new(File::new(read_bytes)),
            );
        }

        Ok(self
            .resources
            .get(&&ResourceKey::File(filename.to_string()))
            .unwrap()
            .clone()
            .downcast_arc::<File>()
            .map_err(|_| "Unable to downcast to File...")
            .unwrap())
    }

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
        // check to see if the texture is already loaded
        if !self
            .resources
            .contains_key(&ResourceKey::Texture(filename.to_string()))
        {
            debug!("Texture: {:#?} not already loaded.", filename);
            // check to see if the file has already been loaded

            let file = self.get_file(filename);

            if file.is_err() {
                error!(
                    "Unable to find file for: \"{}\" Returning generic Texture.",
                    filename
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

            let file_guard = file.unwrap();
            // create texture
            let texture: Arc<dyn Resource> = Arc::new(Texture::new(
                &file_guard.data.read().unwrap().data.clone()[..],
            ));
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
            .map_err(|_| "unable to downcast to Texture")
            .unwrap())
    }

    pub fn get_text(&mut self, filename: &str) -> Result<Arc<Text>> {
        if !self.file_map.contains_key(filename) {
            return Ok(self
                .resources
                .get(&ResourceKey::Text(String::from("error-text")))
                .unwrap()
                .clone()
                .downcast_arc::<Text>()
                .map_err(|_| "unable to downcast to Text...")
                .unwrap());
        }

        if !self
            .resources
            .contains_key(&ResourceKey::Text(filename.to_string()))
        {
            debug!("File: {:#?} not already loaded.", filename);
            let file = self.get_file(filename);

            if file.is_err() {
                error!(
                    "Unable to find file for: \"{}\" Returning generic Text.",
                    filename
                );
                return Ok(self
                    .resources
                    .get(&ResourceKey::Text(String::from("error-text")))
                    .unwrap()
                    .clone()
                    .downcast_arc::<Text>()
                    .map_err(|_| "unable to cast to Text...")
                    .unwrap());
            }

            let file_guard = file.unwrap();
            let text: Arc<dyn Resource> =
                Arc::new(Text::new(file_guard.data.read().unwrap().data.clone()));
            file_guard.data.write().unwrap().dependency =
                Some(ResourceKey::Text(filename.to_string()));
            self.resources
                .insert(ResourceKey::Text(filename.to_string()), text);
            debug!("stored Text resource for: {:#?}", filename);
        } else {
            debug!("Text: {:#?} already loaded.", filename);
        }

        Ok(self
            .resources
            .get(&ResourceKey::Text(filename.to_string()))
            .unwrap()
            .clone()
            .downcast_arc::<Text>()
            .map_err(|_| "unable to downcast to text...")
            .unwrap())
    }
    //
    //  This section is for helper functions and things that shouldn't be public
    //
    /// Update A File Resource
    #[allow(dead_code)]
    fn update_file() {}
    /// Update A Texture Resource
    #[allow(dead_code)]
    fn update_texture() {}

    fn get_abs_file_path(&self, filename: &str) -> String {
        let mut abs = self.root.clone();
        debug!("{:#?}", abs);
        abs = format!("{}{}", abs, self.file_map.get(filename).unwrap().display());
        debug!("{:#?}", abs);
        abs
    }
}
