use anyhow::*;
use downcast_rs::{impl_downcast, DowncastSync};
use image::RgbaImage;

pub trait Resource: DowncastSync {}
impl_downcast!(sync Resource);

pub struct Texture {
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
        Texture { data: buf_image }
    }
}

#[derive(Debug, Clone)]
pub struct File {
    pub dependency: Option<ResourceKey>,
    pub data: Vec<u8>,
}

impl Resource for File {}

#[derive(Debug, PartialEq, Eq, Hash, Clone)]
pub enum ResourceKey {
    Texture(String),
    File(String),
}
