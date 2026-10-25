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
pub struct GoalInfo {
    pub id: usize,
    pub tag: Option<String>,
    pub hypotheses: Vec<String>,
    pub target: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VerificationRequest {
    #[serde(default)]
    pub game_id: Option<String>,
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
    #[serde(default)]
    pub goal_infos: Vec<GoalInfo>,
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
    /// Provides an exact replica of the official Lean 4 Game with:
    /// - Complete Pedagogical Story & Dialogue formatted with Markdown definitions.
    /// - Interactive Step-by-Step Hints Drawer with toggleable reveal.
    /// - Tactics & Theorem Inventory Drawer.
    /// - Multi-Game Selector (NNG4, Real Analysis, Set Theory, Custom Games).
    /// - 3-Pane Layout with synchronized Line Numbers, Math Symbol Toolbar, and Lean Infoview.
    pub fn embedded_html_client() -> &'static str {
        r####"<!DOCTYPE html>
<html lang="en">
<head>
    <meta charset="UTF-8">
    <meta name="viewport" content="width=device-width, initial-scale=1.0">
    <title>Lean 4 Game - Offline Desktop Edition</title>
    <style>
        :root {
            --bg-base: #1e1e2e;
            --bg-mantle: #181825;
            --bg-crust: #11111b;
            --bg-surface: #313244;
            --bg-overlay: #45475a;
            --text-primary: #cdd6f4;
            --text-secondary: #a6adc8;
            --text-muted: #6c7086;
            --blue: #89b4fa;
            --green: #a6e3a1;
            --yellow: #f9e2af;
            --red: #f38ba8;
            --purple: #cba6f7;
            --border: #313244;
            --border-light: #45475a;
            --font-mono: "JetBrains Mono", Consolas, "Fira Code", monospace;
        }

        * {
            box-sizing: border-box;
            margin: 0;
            padding: 0;
        }

        body {
            font-family: -apple-system, BlinkMacSystemFont, "Segoe UI", Roboto, "Helvetica Neue", Arial, sans-serif;
            background-color: var(--bg-base);
            color: var(--text-primary);
            height: 100vh;
            display: flex;
            flex-direction: column;
            overflow: hidden;
            font-size: 14px;
        }

        /* Top Bar */
        #top-bar {
            height: 52px;
            background-color: var(--bg-crust);
            border-bottom: 1px solid var(--border);
            display: flex;
            align-items: center;
            justify-content: space-between;
            padding: 0 16px;
            flex-shrink: 0;
            user-select: none;
        }

        .top-left, .top-center, .top-right {
            display: flex;
            align-items: center;
            gap: 12px;
        }

        .logo-badge {
            font-weight: 700;
            font-size: 1rem;
            color: var(--blue);
            display: flex;
            align-items: center;
            gap: 6px;
        }

        select.game-dropdown {
            background-color: var(--bg-surface);
            color: var(--text-primary);
            border: 1px solid var(--border-light);
            border-radius: 6px;
            padding: 6px 12px;
            font-size: 0.9rem;
            font-weight: 600;
            cursor: pointer;
            outline: none;
        }

        select.game-dropdown:hover {
            border-color: var(--blue);
        }

        .breadcrumb {
            display: flex;
            align-items: center;
            gap: 6px;
            font-size: 0.85rem;
            color: var(--text-secondary);
        }

        .breadcrumb .sep {
            color: var(--text-muted);
        }

        .breadcrumb .current {
            color: var(--text-primary);
            font-weight: 600;
        }

        .nav-btn {
            background-color: var(--bg-surface);
            color: var(--text-primary);
            border: 1px solid var(--border-light);
            padding: 5px 12px;
            border-radius: 6px;
            cursor: pointer;
            font-size: 0.85rem;
            font-weight: 600;
            transition: all 0.15s ease;
        }

        .nav-btn:hover:not(:disabled) {
            background-color: var(--bg-overlay);
            border-color: var(--blue);
            color: #fff;
        }

        .nav-btn:disabled {
            opacity: 0.4;
            cursor: not-allowed;
        }

        .level-pill {
            font-weight: 600;
            color: var(--blue);
            font-size: 0.85rem;
            background: var(--bg-surface);
            padding: 5px 10px;
            border-radius: 6px;
            border: 1px solid var(--border);
        }

        .progress-pill {
            display: flex;
            align-items: center;
            gap: 5px;
            font-size: 0.85rem;
            font-weight: 600;
            color: var(--yellow);
            background: rgba(249, 226, 175, 0.1);
            border: 1px solid rgba(249, 226, 175, 0.3);
            padding: 4px 10px;
            border-radius: 12px;
        }

        .offline-tag {
            font-size: 0.8rem;
            color: var(--green);
            background: rgba(166, 227, 161, 0.12);
            border: 1px solid rgba(166, 227, 161, 0.3);
            padding: 4px 8px;
            border-radius: 12px;
            font-weight: 600;
        }

