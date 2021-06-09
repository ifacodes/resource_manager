use downcast_rs::{impl_downcast, DowncastSync};
use image::RgbaImage;
use std::{any::Any, path::PathBuf};
use uuid::Uuid;

pub trait Resource: DowncastSync {}
impl_downcast!(sync Resource);

pub struct Texture {
    pub diffuse: RgbaImage,
    pub file: Uuid,
}

impl Resource for Texture {}

pub struct File {
    pub path_to_file: PathBuf,
    pub raw_file: Vec<u8>,
}

impl Resource for File {}
