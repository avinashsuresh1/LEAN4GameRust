use lean4game::storage::profile::UserProfile;
use lean4game::storage::save_manager::SaveManager;
use tempfile::tempdir;

#[test]
fn test_profile_creation_and_atomic_save() {
    let tmp = tempdir().unwrap();
    let sm = SaveManager::new(tmp.path().to_path_buf());

    let mut profile = sm.create_profile("Alice").unwrap();
    assert_eq!(profile.name, "Alice");
    assert_eq!(profile.total_completed(), 0);

    // Mark level completed
    profile.mark_completed("tutorial", "tutorial_1", "rfl", 1, Some("example_rfl"));
    profile.save_draft("tutorial", "tutorial_2", "rw [add_zero]");
    sm.save_profile(&profile).unwrap();

    // Verify it exists on filesystem
    let loaded = sm.load_profile("Alice").unwrap();
    assert_eq!(loaded.name, "Alice");
    assert_eq!(loaded.total_completed(), 1);
    assert!(loaded.is_level_completed("tutorial", "tutorial_1"));
    assert_eq!(loaded.unlocked_theorems, vec!["example_rfl".to_string()]);
    assert_eq!(
        loaded.get_script("tutorial", "tutorial_2"),
        Some("rw [add_zero]".to_string())
    );
}

#[test]
fn test_multi_profile_management() {
    let tmp = tempdir().unwrap();
    let sm = SaveManager::new(tmp.path().to_path_buf());

    sm.create_profile("PlayerOne").unwrap();
    sm.create_profile("PlayerTwo").unwrap();

    let list = sm.list_profiles().unwrap();
    assert_eq!(list, vec!["PlayerOne".to_string(), "PlayerTwo".to_string()]);

    sm.delete_profile("PlayerOne").unwrap();
    let list_after = sm.list_profiles().unwrap();
    assert_eq!(list_after, vec!["PlayerTwo".to_string()]);
}

#[test]
fn test_export_and_import() {
    let tmp = tempdir().unwrap();
    let sm = SaveManager::new(tmp.path().to_path_buf());

    let mut profile = UserProfile::new("Explorer");
    profile.mark_completed("addition", "zero_add", "induction n with d hd\n· rw [add_zero]\n· rw [add_succ, hd]", 2, Some("zero_add"));
    sm.save_profile(&profile).unwrap();

    // Export to a separate path
    let export_file = tmp.path().join("exports").join("explorer_backup.json");
    sm.export_profile_to_file("Explorer", &export_file).unwrap();
    assert!(export_file.exists());

    // Import into a new save manager instance
    let tmp2 = tempdir().unwrap();
    let sm2 = SaveManager::new(tmp2.path().to_path_buf());
    let imported = sm2.import_profile_from_file(&export_file).unwrap();

    assert_eq!(imported.name, "Explorer");
    assert!(imported.is_level_completed("addition", "zero_add"));
    assert_eq!(imported.unlocked_theorems, vec!["zero_add".to_string()]);
}

#[test]
fn test_nonexistent_profile_error() {
    let tmp = tempdir().unwrap();
    let sm = SaveManager::new(tmp.path().to_path_buf());

    let err = sm.load_profile("NonExistent");
    assert!(err.is_err());
}