        /* 3-Column Layout */
        #workspace {
            flex: 1;
            display: flex;
            overflow: hidden;
        }

        .panel {
            display: flex;
            flex-direction: column;
            overflow: hidden;
            background-color: var(--bg-base);
        }

        /* Column 1: Guide / Dialogue / Hints / Inventory */
        #guide-panel {
            width: 400px;
            min-width: 320px;
            max-width: 480px;
            background-color: var(--bg-mantle);
            border-right: 1px solid var(--border);
        }

        .panel-nav-tabs {
            display: flex;
            background-color: var(--bg-crust);
            border-bottom: 1px solid var(--border);
            user-select: none;
            overflow-x: auto;
        }

        .panel-tab {
            flex: 1;
            padding: 10px 8px;
            text-align: center;
            font-size: 0.82rem;
            font-weight: 600;
            color: var(--text-secondary);
            border: none;
            background: transparent;
            cursor: pointer;
            border-bottom: 2px solid transparent;
            white-space: nowrap;
            transition: all 0.15s ease;
        }

        .panel-tab:hover {
            color: var(--text-primary);
            background-color: var(--bg-mantle);
        }

        .panel-tab.active {
            color: var(--blue);
            border-bottom-color: var(--blue);
            background-color: var(--bg-mantle);
        }

        .tab-view {
            display: none;
            flex: 1;
            overflow-y: auto;
            padding: 16px;
        }

        .tab-view.active {
            display: flex;
            flex-direction: column;
            gap: 16px;
        }

        /* Story & Dialogue Styling */
        .theorem-box {
            background-color: var(--bg-crust);
            border: 1px solid var(--border);
            border-left: 4px solid var(--blue);
            border-radius: 6px;
            padding: 12px 14px;
        }

        .theorem-box-label {
            font-size: 0.75rem;
            text-transform: uppercase;
            font-weight: 700;
            color: var(--blue);
            letter-spacing: 0.5px;
            margin-bottom: 4px;
        }

        .theorem-box-content {
            font-family: var(--font-mono);
            font-size: 0.92rem;
            color: var(--text-primary);
            word-break: break-all;
        }

        .dialogue-card {
            background-color: var(--bg-crust);
            border: 1px solid var(--border);
            border-radius: 8px;
            padding: 14px;
            display: flex;
            flex-direction: column;
            gap: 10px;
        }

        .dialogue-header {
            display: flex;
            align-items: center;
            gap: 8px;
            font-weight: 700;
            color: var(--yellow);
            font-size: 0.9rem;
            border-bottom: 1px solid var(--border);
            padding-bottom: 8px;
        }

        .dialogue-text {
            font-size: 0.9rem;
            line-height: 1.6;
            color: var(--text-primary);
        }

        .dialogue-text p {
            margin-bottom: 10px;
        }

        .dialogue-text p:last-child {
            margin-bottom: 0;
        }

        .dialogue-text strong {
            color: #fff;
        }

        code.inline-code {
            font-family: var(--font-mono);
            background-color: var(--bg-surface);
            color: var(--blue);
            padding: 2px 6px;
            border-radius: 4px;
            font-size: 0.85em;
        }

        .hints-callout {
            background: rgba(249, 226, 175, 0.08);
            border: 1px solid rgba(249, 226, 175, 0.3);
            border-radius: 6px;
            padding: 10px 14px;
            display: flex;
            justify-content: space-between;
            align-items: center;
            cursor: pointer;
            transition: all 0.15s ease;
        }

        .hints-callout:hover {
            background: rgba(249, 226, 175, 0.15);
            border-color: var(--yellow);
        }

        .chips-container {
            display: flex;
            flex-wrap: wrap;
            gap: 6px;
            margin-top: 6px;
        }

        .tactic-chip {
            background-color: var(--bg-surface);
            color: var(--blue);
            border: 1px solid var(--border-light);
            border-radius: 4px;
            padding: 4px 10px;
            font-family: var(--font-mono);
            font-size: 0.82rem;
            font-weight: 600;
            cursor: pointer;
            transition: all 0.15s ease;
        }

        .tactic-chip:hover {
            background-color: var(--blue);
            color: var(--bg-crust);
        }

        /* Hints Accordion */
        .hint-accordion-card {
            background-color: var(--bg-crust);
            border: 1px solid var(--border);
            border-radius: 6px;
            overflow: hidden;
            margin-bottom: 10px;
        }

        .hint-header {
            padding: 10px 14px;
            background-color: var(--bg-surface);
            cursor: pointer;
            font-weight: 600;
            font-size: 0.88rem;
            display: flex;
            justify-content: space-between;
            align-items: center;
            color: var(--yellow);
            user-select: none;
        }

        .hint-header:hover {
            background-color: var(--bg-overlay);
        }

        .hint-body {
            padding: 12px 14px;
            display: none;
            font-size: 0.9rem;
            line-height: 1.5;
            color: var(--text-primary);
            border-top: 1px solid var(--border);
        }

        .hint-body.open {
            display: block;
        }

        /* Column 2: Code Editor */
        #editor-panel {
            flex: 1;
            min-width: 380px;
            border-right: 1px solid var(--border);
            display: flex;
            flex-direction: column;
            background-color: var(--bg-base);
        }

        .editor-top {
            padding: 10px 16px;
            background-color: var(--bg-mantle);
            border-bottom: 1px solid var(--border);
            display: flex;
            justify-content: space-between;
            align-items: center;
        }

        .editor-theorem-preview {
            font-family: var(--font-mono);
            font-size: 0.88rem;
            color: var(--blue);
            white-space: nowrap;
            overflow: hidden;
            text-overflow: ellipsis;
            max-width: 50%;
        }

        .symbol-toolbar {
            display: flex;
            align-items: center;
            gap: 4px;
        }

        .sym-btn {
            background: var(--bg-surface);
            color: var(--text-primary);
            border: 1px solid var(--border);
            border-radius: 4px;
            padding: 2px 7px;
            font-family: var(--font-mono);
            font-size: 0.85rem;
            cursor: pointer;
            transition: all 0.15s ease;
        }

        .sym-btn:hover {
            background: var(--bg-overlay);
            color: var(--blue);
            border-color: var(--blue);
        }

        .editor-workspace {
            flex: 1;
            display: flex;
            position: relative;
            background-color: var(--bg-crust);
            overflow: hidden;
        }

        .line-numbers {
            width: 44px;
            padding: 14px 8px 14px 0;
            text-align: right;
            font-family: var(--font-mono);
            font-size: 14px;
            line-height: 1.5;
            color: var(--text-muted);
            background-color: var(--bg-crust);
            border-right: 1px solid var(--border);
            user-select: none;
            overflow: hidden;
            white-space: pre-line;
        }

        .code-textarea {
            flex: 1;
            padding: 14px;
            font-family: var(--font-mono);
            font-size: 14px;
            line-height: 1.5;
            background: transparent;
            color: var(--text-primary);
            border: none;
            resize: none;
            outline: none;
            white-space: pre;
            overflow-wrap: normal;
            overflow-x: auto;
            tab-size: 2;
        }

        .editor-bottom {
            padding: 10px 16px;
            background-color: var(--bg-mantle);
            border-top: 1px solid var(--border);
            display: flex;
            justify-content: space-between;
            align-items: center;
        }

        .status-badge {
            display: inline-flex;
            align-items: center;
            gap: 6px;
            padding: 4px 12px;
            border-radius: 12px;
            font-size: 0.82rem;
            font-weight: 700;
        }

        .status-badge.unsolved {
            background-color: rgba(243, 139, 168, 0.15);
            color: var(--red);
            border: 1px solid rgba(243, 139, 168, 0.3);
        }

        .status-badge.solved {
            background-color: rgba(166, 227, 161, 0.15);
            color: var(--green);
            border: 1px solid rgba(166, 227, 161, 0.3);
        }

        .btn-verify {
            background-color: var(--blue);
            color: var(--bg-crust);
            border: none;
            padding: 8px 18px;
            border-radius: 6px;
            font-weight: 700;
            font-size: 0.88rem;
            cursor: pointer;
            transition: all 0.15s ease;
        }

        .btn-verify:hover {
            opacity: 0.9;
            transform: translateY(-1px);
        }

        /* Column 3: Lean 4 Infoview */
        #infoview-panel {
            width: 380px;
            min-width: 300px;
            max-width: 460px;
            background-color: var(--bg-mantle);
            display: flex;
            flex-direction: column;
        }

        .infoview-header {
            padding: 10px 16px;
            background-color: var(--bg-crust);
            border-bottom: 1px solid var(--border);
            display: flex;
            justify-content: space-between;
            align-items: center;
        }

        .infoview-title {
            font-weight: 700;
            font-size: 0.88rem;
            color: var(--text-primary);
        }

        .infoview-tag {
            font-size: 0.75rem;
            background-color: var(--bg-surface);
            color: var(--blue);
            padding: 3px 8px;
            border-radius: 4px;
            font-weight: 600;
        }

        .infoview-body {
            flex: 1;
            padding: 16px;
            overflow-y: auto;
            display: flex;
            flex-direction: column;
            gap: 14px;
        }

        /* Infoview Cards */
        .celebration-card {
            background: linear-gradient(135deg, rgba(166, 227, 161, 0.15), rgba(137, 180, 250, 0.15));
            border: 1px solid var(--green);
            border-radius: 8px;
            padding: 18px;
            text-align: center;
            display: flex;
            flex-direction: column;
            align-items: center;
            gap: 10px;
        }

        .celebration-icon {
            font-size: 2.2rem;
        }

        .celebration-title {
            font-size: 1.15rem;
            font-weight: 700;
            color: var(--green);
        }

        .celebration-desc {
            font-size: 0.88rem;
            color: var(--text-secondary);
        }

        .celebration-btn {
            margin-top: 4px;
            background-color: var(--green);
            color: var(--bg-crust);
            border: none;
            padding: 8px 18px;
            border-radius: 6px;
            font-weight: 700;
            font-size: 0.9rem;
            cursor: pointer;
            transition: all 0.15s ease;
        }

        .celebration-btn:hover {
            opacity: 0.9;
            transform: scale(1.02);
        }

        .goal-box {
            background-color: var(--bg-crust);
            border: 1px solid var(--border);
            border-radius: 6px;
            padding: 12px;
            font-family: var(--font-mono);
            font-size: 0.88rem;
        }

        .goal-case-tag {
            color: var(--yellow);
            font-weight: 700;
            margin-bottom: 8px;
            font-size: 0.82rem;
        }

        .goal-hypotheses {
            border-bottom: 1px solid var(--border);
            padding-bottom: 8px;
            margin-bottom: 8px;
            display: flex;
            flex-direction: column;
            gap: 4px;
        }

        .hyp-line {
            color: var(--text-secondary);
        }

        .hyp-name {
            color: var(--blue);
            font-weight: 600;
        }

        .hyp-type {
            color: var(--text-primary);
        }

        .goal-target {
            color: var(--text-primary);
            font-weight: 600;
            display: flex;
            align-items: baseline;
            gap: 6px;
        }

        .turnstile {
            color: var(--blue);
            font-weight: 800;
            font-size: 1.05rem;
        }

        .tactic-error-card {
            background-color: rgba(243, 139, 168, 0.1);
            border: 1px solid var(--red);
            border-radius: 6px;
            padding: 12px;
            color: var(--red);
            font-size: 0.88rem;
            line-height: 1.4;
        }

        .error-title {
            font-weight: 700;
            margin-bottom: 4px;
            display: flex;
            align-items: center;
            gap: 6px;
        }

        .infoview-messages-bar {
            padding: 8px 16px;
            background-color: var(--bg-crust);
            border-top: 1px solid var(--border);
            font-size: 0.8rem;
            color: var(--text-muted);
            white-space: nowrap;
            overflow: hidden;
            text-overflow: ellipsis;
        }

        /* Worlds & Theorems Views */
        .world-tree-item {
            border: 1px solid var(--border);
            border-radius: 6px;
            overflow: hidden;
            margin-bottom: 10px;
            background-color: var(--bg-crust);
        }

        .world-tree-header {
            padding: 10px 12px;
            background-color: var(--bg-surface);
            font-weight: 700;
            font-size: 0.88rem;
            display: flex;
            justify-content: space-between;
            align-items: center;
            cursor: pointer;
        }

        .world-levels-list {
            padding: 6px;
            display: flex;
            flex-direction: column;
            gap: 3px;
        }

        .level-row {
            padding: 7px 10px;
            border-radius: 4px;
            cursor: pointer;
            display: flex;
            justify-content: space-between;
            align-items: center;
            font-size: 0.85rem;
            transition: all 0.15s ease;
        }

        .level-row:hover {
            background-color: var(--bg-surface);
        }

        .level-row.active {
            background-color: var(--bg-overlay);
            font-weight: 700;
            color: var(--blue);
        }

        .item-inventory {
            background-color: var(--bg-crust);
            border: 1px solid var(--border);
            border-radius: 6px;
            padding: 10px 12px;
            margin-bottom: 8px;
            cursor: pointer;
            transition: all 0.15s ease;
        }

        .item-inventory:hover {
            border-color: var(--blue);
            background-color: var(--bg-surface);
        }

        .item-inventory-title {
            font-family: var(--font-mono);
            font-weight: 700;
            color: var(--blue);
            font-size: 0.85rem;
            margin-bottom: 3px;
        }

        .item-inventory-doc {
            font-size: 0.82rem;
            color: var(--text-secondary);
        }
    </style>
