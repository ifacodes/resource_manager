use downcast_rs::{impl_downcast, DowncastSync};
use image::RgbaImage;
use std::sync::Arc;

pub trait Resource: DowncastSync {}
impl_downcast!(sync Resource);

pub struct Texture {
    pub diffuse: RgbaImage,
}

impl Resource for Texture {}

pub struct File {
    pub dependency: Arc<dyn Resource>,
    pub raw_file: Vec<u8>,
}

impl Resource for File {}
