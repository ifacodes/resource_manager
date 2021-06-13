use downcast_rs::{impl_downcast, DowncastSync};
use image::RgbaImage;

pub trait Resource: DowncastSync {}
impl_downcast!(sync Resource);

pub struct Texture {
    pub data: RgbaImage,
}

impl Resource for Texture {}

pub struct File {
    pub dependency: Option<String>,
    pub data: Vec<u8>,
}

impl Resource for File {}

#[derive(Debug, PartialEq, Eq, Hash)]
pub enum ResourceKey {
    Texture(String),
    File(String),
}
