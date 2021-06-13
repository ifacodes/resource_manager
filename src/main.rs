mod resource;
mod resource_manager;
use env_logger;
use resource_manager::ResourceManager;

fn main() {
    env_logger::init();
    let mut rm: ResourceManager = ResourceManager::new();
    let texture = rm.get_texture("happy-tree").unwrap();
    let _test = rm.get_texture("mmm").unwrap();

    let bytes = std::fs::read("./textures/happy-tree.png").unwrap();
    let image = image::load_from_memory(&bytes[..]).unwrap();
    let texture2 = image.as_rgba8().unwrap().clone();
    assert_eq!(texture2, texture.data, "test");
    loop {
        match rm.check_files() {
            Ok(()) => {}
            Err(e) => eprintln!("{:#?}", e),
        }
    }
}
