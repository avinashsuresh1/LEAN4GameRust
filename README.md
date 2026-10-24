# LEAN 4 Game - Desktop Edition (with Save-to-Disk)

[![CI Tests](https://github.com/avinashsuresh1/LEAN4GameRust/actions/workflows/ci.yml/badge.svg?branch=main)](https://github.com/avinashsuresh1/LEAN4GameRust)
[![Tests Passing](https://img.shields.io/badge/tests-30%20passed-brightgreen.svg)](https://github.com/avinashsuresh1/LEAN4GameRust)
[![Original Game](https://img.shields.io/badge/original%20game-adam.math.hhu.de-blue.svg)](https://adam.math.hhu.de/)

> ### 📢 An Honest Note About What This Project Is
>
> This project is **not** an attempt to rewrite the real Lean 4 compiler or replace the official platform. 
> 
> It is an **exact desktop wrapper** for the official [Lean 4 Game platform (`leanprover-community/lean4game`)](https://github.com/leanprover-community/lean4game) hosted at [adam.math.hhu.de](https://adam.math.hhu.de/). It renders the **exact original game client, Monaco code editor, Infoview, and official Lean 4 / Mathlib backend**, but adds **native desktop support with automatic save-to-disk on your local filesystem**.

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
This app wraps the official game inside a native window (`tao`) using Edge WebView2 (`wry`) and establishes a **bidirectional Rust persistence bridge**:
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

### 2. Launch a Specific Game Directly
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

### 3. Launch Standalone Offline Pure Rust Engine
```bash
cargo run -- --standalone
# or
cargo run -- --offline
```

### 4. Backup & Restore Save Files
```bash
# Export your official game save to a backup JSON file
cargo run -- --export-web saves/my_progress_backup.json

# Import a save file from a backup
cargo run -- --import-web saves/my_progress_backup.json
```

### 5. Automated Proof Verification
```bash
# Verify all 46 curriculum levels in the pure Rust prover
cargo run -- --test-all
```

---

## 🧪 Automated Tests

The codebase includes **30 unit and integration tests** verifying URL routing, injection script generation, IPC messaging, atomic disk persistence, and mathematical tactics:

```bash
cargo test
```

- [`tests/desktop_persistence_tests.rs`](file:///d:/LEAN4Game/tests/desktop_persistence_tests.rs): Tests the JavaScript injection script generation, atomic disk saving and loading, native IPC handling, game URL resolution, and web save export/import.
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
│   │   ├── webview_app.rs               # Tao window + Wry WebView2 integration
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
├── tests/                               # 30 Comprehensive Unit & Integration Tests
│   ├── desktop_persistence_tests.rs
│   ├── curriculum_tests.rs
│   ├── engine_tests.rs
│   ├── tactics_tests.rs
│   └── persistence_tests.rs
└── Cargo.toml                           # tao, wry, eframe, serde, etc.
```

---

## 📜 Credits & Licensing

- **Lean 4 Game Platform (`leanprover-community/lean4game`):** Licensed under **GPL-3.0** by the Lean Community / HHU.
- **Natural Number Game (`leanprover-community/nng4`):** Licensed under **Apache-2.0** by Kevin Buzzard & Mohammad Pedramfar.
- **Lean 4 Compiler & Mathlib (`leanprover/lean4`, `leanprover-community/mathlib4`):** Licensed under **Apache-2.0**.
- **This Desktop Wrapper & Offline Engine:** Licensed under the **MIT License** (see [`LICENSE`](file:///d:/LEAN4Game/LICENSE)).
