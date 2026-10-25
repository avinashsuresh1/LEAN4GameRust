use crate::engine::environment::Environment;
use crate::engine::verifier::Verifier;
use crate::game::registry::CurriculumRegistry;
use serde::{Deserialize, Serialize};
use std::io::{Read, Write};
use std::net::{TcpListener, TcpStream};
use std::process::Command;
use std::thread;
use std::time::Duration;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LeanToolchainInfo {
    pub lean_version: Option<String>,
    pub lake_version: Option<String>,
    pub elan_version: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VerificationRequest {
    pub world_id: String,
    pub level_num: usize,
    pub proof_script: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VerificationResponse {
    pub success: bool,
    pub completed: bool,
    pub message: String,
    pub goals: Vec<String>,
}

pub struct LocalLeanServer {
    pub port: u16,
}

impl LocalLeanServer {
    pub const DEFAULT_PORT: u16 = 8080;

    pub fn new(port: u16) -> Self {
        Self { port }
    }

    /// Detects if Lean 4, Lake, and Elan are installed on the local system.
    pub fn detect_toolchain() -> LeanToolchainInfo {
        let lean_version = Command::new("lean")
            .arg("--version")
            .output()
            .ok()
            .and_then(|out| String::from_utf8(out.stdout).ok())
            .map(|s| s.trim().to_string());

        let lake_version = Command::new("lake")
            .arg("--version")
            .output()
            .ok()
            .and_then(|out| String::from_utf8(out.stdout).ok())
            .map(|s| s.trim().to_string());

        let elan_version = Command::new("elan")
            .arg("--version")
            .output()
            .ok()
            .and_then(|out| String::from_utf8(out.stdout).ok())
            .map(|s| s.trim().to_string());

        LeanToolchainInfo {
            lean_version,
            lake_version,
            elan_version,
        }
    }

    /// Checks if a port on 127.0.0.1 is already accepting connections.
    pub fn is_port_active(port: u16) -> bool {
        TcpStream::connect(("127.0.0.1", port)).is_ok()
    }

    /// Returns the local server URL.
    pub fn local_url(port: u16) -> String {
        format!("http://127.0.0.1:{}/", port)
    }

    /// Generates the self-contained HTML client for offline play.
    pub fn embedded_html_client() -> &'static str {
        r###"<!DOCTYPE html>
<html lang="en">
<head>
    <meta charset="UTF-8">
    <meta name="viewport" content="width=device-width, initial-scale=1.0">
    <title>LEAN 4 Game - Self-Contained Local Desktop Edition</title>
    <style>
        :root {
            --bg-color: #1e1e2e;
            --sidebar-bg: #181825;
            --panel-bg: #11111b;
            --text-color: #cdd6f4;
            --accent-color: #89b4fa;
            --success-color: #a6e3a1;
            --error-color: #f38ba8;
            --border-color: #313244;
        }
        body {
            font-family: -apple-system, BlinkMacSystemFont, "Segoe UI", Roboto, "Helvetica Neue", Arial, sans-serif;
            background-color: var(--bg-color);
            color: var(--text-color);
            margin: 0;
            display: flex;
            height: 100vh;
            overflow: hidden;
        }
        #sidebar {
            width: 320px;
            background-color: var(--sidebar-bg);
            border-right: 1px solid var(--border-color);
            display: flex;
            flex-direction: column;
        }
        #sidebar-header {
            padding: 16px;
            border-bottom: 1px solid var(--border-color);
        }
        #sidebar-header h2 {
            margin: 0;
            font-size: 1.1rem;
            color: var(--accent-color);
        }
        #level-list {
            flex: 1;
            overflow-y: auto;
            padding: 8px;
        }
        .level-item {
            padding: 10px 12px;
            border-radius: 6px;
            cursor: pointer;
            margin-bottom: 4px;
            display: flex;
            justify-content: space-between;
            align-items: center;
        }
        .level-item:hover {
            background-color: var(--border-color);
        }
        .level-item.active {
            background-color: #45475a;
            font-weight: bold;
        }
        #main {
            flex: 1;
            display: flex;
            flex-direction: column;
        }
        #header {
            padding: 16px;
            background-color: var(--sidebar-bg);
            border-bottom: 1px solid var(--border-color);
            display: flex;
            justify-content: space-between;
            align-items: center;
        }
        #editor-container {
            flex: 1;
            display: flex;
        }
        #editor-pane {
            flex: 1;
            display: flex;
            flex-direction: column;
            border-right: 1px solid var(--border-color);
            background-color: var(--panel-bg);
        }
        #editor {
            flex: 1;
            padding: 16px;
            font-family: "JetBrains Mono", Consolas, "Fira Code", monospace;
            font-size: 15px;
            background: transparent;
            color: var(--text-color);
            border: none;
            resize: none;
            outline: none;
            line-height: 1.5;
        }
        #infoview-pane {
            width: 420px;
            background-color: var(--panel-bg);
            display: flex;
            flex-direction: column;
            padding: 16px;
            overflow-y: auto;
        }
        .infoview-header {
            font-weight: bold;
            color: var(--accent-color);
            margin-bottom: 8px;
            border-bottom: 1px solid var(--border-color);
            padding-bottom: 4px;
        }
        .goal-box {
            background-color: var(--sidebar-bg);
            padding: 12px;
            border-radius: 6px;
            font-family: monospace;
            margin-bottom: 12px;
            border-left: 4px solid var(--accent-color);
        }
        .status-badge {
            display: inline-block;
            padding: 4px 10px;
            border-radius: 12px;
            font-size: 0.85rem;
            font-weight: bold;
        }
        .status-badge.solved {
            background-color: #1e4620;
            color: var(--success-color);
        }
        .status-badge.unsolved {
            background-color: #3b2028;
            color: var(--error-color);
        }
        button.btn {
            background-color: var(--accent-color);
            color: #11111b;
            border: none;
            padding: 8px 16px;
            border-radius: 6px;
            font-weight: bold;
            cursor: pointer;
        }
        button.btn:hover {
            opacity: 0.9;
        }
    </style>
