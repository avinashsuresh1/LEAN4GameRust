# LEAN 4 Game - Desktop Edition (with Local Filesystem Persistence)

A native desktop application and persistence bridge for the **Lean 4 Game platform**, allowing you to play the exact official Lean 4 games (**Natural Number Game**, **Real Analysis Game**, **Set Theory**, and more) on your desktop with all progress, level states, code drafts, and proofs automatically and permanently saved to your local disk.

It also includes a **100% offline pure Rust interactive theorem prover** as a secondary standalone engine.

---

## 🎯 Why This Exists

The official Lean 4 Game is hosted online at [`adam.math.hhu.de`](https://adam.math.hhu.de/). While excellent for learning mathematics and formal theorem proving, the browser version stores all user progress in browser `localStorage`. Whenever cookies are cleared, site data resets, or the browser caches are cleaned, **all game progress and handwritten proofs are permanently lost**.

This application solves that problem by packaging the game as a native desktop application with a **Rust bidirectional filesystem bridge** that persists every proof draft and level completion to `./saves/` on your hard drive.

---

## 🌟 Key Features

### 1. 🖥️ Official Lean 4 Game Native Desktop App (Primary Mode)
- **Exact Official Experience**: Powered by native desktop windowing (`tao`) and Edge WebView2 (`wry`). You get the full official Monaco editor, Mathlib compiler connection, interactive infoview, and all published games.
- **Automatic Filesystem Saves**: A custom Rust initialization script hooks into browser storage APIs before page execution:
  - **Cold Hydration**: Reads `./saves/official_lean4game_save.json` on startup and populates browser storage seamlessly.
  - **Atomic Disk Synchronization**: Intercepts `setItem`, `removeItem`, and `clear`, flushing updates via native IPC to Rust.
  - **Crash Prevention**: Uses atomic `.tmp` write and rename semantics so saves are never corrupted.
  - **Heartbeat & Hotkeys**: Periodic 2.5-second heartbeat sync plus `Ctrl+S` manual save trigger.
- **One-Click Game Launchers**:
  - `cargo run`: Launches the main Lean 4 Game hub with all games available.
  - `cargo run -- --game nng`: Opens the Natural Number Game directly.
  - `cargo run -- --game real-analysis`: Opens Real Analysis directly.
  - `cargo run -- --game set-theory`: Opens Set Theory directly.
  - `cargo run -- --game <url_or_repo>`: Opens any official community game.
- **Export & Import**: Backup your save file or transfer it between computers using `--export-web` and `--import-web`.

---

### 2. ⚡ Standalone Pure Rust Offline Engine (Fallback Mode)
- **100% Offline**: Requires zero network connection, zero Lean 4 binaries, and zero Mathlib installation.
- **46 Built-In Levels Across 12 Worlds**:
  - **Natural Number Game (30 levels)**: Tutorial, Addition, Multiplication, Power, Implication, and Inequality Worlds.
  - **Real Analysis (11 levels)**: Real Foundations & Metrics, Sequences & $\varepsilon$-$N$ Limits, Continuity & $\varepsilon$-$\delta$ Limits, Intermediate Value & Order.
  - **Set Theory & Logic (5 levels)**: Subsets & Inclusions, Intersections & Unions.
- **Native GUI (`eframe` / `egui`)**:
  - Infoview displaying active goals (`⊢ ...`) and hypotheses context (`h : ...`).
  - Interactive Lean 4 code editor with real-time verification and cursor diagnostics.
  - Tactic palette buttons (`rfl`, `rw`, `induction`, `exact`, `intro`, `apply`, `use`, `constructor`, `rcases`).
  - Peano and Real Analysis theorem inventory for 1-click proof generation.
- **CLI & Automated Verification**:
  - `--test-all`: Automated test runner verifying all 46 levels end-to-end.
  - `--cli <world> <level>`: Play levels directly in your command line.
  - `--list-games`: Print all games, worlds, and level descriptions.

---

## 🚀 Quick Start & Usage

### 1. Launch Official Desktop App (Recommended)
```bash
cargo run
```
Launches the desktop window loading the official Lean 4 Game with automatic local saves to `./saves/official_lean4game_save.json`.

### 2. Launch Specific Games Directly
```bash
# Natural Number Game (NNG4)
cargo run -- --game nng

# Real Analysis Game
cargo run -- --game real-analysis

# Set Theory Game
cargo run -- --game set-theory

# Any custom community game repository
cargo run -- --game AlexanderBentkamp/lean4game-logic
```

### 3. Launch Standalone Offline Pure Rust Engine
```bash
cargo run -- --standalone
# or
cargo run -- --offline
```

### 4. Verify All Proofs in Curriculum (Automated Test)
```bash
cargo run -- --test-all
```

### 5. Export and Import Save Files
```bash
# Export official desktop save
cargo run -- --export-web saves/backup_official.json

# Import official desktop save
cargo run -- --import-web saves/backup_official.json

# Export pure Rust standalone profile
cargo run -- --export Default saves/backup_profile.json

# Import pure Rust standalone profile
cargo run -- --import saves/backup_profile.json
```

---

## 🧪 Comprehensive Verification Suite

The project includes 30 deterministic unit and integration tests verifying both the persistence bridge and the mathematics engine:

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
│   └── Default.json                     # Offline prover player profile
├── src/
│   ├── desktop/                         # Official Desktop App & Bridge
│   │   ├── bridge.rs                    # IPC handler, injection script, atomic persistence
│   │   ├── webview_app.rs               # Tao window + Wry WebView2 integration
│   │   └── mod.rs
│   ├── engine/                          # Offline Pure Rust Theorem Prover
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
└── Cargo.toml                           # Dependencies: tao, wry, eframe, serde, etc.
```
