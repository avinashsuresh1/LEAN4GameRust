use lean4game::desktop::{DesktopApp, DesktopPersistenceBridge};
use std::collections::HashMap;
use tempfile::tempdir;

#[test]
fn test_resolve_game_url() {
    assert_eq!(
        DesktopApp::resolve_game_url("nng"),
        "https://adam.math.hhu.de/#/g/leanprover-community/nng4"
    );
    assert_eq!(
        DesktopApp::resolve_game_url("real-analysis"),
        "https://adam.math.hhu.de/#/g/tomyun/lean-game-real-analysis"
    );
    assert_eq!(
        DesktopApp::resolve_game_url("ra"),
        "https://adam.math.hhu.de/#/g/tomyun/lean-game-real-analysis"
    );
    assert_eq!(
        DesktopApp::resolve_game_url("set-theory"),
        "https://adam.math.hhu.de/#/g/ImperialCollegeLondon/formalising-mathematics-2024"
    );
    assert_eq!(
        DesktopApp::resolve_game_url("robo"),
        "https://adam.math.hhu.de/#/g/hhu-adam/Robo"
    );
    assert_eq!(
        DesktopApp::resolve_game_url("logic"),
        "https://adam.math.hhu.de/#/g/leanprover-community/logic"
    );
    assert_eq!(
        DesktopApp::resolve_game_url("riehl"),
        "https://adam.math.hhu.de/#/g/emilyriehl/reintroductiontoproofs"
    );
    assert_eq!(
        DesktopApp::resolve_game_url("linear-algebra"),
        "https://adam.math.hhu.de/#/g/ImperialCollegeLondon/linear-algebra"
    );
    assert_eq!(
        DesktopApp::resolve_game_url("hub"),
        "https://adam.math.hhu.de/"
    );
    assert_eq!(
        DesktopApp::resolve_game_url("https://custom.domain.org/#/game"),
        "https://custom.domain.org/#/game"
    );
    assert_eq!(
        DesktopApp::resolve_game_url("AlexanderBentkamp/lean4game-logic"),
        "https://adam.math.hhu.de/#/g/AlexanderBentkamp/lean4game-logic"
    );
}

#[test]
fn test_bridge_injection_script_generation() {
    let mut state = HashMap::new();
    state.insert("nng4_completed".to_string(), "true".to_string());
    state.insert("active_proof".to_string(), "induction n with d hd".to_string());

    let script = DesktopPersistenceBridge::generate_injection_script(&state);

    // Verify key components of the script
    assert!(script.contains("localStorage.setItem(k, v)"));
    assert!(script.contains("window.ipc.postMessage"));
    assert!(script.contains("dumpAllLocalStorage"));
    assert!(script.contains("nng4_completed"));
    assert!(script.contains("active_proof"));
    assert!(script.contains("save_state"));
}

#[test]
fn test_bridge_atomic_save_and_load() {
    let dir = tempdir().unwrap();
    let save_file = dir.path().join("test_save.json");

    let mut state = HashMap::new();
    state.insert("level_tutorial_1".to_string(), "rfl".to_string());
    state.insert("real_analysis_limit".to_string(), "use 0\nrw [abs_zero]".to_string());
    state.insert("user_stars".to_string(), "46".to_string());

    // Save atomically
    let res = DesktopPersistenceBridge::save_state_atomic(&save_file, &state);
    assert!(res.is_ok(), "Atomic save failed: {:?}", res.err());
    assert!(save_file.exists());

    // Load back
    let loaded = DesktopPersistenceBridge::load_saved_state(&save_file);
    assert_eq!(loaded.len(), 3);
    assert_eq!(loaded.get("level_tutorial_1").unwrap(), "rfl");
    assert_eq!(loaded.get("real_analysis_limit").unwrap(), "use 0\nrw [abs_zero]");
    assert_eq!(loaded.get("user_stars").unwrap(), "46");
}

