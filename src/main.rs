mod resource;
mod resource_manager;
use resource_manager::ResourceManager;

fn main() {
    let mut rm: ResourceManager = ResourceManager::new();
    let texture_ref = rm.get_texture("textures/happy-tree.png").unwrap();
    let texture = texture_ref.inner().unwrap();
    let bytes = std::fs::read("./textures/happy-tree.png").unwrap();
    let image = image::load_from_memory(&bytes[..]).unwrap();
    let texture2 = image.as_rgba8().unwrap().clone();
    assert_eq!(texture2, texture.diffuse, "test");
}
