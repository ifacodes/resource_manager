use std::sync::RwLock;

use anyhow::*;
use downcast_rs::{impl_downcast, DowncastSync};
use image::RgbaImage;

pub trait Resource: DowncastSync {}
impl_downcast!(sync Resource);

pub struct Texture {
    pub data: RwLock<InnerTexture>,
}
#[derive(Debug)]
pub struct InnerTexture {
    pub data: RgbaImage,
}

impl Resource for Texture {}
impl Texture {
    pub fn new(data: &[u8]) -> Self {
        let image = image::load_from_memory(data)
            .context(format!("couldn't load image from memory"))
            .unwrap();
        let buf_image = image
            .as_rgba8()
            .context(format!("couldn't get rgba data"))
            .unwrap()
            .clone();

        Texture {
            data: RwLock::new(InnerTexture { data: buf_image }),
        }
    }
    pub fn new_inner(data: &[u8]) -> InnerTexture {
        let image = image::load_from_memory(data)
            .context(format!("couldn't load image from memory"))
            .unwrap();
        let buf_image = image
            .as_rgba8()
            .context(format!("couldn't get rgba data"))
            .unwrap()
            .clone();
        InnerTexture { data: buf_image }
    }
}

#[derive(Debug)]
pub struct File {
    pub data: RwLock<InnerFile>,
}
#[derive(Debug)]
pub struct InnerFile {
    pub dependency: Option<ResourceKey>,
    pub data: Vec<u8>,
}

impl Resource for File {}
impl File {
    pub fn new(data: Vec<u8>) -> Self {
        File {
            data: RwLock::new(InnerFile {
                data,
                dependency: None,
            }),
        }
    }
    pub fn set_dependency(&self, key: Option<ResourceKey>) {
        self.data.write().unwrap().dependency = key;
    }
}

#[derive(Debug, PartialEq, Eq, Hash, Clone)]
pub enum ResourceKey {
    Texture(String),
    File(String),
}