#[test]
fn test_bridge_ipc_message_handling() {
    let dir = tempdir().unwrap();
    let save_file = dir.path().join("ipc_save.json");

    let ipc_json = r#"{
        "type": "save_state",
        "payload": {
            "game_id": "real_analysis",
            "proof_script": "intro c eps h_eps\nuse 0",
            "solved": "true"
        }
    }"#;

    let res = DesktopPersistenceBridge::handle_ipc_message(ipc_json, &save_file);
    assert_eq!(res, Ok(3));

    let loaded = DesktopPersistenceBridge::load_saved_state(&save_file);
    assert_eq!(loaded.len(), 3);
    assert_eq!(loaded.get("game_id").unwrap(), "real_analysis");
    assert_eq!(loaded.get("solved").unwrap(), "true");
}

#[test]
fn test_bridge_export_and_import() {
    let dir = tempdir().unwrap();
    let original_save = dir.path().join("original.json");
    let export_file = dir.path().join("backup.json");
    let restored_file = dir.path().join("restored.json");

    let mut state = HashMap::new();
    state.insert("profile_name".to_string(), "Student1".to_string());
    state.insert("solved_count".to_string(), "46".to_string());

    DesktopPersistenceBridge::save_state_atomic(&original_save, &state).unwrap();

    // Export
    DesktopPersistenceBridge::export_state(&original_save, &export_file).unwrap();
    assert!(export_file.exists());

    // Import
    let count = DesktopPersistenceBridge::import_state(&export_file, &restored_file).unwrap();
    assert_eq!(count, 2);

    let loaded = DesktopPersistenceBridge::load_saved_state(&restored_file);
    assert_eq!(loaded.get("profile_name").unwrap(), "Student1");
    assert_eq!(loaded.get("solved_count").unwrap(), "46");
}

#[test]
fn test_linux_env_vars() {
    let default_vars = DesktopApp::linux_env_vars(&[]);
    let var_map: HashMap<&str, &str> = default_vars.into_iter().collect();
    assert_eq!(var_map.get("LEAN4GAME_LINUX_ENV_CONFIGURED"), Some(&"1"));
    assert_eq!(var_map.get("WEBKIT_DISABLE_DMABUF_RENDERER"), Some(&"1"));
    assert_eq!(var_map.get("WEBKIT_FORCE_SANDBOX"), Some(&"0"));
    assert_eq!(var_map.get("__NV_DISABLE_EXPLICIT_SYNC"), Some(&"1"));
    assert!(!var_map.contains_key("WEBKIT_DISABLE_COMPOSITING_MODE"));
    assert!(!var_map.contains_key("LIBGL_ALWAYS_SOFTWARE"));

    // When --software-render is specified
    let sw_vars = DesktopApp::linux_env_vars(&["--software-render".to_string()]);
    let sw_map: HashMap<&str, &str> = sw_vars.into_iter().collect();
    assert_eq!(sw_map.get("WEBKIT_DISABLE_COMPOSITING_MODE"), Some(&"1"));
    assert_eq!(sw_map.get("LIBGL_ALWAYS_SOFTWARE"), Some(&"1"));

    // When --no-gpu is specified
    let nogpu_vars = DesktopApp::linux_env_vars(&["--no-gpu".to_string()]);
    let nogpu_map: HashMap<&str, &str> = nogpu_vars.into_iter().collect();
    assert_eq!(nogpu_map.get("WEBKIT_DISABLE_COMPOSITING_MODE"), Some(&"1"));
    assert_eq!(nogpu_map.get("LIBGL_ALWAYS_SOFTWARE"), Some(&"1"));
}

#[test]
fn test_reset_json_template_and_import() {
    let reset_path = std::path::Path::new("reset.json");
    assert!(reset_path.exists(), "reset.json must exist in the repository root");

    let state = DesktopPersistenceBridge::load_saved_state(reset_path);
    assert!(state.contains_key("game_progress"), "reset.json must contain game_progress key");
    assert_eq!(state.get("game_progress").unwrap(), "{\"games\":{}}");

    // Verify it can be imported cleanly to a save destination
    let dir = tempdir().unwrap();
    let dest_save = dir.path().join("official_lean4game_save.json");
    let count = DesktopPersistenceBridge::import_state(reset_path, &dest_save).unwrap();
    assert_eq!(count, 1);

    let imported_state = DesktopPersistenceBridge::load_saved_state(&dest_save);
    assert_eq!(imported_state.get("game_progress").unwrap(), "{\"games\":{}}");
}

