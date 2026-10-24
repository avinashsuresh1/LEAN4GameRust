use crate::game::level::Level;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct World {
    pub id: String,
    pub name: String,
    pub description: String,
    pub icon: String,
    pub levels: Vec<Level>,
}

impl World {
    pub fn get_level(&self, level_id: &str) -> Option<&Level> {
        self.levels.iter().find(|l| l.id == level_id)
    }

    pub fn get_level_by_number(&self, number: usize) -> Option<&Level> {
        self.levels.iter().find(|l| l.number == number)
    }
}
