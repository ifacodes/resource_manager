use std::collections::HashMap;

use crate::{resource::Resource, resource_manager::ResourceManager};
pub struct ResourceSystem {
    managers: HashMap<String, ResourceManager<T>>,
}

impl<T> ResourceSystem<T> {
    pub fn new() -> Self {
        Self {
            managers: HashMap::new(),
        }
    }
}