</head>
<body>
    <div id="sidebar">
        <div id="sidebar-header">
            <h2>Self-Contained Local LEAN 4 Server</h2>
            <div style="font-size: 0.8rem; color: #a6adc8; margin-top: 4px;">
                100% Offline Theorem Prover
            </div>
        </div>
        <div id="level-list"></div>
    </div>
    <div id="main">
        <div id="header">
            <div>
                <h3 id="theorem-title" style="margin: 0;">Select a Level</h3>
                <div id="theorem-target" style="font-family: monospace; color: var(--accent-color); margin-top: 4px;"></div>
            </div>
            <div>
                <span id="level-status" class="status-badge unsolved">Unsolved</span>
                <button class="btn" onclick="verifyProof()" style="margin-left: 12px;">Verify (Ctrl+Enter)</button>
            </div>
        </div>
        <div id="editor-container">
            <div id="editor-pane">
                <textarea id="editor" placeholder="-- Enter your Lean 4 proof tactics here (e.g. rfl, rw [add_zero])..."></textarea>
            </div>
            <div id="infoview-pane">
                <div class="infoview-header">Infoview: Tactic State</div>
                <div id="goals-container">
                    <div style="color: #6c7086; font-style: italic;">No active goals.</div>
                </div>
                <div class="infoview-header" style="margin-top: 20px;">Messages</div>
                <div id="messages-container" style="color: #a6adc8; font-size: 0.9rem;">Ready.</div>
            </div>
        </div>
    </div>

    <script>
        let currentWorld = "addition";
        let currentLevel = 1;
        let worldsData = null;

        async function init() {
            try {
                const res = await fetch("/api/games");
                worldsData = await res.json();
                renderSidebar();
                loadLevel(currentWorld, currentLevel);
            } catch(e) {
                document.getElementById("messages-container").innerText = "Failed loading games: " + e;
            }
        }

        function renderSidebar() {
            const list = document.getElementById("level-list");
            list.innerHTML = "";
            if (!worldsData) return;

            worldsData.forEach(w => {
                const worldHeader = document.createElement("div");
                worldHeader.style.padding = "8px 4px 4px 4px";
                worldHeader.style.fontWeight = "bold";
                worldHeader.style.color = "#a6adc8";
                worldHeader.style.fontSize = "0.85rem";
                worldHeader.innerText = w.name;
                list.appendChild(worldHeader);

                w.levels.forEach(lvl => {
                    const div = document.createElement("div");
                    div.className = "level-item" + (w.id === currentWorld && lvl.number === currentLevel ? " active" : "");
                    div.innerHTML = "<span>Level " + lvl.number + ": " + lvl.title + "</span>";
                    div.onclick = () => loadLevel(w.id, lvl.number);
                    list.appendChild(div);
                });
            });
        }

        function loadLevel(worldId, levelNum) {
            currentWorld = worldId;
            currentLevel = levelNum;
            renderSidebar();

            let targetLevel = null;
            if (worldsData) {
                worldsData.forEach(w => {
                    if (w.id === worldId) {
                        w.levels.forEach(l => {
                            if (l.number === levelNum) targetLevel = l;
                        });
                    }
                });
            }

            if (targetLevel) {
                document.getElementById("theorem-title").innerText = targetLevel.title;
                document.getElementById("theorem-target").innerText = "theorem " + targetLevel.theorem_name + " : " + targetLevel.target_str;
                
                const saved = localStorage.getItem("proof_" + worldId + "_" + levelNum) || targetLevel.reference_solution || "";
                document.getElementById("editor").value = saved;
                verifyProof();
            }
        }

        async function verifyProof() {
            const proofScript = document.getElementById("editor").value;
            localStorage.setItem("proof_" + currentWorld + "_" + currentLevel, proofScript);

            try {
                const res = await fetch("/api/verify", {
                    method: "POST",
                    headers: { "Content-Type": "application/json" },
                    body: JSON.stringify({
                        world_id: currentWorld,
                        level_num: currentLevel,
                        proof_script: proofScript
                    })
                });
                const data = await res.json();
                
                const statusBadge = document.getElementById("level-status");
                if (data.completed) {
                    statusBadge.innerText = "Solved";
                    statusBadge.className = "status-badge solved";
                } else {
                    statusBadge.innerText = "Unsolved";
                    statusBadge.className = "status-badge unsolved";
                }

                const goalsContainer = document.getElementById("goals-container");
                if (data.goals.length === 0 && data.completed) {
                    goalsContainer.innerHTML = '<div style="color: #a6e3a1; font-weight: bold;">Proof complete! No remaining goals.</div>';
                } else if (data.goals.length > 0) {
                    goalsContainer.innerHTML = data.goals.map((g, idx) => 
                        '<div class="goal-box">' +
                            '<div style="color: #89b4fa; font-weight: bold; margin-bottom: 4px;">Goal ' + (idx + 1) + '</div>' +
                            '<div>' + g + '</div>' +
                        '</div>'
                    ).join("");
                } else {
                    goalsContainer.innerHTML = '<div style="color: #6c7086; font-style: italic;">No goals open.</div>';
                }

                document.getElementById("messages-container").innerText = data.message;
            } catch(e) {
                document.getElementById("messages-container").innerText = "Verification error: " + e;
            }
        }

        document.getElementById("editor").addEventListener("keydown", (e) => {
            if ((e.ctrlKey || e.metaKey) && e.key === "Enter") {
                e.preventDefault();
                verifyProof();
            }
        });

        window.onload = init;
    </script>
