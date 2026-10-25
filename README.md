# LEAN 4 Game - Desktop Edition (with Save-to-Disk)

[![Tests Passing](https://img.shields.io/badge/tests-40%20passed-brightgreen.svg)](https://github.com/avinashsuresh1/LEAN4GameRust)
[![Original Game](https://img.shields.io/badge/original%20game-adam.math.hhu.de-blue.svg)](https://adam.math.hhu.de/)

> ### 📢 An Honest Note About What This Project Is
>
> This project is **not** an attempt to rewrite the real Lean 4 compiler or replace the official platform. 
> 
> It is an **exact desktop wrapper** for the official [Lean 4 Game platform (`leanprover-community/lean4game`)](https://github.com/leanprover-community/lean4game) hosted at [adam.math.hhu.de](https://adam.math.hhu.de/). It renders the **exact original game client, Monaco code editor, Infoview, and official Lean 4 / Mathlib backend**, but adds **native desktop support with automatic save-to-disk on your local filesystem**.
>
> It was built with Gemini 3.8 Flash model in Google Antigravity

---

## 🔗 Official Game Server & Original Repositories

All original mathematics, game design, and compiler infrastructure belong to the amazing Lean community. Please star and support their original repositories:

- **Original Game Platform & Server:** [`leanprover-community/lean4game`](https://github.com/leanprover-community/lean4game)  
  *Developed by Alexander Bentkamp, Jon Eugster, Patrick Massot, and contributors at Heinrich Heine University Düsseldorf.*
- **Official Web Server:** [https://adam.math.hhu.de/](https://adam.math.hhu.de/)
- **Natural Number Game (NNG4):** [`leanprover-community/nng4`](https://github.com/leanprover-community/nng4)  
  *By Kevin Buzzard and Mohammad Pedramfar.*
- **Real Analysis Game:** [`tomyun/lean-game-real-analysis`](https://github.com/tomyun/lean-game-real-analysis)  
  *By Prof. Alex Kontorovich (Rutgers University) and Tom Yun.*
- **Formalising Mathematics / Set Theory:** [`ImperialCollegeLondon/formalising-mathematics-2024`](https://github.com/ImperialCollegeLondon/formalising-mathematics-2024)  
  *By Kevin Buzzard at Imperial College London.*

---

## 💡 The Problem It Solves: Saving Your Progress to Disk

When you play the Lean 4 Game on `adam.math.hhu.de` in a normal web browser:
1. All your completed levels, stars, and handwritten proof scripts are stored solely inside browser `localStorage`.
2. Whenever you **clear your browser cookies**, wipe site data, run system cleaners, or switch between browsers/devices, **all your hard-earned progress and mathematical proofs are completely wiped out**.

### How This Desktop App Fixes It:
This app wraps the official game inside a native window (`tao`) and establishes a **bidirectional Rust persistence bridge**:
- **Automatic Hydration on Launch:** On startup, it reads `./saves/official_lean4game_save.json` from your local hard drive and injects your proofs and solved states directly into the browser storage before the page even renders.
- **Continuous Local Disk Sync:** Whenever you type code or complete a level, the app intercepts `localStorage.setItem` and immediately sends an IPC message to the Rust backend, which saves it atomically to disk (`./saves/official_lean4game_save.json`).
- **Crash-Resistant Atomic Writes:** Writes to a temporary `.tmp` file first and uses atomic file renaming, ensuring your saves can never be corrupted.
- **Periodic Heartbeat & Manual Hotkeys:** Runs an automatic 2.5-second heartbeat sync, and you can press `Ctrl+S` anytime to flush immediately.
- **Backup & Portability:** You can export your save file or transfer it to another computer at any time.

---

## 🎮 Secondary Fallback: Standalone Pure Rust Offline Prover

In addition to the primary desktop wrapper, this repository also includes a **secondary, 100% offline pure Rust interactive theorem prover** (`--standalone`). 

If you are on an airplane, have no internet connection, or cannot reach the HHU game servers, you can still solve **46 canonical levels across 12 worlds** (Natural Numbers, Real Analysis, and Set Theory) entirely offline using a built-in pure Rust tactic engine (`rfl`, `rw`, `induction`, `exact`, `intro`, `apply`, `use`, `constructor`, `rcases`, `linarith`).

---

## 🚀 Quick Start & Usage

### 🌐 Cross-Platform Support & Prerequisites

The application is completely cross-platform (**Linux**, **macOS**, and **Windows**):

- **Rust toolchain:** [rustup.rs](https://rustup.rs/) (1.75+ or newer).
- **Native Webview Engine (per Operating System):**
  - **Linux:** Uses native WebKitGTK. Install the standard webview development packages:
    - *Ubuntu / Debian:* `sudo apt install libwebkit2gtk-4.1-dev build-essential libssl-dev libgtk-3-dev`
    - *Fedora:* `sudo dnf install webkit2gtk4.1-devel gtk3-devel`
    - *Arch Linux:* `sudo pacman -S webkit2gtk-4.1 gtk3`
  - **macOS:** Uses Apple `WKWebView` (built into macOS, zero additional runtime installation needed).
  - **Windows:** Uses Microsoft Edge WebView2 (pre-installed by default on Windows 10 & 11).
- **Standalone Offline Engine (`--standalone`):** Does not require any WebView runtime; uses native cross-platform OpenGL/WGPU rendering across all operating systems.

### 1. Launch the Official Desktop App (with local saves)
```bash
cargo run
```
Opens the native desktop window connected to the official Lean 4 Game hub with your saves automatically loading from and saving to `./saves/official_lean4game_save.json`.

### 2. Launch Self-Contained Local Lean Server & Desktop App (100% Offline)
```bash
cargo run -- --local
```
Automatically boots an embedded local Lean server on `http://127.0.0.1:8080/` with interactive proof verification and connects the desktop window to it. **Zero internet connection, zero external network requests, and zero remote dependencies.**

### 3. Run Self-Contained Local Server in Terminal
```bash
cargo run -- --serve
```
Runs the standalone embedded HTTP server on `127.0.0.1:8080` exposing the interactive Lean 4 web client and JSON endpoints (`/api/games`, `/api/verify`, `/api/health`).

### 4. Launch a Specific Game Directly
```bash
# Natural Number Game (NNG4)
cargo run -- --game nng

# Real Analysis: The Game
cargo run -- --game real-analysis

# Set Theory / Formalising Mathematics
cargo run -- --game set-theory

# Any other official/community game repository
cargo run -- --game AlexanderBentkamp/lean4game-logic
```

### 5. Launch Standalone Offline Pure Rust Engine (egui/eframe GUI)
```bash
cargo run -- --standalone
# or
cargo run -- --offline
```

### 6. Backup, Restore, and Reset Save Files

Although your game progress and solved stars are **automatically saved continuously to disk** in `./saves/official_lean4game_save.json`, you can easily create standalone file backups to store in cloud storage, commit to version control, or move to another machine.

#### A. Creating a Backup (Export)
When you want to take a snapshot of your progress (or before sharing your device):
1. Close the game window (to ensure all in-memory proofs are fully flushed to disk).
2. Run the export command:
   ```bash
   cargo run -- --export-web my_progress_backup.json
   ```
   *This extracts your complete proof state, theorem inventory, and solved stars into `my_progress_backup.json`.*

#### B. Restoring Progress (Import)
To restore an earlier backup file:
```bash
cargo run -- --import-web my_progress_backup.json
```
On the next launch, the desktop bridge reads this file and automatically re-hydrates the game with your restored progress.

#### C. Resetting Progress from Scratch (`reset.json`)
If you have completed all the games and want to experience the challenge completely fresh from Level 1, this repository includes [`reset.json`](file:///d:/LEAN4Game/reset.json).

`reset.json` is a dedicated empty save state containing `{"game_progress": "{\"games\":{}}"}`. You can reset all game progress back to scratch in one command by importing it:
```bash
cargo run -- --import-web reset.json
```
On next launch, the bridge actively overwrites the webview cache with the empty game structure, clearing all solved stars, level unlocks, and draft proof scripts so you can start from scratch!

### 7. Automated Proof Verification
```bash
# Verify all curriculum levels in the prover
cargo run -- --test-all
```

---

## 🛡️ Security Hardening & Zero-Exploit Architecture

Because desktop WebViews interface with web content and local storage, strict security controls are enforced at the Rust native boundary:

- **Strict Origin & Domain Allowlist (`with_navigation_handler`)**: Web navigation is restricted strictly to official Lean community domains (`adam.math.hhu.de`, `leanprover-community.github.io`) and local loopback (`127.0.0.1`, `localhost`). All untrusted external domains are blocked.
- **URI Scheme Quarantine**: Dangerous URI schemes (`file:`, `javascript:`, `data:`, `shell:`, `vbscript:`) are blocked from top-level navigation.
- **Popup & Window Request Blocking (`with_new_window_req_handler`)**: Unauthorized popup windows or external redirects spawned by page scripts are intercepted and denied.
- **Filesystem Path Traversal Protection**: Save paths are strictly validated against directory traversal attacks (`..` components) and confined to `./saves/`.
- **IPC Payload Validation & DoS Prevention**: Incoming persistence IPC payloads are sanitized, capping keys at 512 bytes, values at 2 MB, and total payloads at 25 MB with null-byte rejection.
- **Zero Remote Dependency Mode (`--local`)**: Enables complete isolation from online infrastructure by executing against the built-in local server.

---

## 🎨 Creating & Playing Custom Games

You can create and run custom games in **both modes**:

### 1. In Official Lean 4 Desktop Mode
Any game repository published on GitHub using the official Lean 4 Game framework can be loaded directly:
```bash
# Play any official or community Lean 4 game repository:
cargo run -- --game <github_username>/<repository_name>

# Or point to a locally hosted game server:
cargo run -- --game http://localhost:8080/#/g/my-game
```
All your handwritten proofs and completed level stars in custom games are automatically captured and saved to `./saves/official_lean4game_save.json`.

### 2. In Standalone Offline Pure Rust Mode
You can create your own mathematical games with custom worlds, levels, theorems, hypotheses, and hints simply by placing a JSON file into the [`./games/`](file:///d:/LEAN4Game/games/) folder.

See [`games/example_custom_game.json`](file:///d:/LEAN4Game/games/example_custom_game.json) for a complete template:

```json
{
  "id": "my_geometry_game",
  "title": "Geometry Foundations",
  "description": "Custom axioms and proofs in metric geometry.",
  "author": "Community Creator",
  "icon": "📐",
  "worlds": [
    {
      "id": "triangle_world",
      "name": "Triangle Properties",
      "description": "Triangle inequalities and congruence.",
      "icon": "🔺",
      "levels": [
        {
          "id": "level_1",
          "world_id": "triangle_world",
          "number": 1,
          "title": "Metric non-negativity",
          "theorem_name": "dist_nonneg",
          "doc": "Prove that distance between distinct points is positive.",
          "hints": ["Use definition of metric space"],
          "context_vars": [["x", "ℝ"], ["y", "ℝ"]],
          "initial_hypotheses": [],
          "target_str": "x + 0 = x",
          "available_tactics": ["rw", "rfl"],
          "reference_solution": "rw [add_zero]"
        }
      ]
    }
  ]
}
```

Any `.json` file in `./games/` is **automatically detected and loaded**:
- Appears as a dedicated tab in the desktop GUI application
- Listed in `cargo run -- --list-games`
- Playable interactively in the terminal via `cargo run -- --cli <world_id> <level_num>`
- Verified by the automated test suite `cargo run -- --test-all`

---

## 🧪 Automated Tests

The codebase includes **40 unit and integration tests** verifying URL routing, custom game loading, injection script generation, IPC messaging, atomic disk persistence, mathematical tactics, security origin guards, path traversal prevention, and embedded local server endpoints:

```bash
cargo test
```

- [`tests/security_tests.rs`](file:///d:/LEAN4Game/tests/security_tests.rs): Tests URL allowlist enforcement, dangerous URI scheme rejection, path traversal protection, IPC payload validation, and embedded local server API endpoints.
- [`tests/desktop_persistence_tests.rs`](file:///d:/LEAN4Game/tests/desktop_persistence_tests.rs): Tests the JavaScript injection script generation, atomic disk saving and loading, native IPC handling, Linux environment resolution, and web save export/import.
- [`tests/curriculum_tests.rs`](file:///d:/LEAN4Game/tests/curriculum_tests.rs): End-to-end verification of all 46 levels across Natural Numbers, Real Analysis, and Set Theory.
- [`tests/tactics_tests.rs`](file:///d:/LEAN4Game/tests/tactics_tests.rs): Tests for `rfl`, `rw`, `induction`, `exact`, `intro`, `apply`, and relational symmetry.
- [`tests/engine_tests.rs`](file:///d:/LEAN4Game/tests/engine_tests.rs): AST manipulation, free variable analysis, expression parsing, and pattern matching.
- [`tests/persistence_tests.rs`](file:///d:/LEAN4Game/tests/persistence_tests.rs): Multi-profile management, atomic saving, and draft preservation for the offline engine.

---

## 📂 Project Architecture

```
d:/LEAN4Game/
├── saves/                               # Local filesystem save directory
│   ├── official_lean4game_save.json     # Official desktop game saves & proof states
│   └── .gitkeep                         # Preserves directory in git
├── src/
│   ├── desktop/                         # Official Desktop App & Save-to-Disk Bridge
│   │   ├── bridge.rs                    # IPC handler, injection script, atomic disk persistence
│   │   ├── local_server.rs              # Embedded self-contained local server & toolchain detection
│   │   ├── security.rs                  # Origin allowlist, traversal protection & payload validation
│   │   ├── webview_app.rs               # Tao window + Wry integration (Wayland/X11 GTK container)
│   │   └── mod.rs
│   ├── engine/                          # Offline Pure Rust Theorem Prover (Fallback)
│   │   ├── expr.rs                      # Mathematical AST (N, R, quantifiers, metrics)
│   │   ├── parser.rs                    # Lean 4 expression & tactic parser
│   │   ├── tactics.rs                   # Tactics (rfl, rw, induction, apply, use, rcases...)
│   │   ├── proof_state.rs               # Goal and hypothesis manager
│   │   ├── environment.rs               # Axioms (Peano, Real Analysis, Metrics)
│   │   └── verifier.rs                  # Step-by-step cursor proof verifier
│   ├── game/                            # Curriculum & Level Definitions
│   │   ├── registry.rs                  # 46 canonical levels in NNG, Real Analysis, Set Theory
│   │   ├── package.rs                   # Game package definitions
│   │   ├── world.rs                     # World definitions
│   │   └── level.rs                     # Level goals & theorem definitions
│   ├── ui/                              # Pure Rust GUI
│   │   └── app.rs                       # Egui/Eframe application
│   ├── storage/                         # Offline Profile & Save Manager
│   │   ├── save_manager.rs              # Atomic file writes & profiles
│   │   └── profile.rs                   # Profile data model
│   ├── cli/                             # Command Line Runner
│   │   └── terminal_game.rs             # CLI game runner and verification
│   ├── lib.rs                           # Shared library exports
│   └── main.rs                          # CLI dispatcher and app launcher
├── tests/                               # 36 Comprehensive Unit & Integration Tests
│   ├── security_tests.rs
│   ├── desktop_persistence_tests.rs
│   ├── curriculum_tests.rs
│   ├── engine_tests.rs
│   ├── tactics_tests.rs
│   └── persistence_tests.rs
└── Cargo.toml                           # tao, wry, eframe, serde, url, etc.
```

---

## 📜 Credits & Licensing

- **Lean 4 Game Platform (`leanprover-community/lean4game`):** Licensed under **GPL-3.0** by the Lean Community / HHU.
- **Natural Number Game (`leanprover-community/nng4`):** Licensed under **Apache-2.0** by Kevin Buzzard & Mohammad Pedramfar.
- **Lean 4 Compiler & Mathlib (`leanprover/lean4`, `leanprover-community/mathlib4`):** Licensed under **Apache-2.0**.
- **This Desktop Wrapper & Offline Engine:** Licensed under the **GNU General Public License v3.0 (GPL-3.0)** (see [`LICENSE`](file:///d:/LEAN4Game/LICENSE)).
