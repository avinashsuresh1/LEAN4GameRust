use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs::{self, File};
use std::io::{self, Write};
use std::path::{Path, PathBuf};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IpcSaveMessage {
    #[serde(rename = "type")]
    pub msg_type: String,
    pub payload: HashMap<String, String>,
}

pub struct DesktopPersistenceBridge;

impl DesktopPersistenceBridge {
    pub const DEFAULT_SAVE_FILENAME: &'static str = "official_lean4game_save.json";

    /// Default path for official Lean4Game web client state
    pub fn default_save_path() -> PathBuf {
        let mut path = PathBuf::from("saves");
        if !path.exists() {
            let _ = fs::create_dir_all(&path);
        }
        path.push(Self::DEFAULT_SAVE_FILENAME);
        path
    }

    /// Load saved localStorage key-values from disk.
    pub fn load_saved_state(path: &Path) -> HashMap<String, String> {
        if !path.exists() {
            return HashMap::new();
        }

        match fs::read_to_string(path) {
            Ok(content) => serde_json::from_str(&content).unwrap_or_default(),
            Err(_) => HashMap::new(),
        }
    }

    /// Atomically save key-values to disk using a temporary file.
    pub fn save_state_atomic(path: &Path, data: &HashMap<String, String>) -> io::Result<()> {
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent)?;
        }

        let tmp_path = path.with_extension("tmp");
        let json_str = serde_json::to_string_pretty(data)
            .map_err(|e| io::Error::new(io::ErrorKind::InvalidData, e))?;

        let mut file = File::create(&tmp_path)?;
        file.write_all(json_str.as_bytes())?;
        file.sync_all()?;
        drop(file);

        fs::rename(&tmp_path, path)?;
        Ok(())
    }

    /// Generates the JavaScript injection snippet that hydrates localStorage before
    /// page scripts execute, and proxies localStorage changes back to Rust via window.ipc.
    pub fn generate_injection_script(saved_state: &HashMap<String, String>) -> String {
        let serialized_state = serde_json::to_string(saved_state).unwrap_or_else(|_| "{}".to_string());

        format!(
            r#"(function() {{
    try {{
        console.log("[LEAN4Game Desktop] Initializing local filesystem persistence bridge...");

        const savedState = {saved_state_json};
        if (savedState && typeof savedState === 'object') {{
            let restoredCount = 0;
            for (const [k, v] of Object.entries(savedState)) {{
                try {{
                    localStorage.setItem(k, v);
                    restoredCount++;
                }} catch(e) {{
                    console.error("[LEAN4Game Desktop] Failed restoring key:", k, e);
                }}
            }}
            console.log("[LEAN4Game Desktop] Successfully restored " + restoredCount + " items from local filesystem.");
        }}

        function dumpAllLocalStorage() {{
            const dump = {{}};
            try {{
                for (let i = 0; i < localStorage.length; i++) {{
                    const key = localStorage.key(i);
                    if (key) {{
                        dump[key] = localStorage.getItem(key);
                    }}
                }}
            }} catch(e) {{}}
            return dump;
        }}

        let debounceTimer = null;
        function notifyRustSave() {{
            if (debounceTimer) clearTimeout(debounceTimer);
            debounceTimer = setTimeout(() => {{
                try {{
                    const dump = dumpAllLocalStorage();
                    const msg = JSON.stringify({{
                        type: "save_state",
                        payload: dump
                    }});
                    if (window.ipc && window.ipc.postMessage) {{
                        window.ipc.postMessage(msg);
                    }}
                }} catch(e) {{
                    console.error("[LEAN4Game Desktop] Error posting save to Rust:", e);
                }}
            }}, 400);
        }}

        // Hook Storage.prototype so it works across all WebKit and Safari engines
        if (typeof Storage !== 'undefined' && Storage.prototype) {{
            try {{
                const originalSetItem = Storage.prototype.setItem;
                Storage.prototype.setItem = function(key, val) {{
                    originalSetItem.call(this, key, val);
                    notifyRustSave();
                }};

                const originalRemoveItem = Storage.prototype.removeItem;
                Storage.prototype.removeItem = function(key) {{
                    originalRemoveItem.call(this, key);
                    notifyRustSave();
                }};

                const originalClear = Storage.prototype.clear;
                Storage.prototype.clear = function() {{
                    originalClear.call(this);
                    notifyRustSave();
                }};
            }} catch(e) {{
                console.error("[LEAN4Game Desktop] Failed to hook Storage.prototype:", e);
            }}
        }}

        // Periodic heartbeat flush every 2.5 seconds to guarantee save on proof completion
        setInterval(notifyRustSave, 2500);

        // Ctrl+S / Cmd+S manual save shortcut
        window.addEventListener('keydown', (e) => {{
            if ((e.ctrlKey || e.metaKey) && e.key.toLowerCase() === 's') {{
                e.preventDefault();
                notifyRustSave();
                console.log("[LEAN4Game Desktop] Manual save triggered (Ctrl+S).");
            }}
        }});
    }} catch(topErr) {{
        console.error("[LEAN4Game Desktop] Persistence script error:", topErr);
    }}
}})();"#,
            saved_state_json = serialized_state
        )
    }

    /// Process an IPC message from the WebView2 JavaScript runtime and update disk.
    pub fn handle_ipc_message(json_str: &str, save_path: &Path) -> Result<usize, String> {
        let msg: IpcSaveMessage = serde_json::from_str(json_str)
            .map_err(|e| format!("Failed to parse IPC message: {}", e))?;

        if msg.msg_type == "save_state" {
            let count = msg.payload.len();
            Self::save_state_atomic(save_path, &msg.payload)
                .map_err(|e| format!("Filesystem write error: {}", e))?;
            Ok(count)
        } else {
            Err(format!("Unknown IPC message type: {}", msg.msg_type))
        }
    }

    /// Export saved state to an external backup JSON file.
    pub fn export_state(src_path: &Path, dest_path: &Path) -> io::Result<()> {
        let state = Self::load_saved_state(src_path);
        Self::save_state_atomic(dest_path, &state)
    }

    /// Import state from an external JSON backup file.
    pub fn import_state(src_path: &Path, dest_path: &Path) -> io::Result<usize> {
        let state = Self::load_saved_state(src_path);
        let count = state.len();
        Self::save_state_atomic(dest_path, &state)?;
        Ok(count)
    }
}