</body>
</html>"###
    }

    /// Handles an incoming HTTP request.
    fn handle_connection(mut stream: TcpStream) {
        let mut buffer = [0u8; 16384];
        let bytes_read = match stream.read(&mut buffer) {
            Ok(n) if n > 0 => n,
            _ => return,
        };

        let request = String::from_utf8_lossy(&buffer[..bytes_read]);
        let mut lines = request.lines();
        let first_line = lines.next().unwrap_or("");
        let parts: Vec<&str> = first_line.split_whitespace().collect();

        if parts.len() < 2 {
            return;
        }

        let method = parts[0];
        let path = parts[1];

        match (method, path) {
            ("GET", "/") | ("GET", "/index.html") => {
                let html = Self::embedded_html_client();
                let response = format!(
                    "HTTP/1.1 200 OK\r\nContent-Type: text/html; charset=utf-8\r\nContent-Length: {}\r\nConnection: close\r\n\r\n{}",
                    html.len(),
                    html
                );
                let _ = stream.write_all(response.as_bytes());
            }
            ("GET", "/api/health") => {
                let json = r#"{"status":"ok","offline":true,"engine":"pure-rust","version":"0.1.0"}"#;
                let response = format!(
                    "HTTP/1.1 200 OK\r\nContent-Type: application/json\r\nContent-Length: {}\r\nConnection: close\r\n\r\n{}",
                    json.len(),
                    json
                );
                let _ = stream.write_all(response.as_bytes());
            }
            ("GET", "/api/games") => {
                let worlds = CurriculumRegistry::get_worlds();
                let json = serde_json::to_string(&worlds).unwrap_or_else(|_| "[]".to_string());
                let response = format!(
                    "HTTP/1.1 200 OK\r\nContent-Type: application/json\r\nContent-Length: {}\r\nConnection: close\r\n\r\n{}",
                    json.len(),
                    json
                );
                let _ = stream.write_all(response.as_bytes());
            }
            ("POST", "/api/verify") => {
                let body = if let Some(pos) = request.find("\r\n\r\n") {
                    &request[pos + 4..]
                } else if let Some(pos) = request.find("\n\n") {
                    &request[pos + 2..]
                } else {
                    ""
                };

                let resp_obj = match serde_json::from_str::<VerificationRequest>(body) {
                    Ok(req) => {
                        let worlds = CurriculumRegistry::get_worlds();
                        let found = worlds
                            .iter()
                            .find(|w| w.id == req.world_id)
                            .and_then(|w| w.levels.iter().find(|l| l.number == req.level_num));

                        match found {
                            Some(level) => {
                                let env = Environment::standard_peano();
                                let initial_state = level.build_initial_state();
                                let res = Verifier::verify_script(&initial_state, &req.proof_script, &env);

                                let open_goals: Vec<String> = res
                                    .final_state
                                    .goals
                                    .iter()
                                    .map(|g| g.target.to_string())
                                    .collect();

                                VerificationResponse {
                                    success: res.is_complete,
                                    completed: res.is_complete,
                                    message: if res.is_complete {
                                        "Proof complete! Level successfully solved.".to_string()
                                    } else if let Some(err) = res.first_error {
                                        format!("Tactic error: {}", err)
                                    } else {
                                        format!("Proof in progress: {} goal(s) remaining.", open_goals.len())
                                    },
                                    goals: open_goals,
                                }
                            }
                            None => VerificationResponse {
                                success: false,
                                completed: false,
                                message: format!("Level not found: world '{}' level {}", req.world_id, req.level_num),
                                goals: vec![],
                            },
                        }
                    }
                    Err(e) => VerificationResponse {
                        success: false,
                        completed: false,
                        message: format!("Malformed request body: {}", e),
                        goals: vec![],
                    },
                };

                let json = serde_json::to_string(&resp_obj).unwrap_or_else(|_| "{}".to_string());
                let response = format!(
                    "HTTP/1.1 200 OK\r\nContent-Type: application/json\r\nContent-Length: {}\r\nConnection: close\r\n\r\n{}",
                    json.len(),
                    json
                );
                let _ = stream.write_all(response.as_bytes());
            }
            _ => {
                let response = "HTTP/1.1 404 NOT FOUND\r\nContent-Length: 0\r\nConnection: close\r\n\r\n";
                let _ = stream.write_all(response.as_bytes());
            }
        }
    }

    /// Starts the local server in a background thread.
    /// Returns the active bound port or an error.
    pub fn start_in_background(port: u16) -> Result<u16, String> {
        let listener = TcpListener::bind(("127.0.0.1", port))
            .or_else(|_| TcpListener::bind(("127.0.0.1", 0)))
            .map_err(|e| format!("Failed to bind local server to 127.0.0.1: {}", e))?;

        let actual_port = listener
            .local_addr()
            .map_err(|e| format!("Failed to get local address: {}", e))?
            .port();

        thread::spawn(move || {
            for stream in listener.incoming() {
                if let Ok(stream) = stream {
                    thread::spawn(move || {
                        Self::handle_connection(stream);
                    });
                }
            }
        });

        // Small pause to ensure listener is ready
        thread::sleep(Duration::from_millis(50));
        Ok(actual_port)
    }
}
