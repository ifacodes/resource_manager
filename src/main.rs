mod resource;
mod resource_manager;
use resource_manager::ResourceManager;

fn main() {
    let mut rm: ResourceManager = ResourceManager::new(".");
    let texture = rm.get_texture("happy-tree").unwrap();
    let _texture3 = rm.get_texture("no.png");

    let bytes = std::fs::read("./textures/happy-tree.png").unwrap();
    let image = image::load_from_memory(&bytes[..]).unwrap();
    let texture2 = image.as_rgba8().unwrap().clone();
    assert_eq!(texture2, texture.diffuse, "test");
    loop {
        match rm.check_files() {
            Ok(()) => {}
            Err(e) => eprintln!("{:#?}", e),
        }
    }
}
