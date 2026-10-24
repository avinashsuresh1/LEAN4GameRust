use crate::storage::profile::UserProfile;
use std::fs::{self, File};
use std::io::{Read, Write};
use std::path::{Path, PathBuf};

#[derive(Debug, Clone)]
pub struct SaveManager {
    base_dir: PathBuf,
}

impl SaveManager {
    pub fn new(base_dir: PathBuf) -> Self {
        if !base_dir.exists() {
            let _ = fs::create_dir_all(&base_dir);
        }
        Self { base_dir }
    }

    pub fn default_saves_dir() -> PathBuf {
        PathBuf::from("./saves")
    }

    pub fn base_dir(&self) -> &Path {
        &self.base_dir
    }

    fn profile_path(&self, name: &str) -> PathBuf {
        let safe_name = name.replace(|c: char| !c.is_alphanumeric() && c != '_' && c != '-', "_");
        self.base_dir.join(format!("{}.json", safe_name))
    }

    /// List all available profile names in the saves directory.
    pub fn list_profiles(&self) -> Result<Vec<String>, String> {
        if !self.base_dir.exists() {
            return Ok(Vec::new());
        }

        let mut profiles = Vec::new();
        let entries = fs::read_dir(&self.base_dir)
            .map_err(|e| format!("Failed to read saves directory: {}", e))?;

        for entry in entries.flatten() {
            let path = entry.path();
            if path.is_file() && path.extension().and_then(|s| s.to_str()) == Some("json") {
                if let Some(stem) = path.file_stem().and_then(|s| s.to_str()) {
                    profiles.push(stem.to_string());
                }
            }
        }

        profiles.sort();
        Ok(profiles)
    }

    /// Save a profile to the filesystem atomically.
    pub fn save_profile(&self, profile: &UserProfile) -> Result<(), String> {
        if !self.base_dir.exists() {
            fs::create_dir_all(&self.base_dir)
                .map_err(|e| format!("Failed to create saves directory: {}", e))?;
        }

        let target_path = self.profile_path(&profile.name);
        let tmp_path = self.base_dir.join(format!("{}.tmp", profile.name));

        let json = serde_json::to_string_pretty(profile)
            .map_err(|e| format!("Serialization error: {}", e))?;

        // Write to temporary file first
        {
            let mut file = File::create(&tmp_path)
                .map_err(|e| format!("Failed to create temp save file {:?}: {}", tmp_path, e))?;
            file.write_all(json.as_bytes())
                .map_err(|e| format!("Failed to write save data: {}", e))?;
            file.flush()
                .map_err(|e| format!("Failed to flush save data: {}", e))?;
        }

        // Atomically replace target
        if target_path.exists() {
            let _ = fs::remove_file(&target_path);
        }
        fs::rename(&tmp_path, &target_path)
            .map_err(|e| format!("Failed to finalize save file: {}", e))?;

        Ok(())
    }

    /// Load a profile from the filesystem.
    pub fn load_profile(&self, name: &str) -> Result<UserProfile, String> {
        let path = self.profile_path(name);
        if !path.exists() {
            return Err(format!("Profile '{}' does not exist at {:?}", name, path));
        }

        let mut file = File::open(&path)
            .map_err(|e| format!("Failed to open profile file {:?}: {}", path, e))?;
        let mut contents = String::new();
        file.read_to_string(&mut contents)
            .map_err(|e| format!("Failed to read profile file: {}", e))?;

        let profile: UserProfile = serde_json::from_str(&contents)
            .map_err(|e| format!("Invalid profile JSON format: {}", e))?;

        Ok(profile)
    }

    /// Create and persist a new user profile.
    pub fn create_profile(&self, name: &str) -> Result<UserProfile, String> {
        let trimmed = name.trim();
        if trimmed.is_empty() {
            return Err("Profile name cannot be empty".to_string());
        }

        let profile = UserProfile::new(trimmed);
        self.save_profile(&profile)?;
        Ok(profile)
    }

    /// Delete a profile from the filesystem.
    pub fn delete_profile(&self, name: &str) -> Result<(), String> {
        let path = self.profile_path(name);
        if path.exists() {
            fs::remove_file(&path)
                .map_err(|e| format!("Failed to delete profile file {:?}: {}", path, e))?;
        }
        Ok(())
    }

    /// Export a profile to an external file on the user's filesystem.
    pub fn export_profile_to_file(&self, profile_name: &str, dest_path: &Path) -> Result<(), String> {
        let profile = self.load_profile(profile_name)?;
        let json = serde_json::to_string_pretty(&profile)
            .map_err(|e| format!("Serialization error: {}", e))?;

        if let Some(parent) = dest_path.parent() {
            if !parent.exists() {
                let _ = fs::create_dir_all(parent);
            }
        }

        fs::write(dest_path, json.as_bytes())
            .map_err(|e| format!("Failed to export profile to {:?}: {}", dest_path, e))?;

        Ok(())
    }

    /// Import a profile from an external JSON file on the user's filesystem.
    pub fn import_profile_from_file(&self, source_path: &Path) -> Result<UserProfile, String> {
        if !source_path.exists() {
            return Err(format!("Source file does not exist: {:?}", source_path));
        }

        let contents = fs::read_to_string(source_path)
            .map_err(|e| format!("Failed to read source file: {}", e))?;

        let profile: UserProfile = serde_json::from_str(&contents)
            .map_err(|e| format!("Invalid profile backup file format: {}", e))?;

        self.save_profile(&profile)?;
        Ok(profile)
    }
}
