use crate::game::world::World;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GamePackage {
    pub id: String,
    pub title: String,
    pub description: String,
    pub author: String,
    pub icon: String,
    pub worlds: Vec<World>,
}

impl GamePackage {
    pub fn get_world(&self, world_id: &str) -> Option<&World> {
        self.worlds.iter().find(|w| w.id == world_id)
    }

    pub fn total_levels(&self) -> usize {
        self.worlds.iter().map(|w| w.levels.len()).sum()
    }
}
