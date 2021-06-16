mod resource;
mod resource_manager;
use env_logger;
use resource_manager::ResourceManager;

fn main() {
    env_logger::init();
    let mut rm: ResourceManager = ResourceManager::new();
    rm.init();
    let texture = rm.get_texture("happy-tree").unwrap();
    let _test = rm.get_texture("mmm").unwrap();
    let text = rm.get_text("test-text").unwrap();
    let mut old_text: String;

    let bytes = std::fs::read("./resources/texture/happy-tree.png").unwrap();
    let image = image::load_from_memory(&bytes[..]).unwrap();
    let texture2 = image.as_rgba8().unwrap().clone();
    {
        let guard = texture.data.read().unwrap();
        assert_eq!(texture2, guard.data);
    }
    {
        let text_guard = text.data.read().unwrap();
        println!("{}", text_guard.data);
        old_text = text_guard.data.clone();
    }
    loop {
        match rm.check_files() {
            Ok(()) => {}
            Err(e) => eprintln!("{:#?}", e),
        }
        let guard = texture.data.read().unwrap();
        if texture2 == guard.data {
            println!("Not Updated!");
        }
        if texture2 != guard.data {
            println!("Updated!");
        }
        let text_guard = text.data.read().unwrap();
        if old_text != text_guard.data {
            println!("Old: {}\nNew: {}", old_text, text_guard.data);
            old_text = text_guard.data.clone();
        }
    }
}
