use std::sync::RwLock;

use anyhow::*;
use downcast_rs::{impl_downcast, DowncastSync};
use image::RgbaImage;

pub trait Resource: DowncastSync {}
impl_downcast!(sync Resource);

#[derive(Debug, PartialEq, Eq, Hash, Clone)]
pub enum ResourceKey {
    Texture(String),
    File(String),
    Text(String),
    Item(String),
}

pub struct Texture {
    pub data: RwLock<InnerResource<RgbaImage>>,
}

#[derive(Debug)]
pub struct InnerResource<T> {
    pub data: T,
    pub dependency: Option<ResourceKey>,
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
            data: RwLock::new(InnerResource::<RgbaImage> {
                data: buf_image,
                dependency: None,
            }),
        }
    }
    pub fn new_inner(data: &[u8]) -> InnerResource<RgbaImage> {
        let image = image::load_from_memory(data)
            .context(format!("couldn't load image from memory"))
            .unwrap();
        let buf_image = image
            .as_rgba8()
            .context(format!("couldn't get rgba data"))
            .unwrap()
            .clone();
        InnerResource::<RgbaImage> {
            data: buf_image,
            dependency: None,
        }
    }
}

#[derive(Debug)]
pub struct File {
    pub data: RwLock<InnerResource<Vec<u8>>>,
}

impl Resource for File {}
impl File {
    pub fn new(data: Vec<u8>) -> Self {
        File {
            data: RwLock::new(InnerResource::<Vec<u8>> {
                data,
                dependency: None,
            }),
        }
    }
    pub fn set_dependency(&self, key: Option<ResourceKey>) {
        self.data.write().unwrap().dependency = key;
    }
    pub fn new_inner(data: Vec<u8>) -> InnerResource<Vec<u8>> {
        InnerResource::<Vec<u8>> {
            data,
            dependency: None,
        }
    }
}

pub struct Text {
    pub data: RwLock<InnerResource<String>>,
}
impl Resource for Text {}
impl Text {
    pub fn new(data: Vec<u8>) -> Self {
        Self {
            data: RwLock::new(InnerResource::<String> {
                data: String::from_utf8(data).unwrap(),
                dependency: None,
            }),
        }
    }
    pub fn new_inner(data: Vec<u8>) -> InnerResource<String> {
        InnerResource::<String> {
            data: String::from_utf8(data).unwrap(),
            dependency: None,
        }
    }
}

pub struct ItemDetails {
    pub name: String,
    pub durability: u16,
    pub details: String,
}

pub struct Item {
    pub data: RwLock<InnerResource<ItemDetails>>,
}
impl Resource for Item {}
impl Item {
    pub fn new(details: ItemDetails) -> Self {
        Self {
            data: RwLock::new(InnerResource::<ItemDetails> {
                data: details,
                dependency: None,
            }),
        }
    }
    pub fn new_inner(data: ItemDetails) -> InnerResource<ItemDetails> {
        InnerResource::<ItemDetails> {
            data,
            dependency: None,
        }
    }
}