</head>
<body>
    <!-- Top Bar -->
    <header id="top-bar">
        <div class="top-left">
            <span class="logo-badge">Lean 4 Game</span>
            <select id="game-select" class="game-dropdown" onchange="onGameSelected(this.value)">
                <!-- Populated dynamically -->
            </select>
            <div class="breadcrumb">
                <span id="bc-world">Addition World</span>
                <span class="sep">›</span>
                <span id="bc-level" class="current">Level 1</span>
            </div>
        </div>
        <div class="top-center">
            <button id="btn-prev" class="nav-btn" onclick="prevLevel()">‹ Prev</button>
            <div id="level-pill" class="level-pill">Level 1 / 8</div>
            <button id="btn-next" class="nav-btn" onclick="nextLevel()">Next ›</button>
        </div>
        <div class="top-right">
            <div class="progress-pill">
                <span>⭐</span>
                <span id="solved-count">0</span> / <span id="total-count">0</span>
            </div>
            <button class="nav-btn" onclick="resetProofCode()">↺ Reset</button>
            <span class="offline-tag">🟢 100% Offline</span>
        </div>
    </header>

    <!-- Main Workspace -->
    <main id="workspace">
        <!-- Column 1: Guide / Story / Hints / Inventory -->
        <section id="guide-panel" class="panel">
            <nav class="panel-nav-tabs">
                <button id="tab-btn-story" class="panel-tab active" onclick="switchLeftTab('story')">💬 Story</button>
                <button id="tab-btn-hints" class="panel-tab" onclick="switchLeftTab('hints')">💡 Hints (<span id="hints-counter">0</span>)</button>
                <button id="tab-btn-tactics" class="panel-tab" onclick="switchLeftTab('tactics')">🛠️ Tactics</button>
                <button id="tab-btn-theorems" class="panel-tab" onclick="switchLeftTab('theorems')">📚 Theorems</button>
                <button id="tab-btn-worlds" class="panel-tab" onclick="switchLeftTab('worlds')">🗺️ Worlds</button>
            </nav>

            <!-- Story Tab -->
            <div id="tab-view-story" class="tab-view active">
                <div class="theorem-box">
                    <div class="theorem-box-label">Current Theorem</div>
                    <div id="theorem-signature" class="theorem-box-content">theorem ...</div>
                </div>

                <div class="dialogue-card">
                    <div class="dialogue-header">
                        <span>🧑‍🏫</span>
                        <span id="dialogue-title">Pedagogical Story & Dialogue</span>
                    </div>
                    <div id="dialogue-body" class="dialogue-text">
                        <!-- Rendered Markdown from level.doc -->
                    </div>
                </div>

                <div id="hints-callout-box" class="hints-callout" onclick="switchLeftTab('hints')">
                    <span style="font-weight: 600; color: var(--yellow);">💡 Step-by-step hints available</span>
                    <span style="font-size: 0.8rem; color: var(--text-secondary);">View Hints ➔</span>
                </div>

                <div>
                    <div style="font-size: 0.82rem; font-weight: 700; color: var(--text-secondary); text-transform: uppercase;">
                        Available Tactics for this Level
                    </div>
                    <div id="level-tactics-chips" class="chips-container">
                        <!-- Populated dynamically -->
                    </div>
                </div>
            </div>

            <!-- Hints Tab -->
            <div id="tab-view-hints" class="tab-view">
                <div style="display: flex; justify-content: space-between; align-items: center;">
                    <div style="font-weight: 700; font-size: 0.95rem; color: var(--yellow);">Step-by-Step Guidance</div>
                    <button class="nav-btn" style="padding: 3px 8px; font-size: 0.78rem;" onclick="revealAllHints()">Reveal All</button>
                </div>
                <div id="hints-list">
                    <!-- Populated dynamically -->
                </div>
                <div style="margin-top: 10px; padding: 12px; background: var(--bg-crust); border: 1px solid var(--border); border-radius: 6px;">
                    <div style="display: flex; justify-content: space-between; align-items: center;">
                        <span style="font-size: 0.85rem; font-weight: 600; color: var(--text-secondary);">Stuck on this proof?</span>
                        <button class="nav-btn" style="padding: 3px 8px; font-size: 0.78rem;" onclick="toggleSolutionBox()">Show Solution</button>
                    </div>
                    <div id="solution-box" style="display: none; margin-top: 10px;">
                        <pre id="solution-code" style="font-family: var(--font-mono); font-size: 0.85rem; background: var(--bg-surface); padding: 8px; border-radius: 4px; overflow-x: auto; color: var(--text-primary);"></pre>
                        <button class="nav-btn" style="margin-top: 6px; width: 100%;" onclick="insertSolution()">Insert Solution into Editor</button>
                    </div>
                </div>
            </div>

            <!-- Tactics Reference Tab -->
            <div id="tab-view-tactics" class="tab-view">
                <div style="font-size: 0.85rem; color: var(--text-secondary); margin-bottom: 8px;">
                    Click any tactic to insert a snippet into your proof editor.
                </div>
                <div id="tactics-reference-list">
                    <!-- Populated dynamically -->
                </div>
            </div>

            <!-- Theorems Inventory Tab -->
            <div id="tab-view-theorems" class="tab-view">
                <div style="font-size: 0.85rem; color: var(--text-secondary); margin-bottom: 8px;">
                    Axioms and previously proved theorems in this game. Click to insert <code>rw [thm]</code>.
                </div>
                <div id="theorems-inventory-list">
                    <!-- Populated dynamically -->
                </div>
            </div>

            <!-- Worlds Map Tab -->
            <div id="tab-view-worlds" class="tab-view">
                <div id="worlds-map-list">
                    <!-- Populated dynamically -->
                </div>
            </div>
        </section>

        <!-- Column 2: Code Editor -->
        <section id="editor-panel" class="panel">
            <div class="editor-top">
                <span id="editor-theorem-header" class="editor-theorem-preview">theorem ...</span>
                <div class="symbol-toolbar">
                    <span style="font-size: 0.8rem; color: var(--text-muted); margin-right: 4px;">Insert:</span>
                    <button class="sym-btn" onclick="insertSym('⊢')" title="Turnstile">⊢</button>
                    <button class="sym-btn" onclick="insertSym('←')" title="Reverse rewrite">←</button>
                    <button class="sym-btn" onclick="insertSym('·')" title="Bullet">·</button>
                    <button class="sym-btn" onclick="insertSym('ℕ')" title="Natural numbers">ℕ</button>
                    <button class="sym-btn" onclick="insertSym('ℝ')" title="Real numbers">ℝ</button>
                    <button class="sym-btn" onclick="insertSym('≤')" title="Less or equal">≤</button>
                    <button class="sym-btn" onclick="insertSym('≥')" title="Greater or equal">≥</button>
                    <button class="sym-btn" onclick="insertSym('→')" title="Implies">→</button>
                    <button class="sym-btn" onclick="insertSym('∀')" title="For all">∀</button>
                    <button class="sym-btn" onclick="insertSym('∃')" title="Exists">∃</button>
                    <button class="sym-btn" onclick="insertSym('≠')" title="Not equal">≠</button>
                </div>
            </div>
            <div class="editor-workspace">
                <div id="line-numbers" class="line-numbers">1</div>
                <textarea id="code-editor" class="code-textarea" spellcheck="false" placeholder="-- Enter proof tactics here (e.g. rfl, rw [add_zero])..."></textarea>
            </div>
            <div class="editor-bottom">
                <div style="display: flex; align-items: center; gap: 10px;">
                    <span id="status-badge" class="status-badge unsolved">Unsolved</span>
                    <span style="font-size: 0.8rem; color: var(--text-muted);">Press <strong>Ctrl+Enter</strong> to verify</span>
                </div>
                <button class="btn-verify" onclick="verifyProof()">Verify Proof (Ctrl+Enter)</button>
            </div>
        </section>

        <!-- Column 3: Lean 4 Infoview -->
        <section id="infoview-panel" class="panel">
            <div class="infoview-header">
                <span class="infoview-title">Lean 4 Infoview</span>
                <span class="infoview-tag">Tactic State</span>
            </div>
            <div id="infoview-body" class="infoview-body">
                <!-- Goals / Hypotheses / Errors / Celebration banner -->
            </div>
            <div id="infoview-messages" class="infoview-messages-bar">Ready.</div>
        </section>
    </main>

    <script>
        let gamesData = [];
        let currentGameId = "natural_numbers";
        let currentWorldId = "tutorial";
        let currentLevelNum = 1;

        const TACTICS_GUIDE = [
            { name: "rfl", syntax: "rfl", doc: "Reflexivity. Closes a goal of the form A = A or definitionally equal expressions." },
            { name: "rw", syntax: "rw [lemma]", doc: "Rewrite. Substitutes the left side of equality 'lemma' with its right side." },
            { name: "rw [← ...]", syntax: "rw [← lemma]", doc: "Reverse rewrite. Substitutes the right side of equality 'lemma' with its left side." },
            { name: "intro", syntax: "intro h", doc: "Introduce assumption. Moves an antecedent from implication or a universal variable into context." },
            { name: "exact", syntax: "exact h", doc: "Exact match. Closes the goal if hypothesis or theorem 'h' matches the target exactly." },
            { name: "apply", syntax: "apply f", doc: "Modus ponens. Backwards deduction using an implication f : P → Q to turn goal Q into P." },
            { name: "induction", syntax: "induction n with d hd", doc: "Mathematical induction. Splits goal on natural number n into base case 0 and inductive step succ d." },
            { name: "sorry", syntax: "sorry", doc: "Admit goal. Temporarily bypasses proof verification for the active goal." }
        ];

        async function init() {
            try {
                const res = await fetch("/api/games");
                gamesData = await res.json();

                renderGameSelector();
                renderTacticsReference();

                // Restore last viewed game/world/level from localStorage if present
                const lastGame = localStorage.getItem("lean4game_last_game");
                const lastWorld = localStorage.getItem("lean4game_last_world");
                const lastLevel = parseInt(localStorage.getItem("lean4game_last_level") || "1", 10);

                if (lastGame && gamesData.some(g => g.id === lastGame)) {
                    currentGameId = lastGame;
                    const g = gamesData.find(g => g.id === lastGame);
                    if (lastWorld && g.worlds.some(w => w.id === lastWorld)) {
                        currentWorldId = lastWorld;
                        currentLevelNum = lastLevel;
                    } else if (g.worlds.length > 0) {
                        currentWorldId = g.worlds[0].id;
                        currentLevelNum = 1;
                    }
                } else if (gamesData.length > 0) {
                    currentGameId = gamesData[0].id;
                    if (gamesData[0].worlds.length > 0) {
                        currentWorldId = gamesData[0].worlds[0].id;
                        currentLevelNum = 1;
                    }
                }

                document.getElementById("game-select").value = currentGameId;
                loadLevel(currentGameId, currentWorldId, currentLevelNum);
            } catch (e) {
                document.getElementById("infoview-messages").innerText = "Failed loading games: " + e;
            }
        }

        function escapeHtml(str) {
            if (!str) return "";
            return str
                .replace(/&/g, "&amp;")
                .replace(/</g, "&lt;")
                .replace(/>/g, "&gt;")
                .replace(/"/g, "&quot;")
                .replace(/'/g, "&#039;");
        }

        function formatMarkdown(text) {
            if (!text) return "";
            let html = escapeHtml(text);
            html = html.replace(/\*\*(.*?)\*\*/g, '<strong>$1</strong>');
            html = html.replace(/\*(.*?)\*/g, '<em>$1</em>');
            html = html.replace(/`([^`]+)`/g, '<code class="inline-code">$1</code>');
            const paras = html.split(/\n\n+/);
            return paras.map(p => {
                const trimmed = p.trim();
                if (!trimmed) return "";
                return '<p>' + trimmed.replace(/\n/g, '<br>') + '</p>';
            }).join("");
        }

        function renderGameSelector() {
            const select = document.getElementById("game-select");
            select.innerHTML = "";
            gamesData.forEach(g => {
                const opt = document.createElement("option");
                opt.value = g.id;
                opt.innerText = (g.icon || "🎮") + " " + g.title;
                select.appendChild(opt);
            });
        }

        function onGameSelected(gameId) {
            currentGameId = gameId;
            const game = gamesData.find(g => g.id === gameId);
            if (game && game.worlds.length > 0) {
                currentWorldId = game.worlds[0].id;
                currentLevelNum = 1;
                loadLevel(currentGameId, currentWorldId, currentLevelNum);
            }
        }

        function switchLeftTab(tabName) {
            const tabs = ["story", "hints", "tactics", "theorems", "worlds"];
            tabs.forEach(t => {
                const btn = document.getElementById("tab-btn-" + t);
                const view = document.getElementById("tab-view-" + t);
                if (btn) btn.classList.toggle("active", t === tabName);
                if (view) view.classList.toggle("active", t === tabName);
            });
        }

        function getCurrentGame() {
            return gamesData.find(g => g.id === currentGameId);
        }

        function getCurrentWorld() {
            const game = getCurrentGame();
            return game ? game.worlds.find(w => w.id === currentWorldId) : null;
        }

        function getCurrentLevel() {
            const world = getCurrentWorld();
            return world ? world.levels.find(l => l.number === currentLevelNum) : null;
        }

        function loadLevel(gameId, worldId, levelNum) {
            currentGameId = gameId;
            currentWorldId = worldId;
            currentLevelNum = levelNum;

            localStorage.setItem("lean4game_last_game", gameId);
            localStorage.setItem("lean4game_last_world", worldId);
            localStorage.setItem("lean4game_last_level", levelNum.toString());

            const game = getCurrentGame();
            const world = getCurrentWorld();
            const level = getCurrentLevel();

            if (!game || !world || !level) return;

            // Update Breadcrumbs & Navigation Header
            document.getElementById("bc-world").innerText = (world.icon || "") + " " + world.name;
            document.getElementById("bc-level").innerText = "Level " + level.number + ": " + level.title;
            document.getElementById("level-pill").innerText = "Level " + level.number + " / " + world.levels.length;

            // Nav buttons state
            const isFirst = (worldId === game.worlds[0].id && levelNum === 1);
            const isLast = (worldId === game.worlds[game.worlds.length - 1].id && levelNum === world.levels[world.levels.length - 1].number);
            document.getElementById("btn-prev").disabled = isFirst;
            document.getElementById("btn-next").disabled = isLast;

            // Update Story & Theorem Box
            const varsStr = level.context_vars && level.context_vars.length > 0
                ? " (" + level.context_vars.map(cv => cv[0] + " : " + cv[1]).join(" ") + ")"
                : "";
            const sig = "theorem " + level.theorem_name + varsStr + " : " + level.target_str;
            document.getElementById("theorem-signature").innerText = sig;
            document.getElementById("editor-theorem-header").innerText = sig + " := by";

            document.getElementById("dialogue-title").innerText = world.name + " — " + level.title;
            document.getElementById("dialogue-body").innerHTML = formatMarkdown(level.doc);

            // Update Hints
            const hintsCount = (level.hints || []).length;
            document.getElementById("hints-counter").innerText = hintsCount;
            document.getElementById("hints-callout-box").style.display = hintsCount > 0 ? "flex" : "none";
            renderHintsList(level.hints || []);

            // Reference solution box reset
            document.getElementById("solution-box").style.display = "none";
            document.getElementById("solution-code").innerText = level.reference_solution || "";

            // Available Tactics Chips
            renderLevelTacticsChips(level.available_tactics || []);

            // Render Unlocked Theorems Inventory
            renderTheoremsInventory();

            // Render Worlds Map
            renderWorldsMap();

            // Update Progress Pill
            updateProgressPill();

            // Load proof script from localStorage or initial template
            const saveKey = "lean4game_proof_" + gameId + "_" + worldId + "_" + levelNum;
            const saved = localStorage.getItem(saveKey);
            const starter = saved !== null ? saved : "-- Write your proof tactics here\n";
            document.getElementById("code-editor").value = starter;
            updateLineNumbers();

            // Reset solved status badge
            const isSolved = localStorage.getItem("lean4game_solved_" + gameId + "_" + worldId + "_" + levelNum) === "true";
            updateStatusBadge(isSolved);

            // Auto verify to populate Infoview with initial goals
            verifyProof();
        }

        function updateStatusBadge(isSolved) {
            const badge = document.getElementById("status-badge");
            if (isSolved) {
                badge.className = "status-badge solved";
                badge.innerText = "Solved ⭐";
            } else {
                badge.className = "status-badge unsolved";
                badge.innerText = "Unsolved";
            }
        }

        function updateProgressPill() {
            let total = 0;
            let solved = 0;
            const game = getCurrentGame();
            if (!game) return;

            game.worlds.forEach(w => {
                w.levels.forEach(l => {
                    total++;
                    if (localStorage.getItem("lean4game_solved_" + game.id + "_" + w.id + "_" + l.number) === "true") {
                        solved++;
                    }
                });
            });

            document.getElementById("solved-count").innerText = solved;
            document.getElementById("total-count").innerText = total;
        }

        function renderHintsList(hints) {
            const container = document.getElementById("hints-list");
            container.innerHTML = "";
            if (hints.length === 0) {
                container.innerHTML = '<div style="color: var(--text-muted); font-style: italic; padding: 8px 0;">No hints needed for this introductory level!</div>';
                return;
            }

            hints.forEach((h, idx) => {
                const card = document.createElement("div");
                card.className = "hint-accordion-card";

                const header = document.createElement("div");
                header.className = "hint-header";
                header.innerHTML = "<span>💡 Hint " + (idx + 1) + "</span><span class='hint-toggle-icon'>▼</span>";

                const body = document.createElement("div");
                body.className = "hint-body";
                body.innerHTML = formatMarkdown(h);

                header.onclick = () => {
                    const isOpen = body.classList.toggle("open");
                    header.querySelector(".hint-toggle-icon").innerText = isOpen ? "▲" : "▼";
                };

                card.appendChild(header);
                card.appendChild(body);
                container.appendChild(card);
            });
        }

        function revealAllHints() {
            document.querySelectorAll("#hints-list .hint-body").forEach(b => b.classList.add("open"));
            document.querySelectorAll("#hints-list .hint-toggle-icon").forEach(i => i.innerText = "▲");
        }

        function toggleSolutionBox() {
            const box = document.getElementById("solution-box");
            box.style.display = box.style.display === "none" ? "block" : "none";
        }

        function insertSolution() {
            const level = getCurrentLevel();
            if (level && level.reference_solution) {
                document.getElementById("code-editor").value = level.reference_solution;
                updateLineNumbers();
                verifyProof();
            }
        }

        function resetProofCode() {
            const level = getCurrentLevel();
            if (!level) return;
            document.getElementById("code-editor").value = "-- Write your proof tactics here\n";
            updateLineNumbers();
            verifyProof();
        }

        function renderLevelTacticsChips(tactics) {
            const container = document.getElementById("level-tactics-chips");
            container.innerHTML = "";
            tactics.forEach(t => {
                const chip = document.createElement("button");
                chip.className = "tactic-chip";
                chip.innerText = t;
                chip.onclick = () => insertSnippet(t);
                container.appendChild(chip);
            });
        }

        function renderTacticsReference() {
            const container = document.getElementById("tactics-reference-list");
            container.innerHTML = "";
            TACTICS_GUIDE.forEach(t => {
                const item = document.createElement("div");
                item.className = "item-inventory";
                item.innerHTML = '<div class="item-inventory-title">' + escapeHtml(t.syntax) + '</div>' +
                                  '<div class="item-inventory-doc">' + escapeHtml(t.doc) + '</div>';
                item.onclick = () => insertSnippet(t.name);
                container.appendChild(item);
            });
        }

        function renderTheoremsInventory() {
            const container = document.getElementById("theorems-inventory-list");
            container.innerHTML = "";
            const game = getCurrentGame();
            if (!game) return;

            // Collect standard axioms and proved theorems
            const theorems = [];
            game.worlds.forEach(w => {
                w.levels.forEach(l => {
                    const isSolved = localStorage.getItem("lean4game_solved_" + game.id + "_" + w.id + "_" + l.number) === "true";
                    theorems.push({
                        name: l.theorem_name,
                        target: l.target_str,
                        doc: l.title,
                        is_solved: isSolved
                    });
                });
            });

            theorems.forEach(thm => {
                const div = document.createElement("div");
                div.className = "item-inventory";
                const star = thm.is_solved ? " ⭐" : "";
                div.innerHTML = '<div class="item-inventory-title">' + escapeHtml(thm.name) + star + '</div>' +
                                '<div style="font-family: var(--font-mono); font-size: 0.8rem; color: var(--text-primary); margin-bottom: 2px;">' + escapeHtml(thm.target) + '</div>' +
                                '<div class="item-inventory-doc">' + escapeHtml(thm.doc) + '</div>';
                div.onclick = () => insertSnippet("rw [" + thm.name + "]");
                container.appendChild(div);
            });
        }

        function renderWorldsMap() {
            const container = document.getElementById("worlds-map-list");
            container.innerHTML = "";
            const game = getCurrentGame();
            if (!game) return;

            game.worlds.forEach(w => {
                const item = document.createElement("div");
                item.className = "world-tree-item";

                let solvedInWorld = 0;
                w.levels.forEach(l => {
                    if (localStorage.getItem("lean4game_solved_" + game.id + "_" + w.id + "_" + l.number) === "true") {
                        solvedInWorld++;
                    }
                });

                const header = document.createElement("div");
                header.className = "world-tree-header";
                header.innerHTML = '<span>' + (w.icon || "📁") + ' ' + escapeHtml(w.name) + '</span>' +
                                   '<span style="font-size: 0.8rem; color: var(--yellow);">' + solvedInWorld + ' / ' + w.levels.length + ' ⭐</span>';

                const list = document.createElement("div");
                list.className = "world-levels-list";

                w.levels.forEach(l => {
                    const row = document.createElement("div");
                    const isCurrent = (w.id === currentWorldId && l.number === currentLevelNum);
                    const isSolved = localStorage.getItem("lean4game_solved_" + game.id + "_" + w.id + "_" + l.number) === "true";

                    row.className = "level-row" + (isCurrent ? " active" : "");
                    row.innerHTML = '<span>Level ' + l.number + ': ' + escapeHtml(l.title) + '</span>' +
                                    '<span>' + (isSolved ? "⭐" : "○") + '</span>';
                    row.onclick = () => loadLevel(game.id, w.id, l.number);
                    list.appendChild(row);
                });

                item.appendChild(header);
                item.appendChild(list);
                container.appendChild(item);
            });
        }

        function prevLevel() {
            const game = getCurrentGame();
            const world = getCurrentWorld();
            if (!game || !world) return;

            if (currentLevelNum > 1) {
                loadLevel(currentGameId, currentWorldId, currentLevelNum - 1);
            } else {
                const wIdx = game.worlds.findIndex(w => w.id === currentWorldId);
                if (wIdx > 0) {
                    const prevW = game.worlds[wIdx - 1];
                    loadLevel(currentGameId, prevW.id, prevW.levels.length);
                }
            }
        }

        function nextLevel() {
            const game = getCurrentGame();
            const world = getCurrentWorld();
            if (!game || !world) return;

            if (currentLevelNum < world.levels.length) {
                loadLevel(currentGameId, currentWorldId, currentLevelNum + 1);
            } else {
                const wIdx = game.worlds.findIndex(w => w.id === currentWorldId);
                if (wIdx < game.worlds.length - 1) {
                    const nextW = game.worlds[wIdx + 1];
                    loadLevel(currentGameId, nextW.id, 1);
                }
            }
        }

        function insertSym(sym) {
            insertSnippet(sym);
        }

        function insertSnippet(text) {
            const editor = document.getElementById("code-editor");
            const start = editor.selectionStart;
            const end = editor.selectionEnd;
            const curVal = editor.value;
            editor.value = curVal.substring(0, start) + text + curVal.substring(end);
            editor.selectionStart = editor.selectionEnd = start + text.length;
            editor.focus();
            updateLineNumbers();
        }

        function updateLineNumbers() {
            const editor = document.getElementById("code-editor");
            const lines = editor.value.split("\n").length;
            const gutter = document.getElementById("line-numbers");
            let nums = "";
            for (let i = 1; i <= Math.max(lines, 1); i++) {
                nums += i + "\n";
            }
            gutter.innerText = nums;
        }

        function formatHypothesisLine(hypStr) {
            const colonIdx = hypStr.indexOf(":");
            if (colonIdx === -1) {
                return '<span class="hyp-type">' + escapeHtml(hypStr) + '</span>';
            }
            const name = hypStr.substring(0, colonIdx).trim();
            const type = hypStr.substring(colonIdx + 1).trim();
            return '<span class="hyp-name">' + escapeHtml(name) + '</span> : <span class="hyp-type">' + escapeHtml(type) + '</span>';
        }

        async function verifyProof() {
            const proofScript = document.getElementById("code-editor").value;
            const saveKey = "lean4game_proof_" + currentGameId + "_" + currentWorldId + "_" + currentLevelNum;
            localStorage.setItem(saveKey, proofScript);

            try {
                const res = await fetch("/api/verify", {
                    method: "POST",
                    headers: { "Content-Type": "application/json" },
                    body: JSON.stringify({
                        game_id: currentGameId,
                        world_id: currentWorldId,
                        level_num: currentLevelNum,
                        proof_script: proofScript
                    })
                });
                const data = await res.json();

                const isSolved = data.completed;
                if (isSolved) {
                    localStorage.setItem("lean4game_solved_" + currentGameId + "_" + currentWorldId + "_" + currentLevelNum, "true");
                }
                updateStatusBadge(isSolved);
                updateProgressPill();
                renderWorldsMap();

                const infoview = document.getElementById("infoview-body");
                if (data.completed) {
                    infoview.innerHTML = `
                        <div class="celebration-card">
                            <div class="celebration-icon">🎉</div>
                            <div class="celebration-title">Goals accomplished!</div>
                            <div class="celebration-desc">All goals have been closed. Level completed!</div>
                            <button class="celebration-btn" onclick="nextLevel()">Next Level ➔</button>
                        </div>
                    `;
                } else if (data.goal_infos && data.goal_infos.length > 0) {
                    const goalsHtml = data.goal_infos.map((g, idx) => `
                        <div class="goal-box">
                            ${g.tag ? `<div class="goal-case-tag">case ${escapeHtml(g.tag)}</div>` : `<div class="goal-case-tag">Goal ${idx + 1}</div>`}
                            ${g.hypotheses.length > 0 ? `
                                <div class="goal-hypotheses">
                                    ${g.hypotheses.map(h => `<div class="hyp-line">${formatHypothesisLine(h)}</div>`).join("")}
                                </div>
                            ` : ''}
                            <div class="goal-target">
                                <span class="turnstile">⊢</span>
                                <span>${escapeHtml(g.target)}</span>
                            </div>
                        </div>
                    `).join("");

                    infoview.innerHTML = `
                        <div style="font-weight: 700; color: var(--blue); font-size: 0.88rem; margin-bottom: 4px;">
                            ${data.goal_infos.length} goal${data.goal_infos.length > 1 ? 's' : ''}
                        </div>
                        ${goalsHtml}
                    `;
                } else if (!data.completed && data.message && data.message.includes("error")) {
                    infoview.innerHTML = `
                        <div class="tactic-error-card">
                            <div class="error-title">❌ Tactic Error</div>
                            <div>${escapeHtml(data.message)}</div>
                        </div>
                    `;
                } else {
                    infoview.innerHTML = '<div style="color: var(--text-muted); font-style: italic;">No active goals.</div>';
                }

                document.getElementById("infoview-messages").innerText = data.message;
            } catch (e) {
                document.getElementById("infoview-messages").innerText = "Verification error: " + e;
            }
        }

        // Editor Event Handlers
        const editor = document.getElementById("code-editor");
        editor.addEventListener("input", updateLineNumbers);
        editor.addEventListener("scroll", function() {
            document.getElementById("line-numbers").scrollTop = this.scrollTop;
        });

        editor.addEventListener("keydown", function(e) {
            if (e.key === "Tab") {
                e.preventDefault();
                insertSnippet("  ");
            } else if ((e.ctrlKey || e.metaKey) && e.key === "Enter") {
                e.preventDefault();
                verifyProof();
            }
        });

        window.onload = init;
    </script>
</body>
</html>"####
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
                let games = CurriculumRegistry::get_games();
                let json = serde_json::to_string(&games).unwrap_or_else(|_| "[]".to_string());
                let response = format!(
                    "HTTP/1.1 200 OK\r\nContent-Type: application/json\r\nContent-Length: {}\r\nConnection: close\r\n\r\n{}",
                    json.len(),
                    json
                );
                let _ = stream.write_all(response.as_bytes());
            }
            ("GET", "/api/worlds") => {
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
                        let games = CurriculumRegistry::get_games();
                        let mut target_game_id = req.game_id.clone();
                        let mut target_level = None;
                        let mut target_world_id = req.world_id.clone();

                        // Search across games to find matching game and level
                        for g in &games {
                            if let Some(ref gid) = req.game_id {
                                if &g.id != gid {
                                    continue;
                                }
                            }
                            for w in &g.worlds {
                                if w.id == req.world_id {
                                    if let Some(l) = w.levels.iter().find(|l| l.number == req.level_num) {
                                        target_level = Some(l.clone());
                                        if target_game_id.is_none() {
                                            target_game_id = Some(g.id.clone());
                                        }
                                        target_world_id = w.id.clone();
                                        break;
                                    }
                                }
                            }
                            if target_level.is_some() {
                                break;
                            }
                        }

                        match target_level {
                            Some(level) => {
                                let game_id_str = target_game_id.as_deref().unwrap_or("natural_numbers");
                                let mut env = Environment::standard_for_game(game_id_str);

                                // Progressively unlock theorems in this game up to the requested level
                                if let Some(game) = games.iter().find(|g| g.id == game_id_str) {
                                    for world in &game.worlds {
                                        if world.id == target_world_id {
                                            for l in &world.levels {
                                                if l.number < req.level_num {
                                                    env.add_theorem(l.to_theorem());
                                                }
                                            }
                                            break;
                                        } else {
                                            for l in &world.levels {
                                                env.add_theorem(l.to_theorem());
                                            }
                                        }
                                    }
                                }

                                let initial_state = level.build_initial_state();
                                let res = Verifier::verify_script(&initial_state, &req.proof_script, &env);

                                let open_goals: Vec<String> = res
                                    .final_state
                                    .goals
                                    .iter()
                                    .map(|g| g.target.to_string())
                                    .collect();

                                let goal_infos: Vec<GoalInfo> = res
                                    .final_state
                                    .goals
                                    .iter()
                                    .map(|g| {
                                        let mut hyps = Vec::new();
                                        let mut vars_by_type: std::collections::BTreeMap<String, Vec<String>> =
                                            std::collections::BTreeMap::new();
                                        let mut props = Vec::new();

                                        for h in &g.hypotheses {
                                            if h.is_var {
                                                vars_by_type
                                                    .entry(format!("{}", h.typ))
                                                    .or_default()
                                                    .push(h.name.clone());
                                            } else {
                                                props.push(h);
                                            }
                                        }

                                        for (typ, names) in vars_by_type {
                                            hyps.push(format!("{} : {}", names.join(" "), typ));
                                        }
                                        for h in props {
                                            hyps.push(format!("{} : {}", h.name, h.typ));
                                        }

                                        GoalInfo {
                                            id: g.id,
                                            tag: g.tag.clone(),
                                            hypotheses: hyps,
                                            target: g.target.to_string(),
                                        }
                                    })
                                    .collect();

                                VerificationResponse {
                                    success: res.is_complete,
                                    completed: res.is_complete,
                                    message: if res.is_complete {
                                        "🎉 Goals accomplished! Level successfully solved.".to_string()
                                    } else if let Some(err) = res.first_error {
                                        format!("Tactic error: {}", err)
                                    } else {
                                        format!("Proof in progress: {} goal(s) remaining.", open_goals.len())
                                    },
                                    goals: open_goals,
                                    goal_infos,
                                }
                            }
                            None => VerificationResponse {
                                success: false,
                                completed: false,
                                message: format!("Level not found: world '{}' level {}", req.world_id, req.level_num),
                                goals: vec![],
                                goal_infos: vec![],
                            },
                        }
                    }
                    Err(e) => VerificationResponse {
                        success: false,
                        completed: false,
                        message: format!("Malformed request body: {}", e),
                        goals: vec![],
                        goal_infos: vec![],
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
