use image::RgbaImage;
use std::path::PathBuf;
use uuid::Uuid;

pub enum Resource {
    Texture(Texture),
    File(File),
    //Audio,
    //Item,
}

impl Resource {
    pub fn inner(&self) -> Option<&Texture> {
        match self {
            Resource::Texture(t) => Some(&t),
            _ => None,
        }
    }
}

pub struct Texture {
    pub diffuse: RgbaImage,
    pub file: Uuid,
}

pub struct File {
    pub path_to_file: PathBuf,
    pub raw_file: Vec<u8>,
}
