use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LevelProgress {
    pub completed: bool,
    pub proof_script: String,
    pub step_count: usize,
    pub completed_at: Option<DateTime<Utc>>,
    pub last_modified: DateTime<Utc>,
}

fn default_game_id() -> String {
    "natural_numbers".to_string()
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UserProfile {
    pub version: u32,
    pub name: String,
    pub created_at: DateTime<Utc>,
    pub last_played: DateTime<Utc>,
    #[serde(default = "default_game_id")]
    pub active_game_id: String,
    pub active_world_id: String,
    pub active_level_id: String,
    pub levels: HashMap<String, LevelProgress>,
    pub unlocked_theorems: Vec<String>,
    pub draft_scripts: HashMap<String, String>,
}

impl UserProfile {
    pub fn new(name: impl Into<String>) -> Self {
        let now = Utc::now();
        Self {
            version: 1,
            name: name.into(),
            created_at: now,
            last_played: now,
            active_game_id: "natural_numbers".to_string(),
            active_world_id: "tutorial".to_string(),
            active_level_id: "tutorial_1".to_string(),
            levels: HashMap::new(),
            unlocked_theorems: Vec::new(),
            draft_scripts: HashMap::new(),
        }
    }

    fn key(world_id: &str, level_id: &str) -> String {
        format!("{}/{}", world_id, level_id)
    }

    pub fn is_level_completed(&self, world_id: &str, level_id: &str) -> bool {
        self.levels
            .get(&Self::key(world_id, level_id))
            .map(|p| p.completed)
            .unwrap_or(false)
    }

    pub fn total_completed(&self) -> usize {
        self.levels.values().filter(|p| p.completed).count()
    }

    pub fn get_script(&self, world_id: &str, level_id: &str) -> Option<String> {
        let k = Self::key(world_id, level_id);
        if let Some(draft) = self.draft_scripts.get(&k) {
            return Some(draft.clone());
        }
        if let Some(progress) = self.levels.get(&k) {
            return Some(progress.proof_script.clone());
        }
        None
    }

    pub fn save_draft(&mut self, world_id: &str, level_id: &str, script: &str) {
        let k = Self::key(world_id, level_id);
        self.draft_scripts.insert(k, script.to_string());
        self.last_played = Utc::now();
    }

    pub fn mark_completed(
        &mut self,
        world_id: &str,
        level_id: &str,
        script: &str,
        step_count: usize,
        theorem_name: Option<&str>,
    ) {
        let now = Utc::now();
        let k = Self::key(world_id, level_id);

        let progress = LevelProgress {
            completed: true,
            proof_script: script.to_string(),
            step_count,
            completed_at: Some(now),
            last_modified: now,
        };

        self.levels.insert(k.clone(), progress);
        self.draft_scripts.insert(k, script.to_string());
        self.last_played = now;

        if let Some(thm) = theorem_name {
            if !self.unlocked_theorems.contains(&thm.to_string()) {
                self.unlocked_theorems.push(thm.to_string());
            }
        }
    }
}
