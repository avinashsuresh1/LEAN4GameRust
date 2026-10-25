use lean4game::cli::TerminalRunner;
use lean4game::desktop::{DesktopApp, DesktopPersistenceBridge};
use lean4game::storage::save_manager::SaveManager;
use lean4game::ui::Lean4GameApp;
use std::env;
use std::path::PathBuf;

fn main() {
    // Cross-platform guarantee for Linux:
    // WebKitGTK requires environment flags (such as disabling DMA-BUF conflict and sandbox permission issues)
    // to be set in the process environment BEFORE GTK/WebKit shared libraries load.
    // If not already configured, we transparently re-exec the process with these environment variables
    // so a single `cargo run` works out of the box on all Linux distros.
    #[cfg(target_os = "linux")]
    {
        if std::env::var_os("LEAN4GAME_LINUX_ENV_CONFIGURED").is_none() {
            if let Ok(exe) = std::env::current_exe() {
                let mut cmd = std::process::Command::new(exe);
                let raw_args: Vec<String> = std::env::args().skip(1).collect();
                cmd.args(&raw_args);
                for (k, v) in DesktopApp::linux_env_vars(&raw_args) {
                    cmd.env(k, v);
                }

                #[cfg(unix)]
                {
                    use std::os::unix::process::CommandExt;
                    let _ = cmd.exec();
                }
            }
        }
    }

    let args: Vec<String> = env::args().collect();

    if args.contains(&"--help".to_string()) || args.contains(&"-h".to_string()) {
        println!("=======================================================");
        println!("  LEAN 4 Game - Desktop Application with Local Saves");
        println!("=======================================================");
        println!("\nUsage:");
        println!("  cargo run                               Launch Official Lean 4 Game Desktop App (with local saves)");
        println!("  cargo run -- --local                    Launch Self-Contained Local Lean Server & Desktop App (100% offline)");
        println!("  cargo run -- --game <id_or_url>         Launch specific game (e.g. 'nng', 'real-analysis', 'set-theory')");
        println!("  cargo run -- --standalone              Launch Pure Rust Offline Prover GUI");
        println!("  cargo run -- --serve                    Run the self-contained local Lean server in terminal");
        println!("  cargo run -- --test-all                 Run automated proof verification on all 46 levels");
        println!("  cargo run -- --list-games               List available mathematical games and worlds");
        println!("  cargo run -- --list-profiles            List player profiles and completion stats");
        println!("  cargo run -- --cli <world> <level_num>  Play a level interactively in terminal");
        println!("  cargo run -- --export <profile> <path>  Export profile JSON to file");
        println!("  cargo run -- --import <path>            Import profile JSON from file");
        println!("  cargo run -- --export-web <path>        Export official game save to file");
        println!("  cargo run -- --import-web <path>        Import official game save from file");
        println!("=======================================================");
        return;
    }

    if args.contains(&"--serve".to_string()) || args.contains(&"--local-server".to_string()) {
        println!("=======================================================");
        println!("  LEAN 4 Game - Self-Contained Local Server");
        println!("=======================================================");
        match lean4game::desktop::LocalLeanServer::start_in_background(lean4game::desktop::LocalLeanServer::DEFAULT_PORT) {
            Ok(port) => {
                println!("🚀 Server listening on http://127.0.0.1:{}/", port);
                println!("🌐 Web UI available at: http://127.0.0.1:{}/", port);
                println!("🔌 API endpoints: /api/games, /api/verify, /api/health");
                println!("Press Ctrl+C to terminate.");
                loop {
                    std::thread::sleep(std::time::Duration::from_secs(60));
                }
            }
            Err(e) => {
                eprintln!("Failed to start local server: {}", e);
                std::process::exit(1);
            }
        }
    }

    if args.contains(&"--list-games".to_string()) || args.contains(&"-l".to_string()) {
        TerminalRunner::list_games();
        return;
    }

    if args.contains(&"--test-all".to_string()) {
        if let Err(err) = TerminalRunner::test_all_curriculum() {
            eprintln!("Error during verification: {}", err);
            std::process::exit(1);
        }
        return;
    }

    if args.contains(&"--list-profiles".to_string()) {
        if let Err(err) = TerminalRunner::list_profiles(None) {
            eprintln!("Error listing profiles: {}", err);
            std::process::exit(1);
        }
        return;
    }

    if let Some(pos) = args.iter().position(|a| a == "--cli") {
        let world_id = args.get(pos + 1).cloned().unwrap_or_else(|| "tutorial".to_string());
        let level_num = args
            .get(pos + 2)
            .and_then(|s| s.parse::<usize>().ok())
            .unwrap_or(1);

        if let Err(err) = TerminalRunner::play_interactive(&world_id, level_num) {
            eprintln!("Error: {}", err);
            std::process::exit(1);
        }
        return;
    }

    if let Some(pos) = args.iter().position(|a| a == "--export") {
        if let (Some(profile_name), Some(dest)) = (args.get(pos + 1), args.get(pos + 2)) {
            let sm = SaveManager::new(SaveManager::default_saves_dir());
            if let Err(e) = sm.export_profile_to_file(profile_name, &PathBuf::from(dest)) {
                eprintln!("Export error: {}", e);
                std::process::exit(1);
            }
            println!("Successfully exported '{}' to {}", profile_name, dest);
            return;
        } else {
            eprintln!("Usage: lean4game --export <profile_name> <destination_path>");
            std::process::exit(1);
        }
    }

    if let Some(pos) = args.iter().position(|a| a == "--import") {
        if let Some(src) = args.get(pos + 1) {
            let sm = SaveManager::new(SaveManager::default_saves_dir());
            match sm.import_profile_from_file(&PathBuf::from(src)) {
                Ok(p) => {
                    println!("Successfully imported profile '{}' from {}", p.name, src);
                    return;
                }
                Err(e) => {
                    eprintln!("Import error: {}", e);
                    std::process::exit(1);
                }
            }
        } else {
            eprintln!("Usage: lean4game --import <source_json_path>");
            std::process::exit(1);
        }
    }

    if let Some(pos) = args.iter().position(|a| a == "--export-web") {
        if let Some(dest) = args.get(pos + 1) {
            let default_save = DesktopPersistenceBridge::default_save_path();
            if let Err(e) = DesktopPersistenceBridge::export_state(&default_save, &PathBuf::from(dest)) {
                eprintln!("Web save export error: {}", e);
                std::process::exit(1);
            }
            println!("Successfully exported official game save to {}", dest);
            return;
        } else {
            eprintln!("Usage: lean4game --export-web <destination_json_path>");
            std::process::exit(1);
        }
    }

    if let Some(pos) = args.iter().position(|a| a == "--import-web") {
        if let Some(src) = args.get(pos + 1) {
            let default_save = DesktopPersistenceBridge::default_save_path();
            match DesktopPersistenceBridge::import_state(&PathBuf::from(src), &default_save) {
                Ok(count) => {
                    println!("Successfully imported {} items into official game save from {}", count, src);
                    return;
                }
                Err(e) => {
                    eprintln!("Web save import error: {}", e);
                    std::process::exit(1);
                }
            }
        } else {
            eprintln!("Usage: lean4game --import-web <source_json_path>");
            std::process::exit(1);
        }
    }

    // Pure Rust Offline Mode (via --standalone or --offline)
    if args.contains(&"--standalone".to_string()) || args.contains(&"--offline".to_string()) {
        println!("🚀 Launching Pure Rust Offline Prover GUI...");
        let native_options = eframe::NativeOptions {
            viewport: eframe::egui::ViewportBuilder::default()
                .with_inner_size([1200.0, 780.0])
                .with_min_inner_size([900.0, 600.0])
                .with_title("LEAN 4 Game - Pure Rust Offline Edition"),
            ..Default::default()
        };

        if let Err(e) = eframe::run_native(
            "LEAN 4 Game",
            native_options,
            Box::new(|cc| Ok(Box::new(Lean4GameApp::new(cc)))),
        ) {
            eprintln!("Failed to launch standalone GUI: {}", e);
            std::process::exit(1);
        }
        return;
    }

    // Default: Official Lean 4 Game Native Desktop Window with Local Filesystem Persistence
    let devtools = args.contains(&"--devtools".to_string());
    let target_game = if args.contains(&"--local".to_string()) || args.contains(&"--self-contained".to_string()) {
        Some("local")
    } else if let Some(pos) = args.iter().position(|a| a == "--game" || a == "-g") {
        args.get(pos + 1).map(|s| s.as_str())
    } else {
        None
    };

    if let Err(err) = DesktopApp::launch(target_game, None, devtools) {
        eprintln!("Desktop launch error: {}", err);
        println!("Falling back to Pure Rust Standalone Engine...");
        let native_options = eframe::NativeOptions {
            viewport: eframe::egui::ViewportBuilder::default()
                .with_inner_size([1200.0, 780.0])
                .with_min_inner_size([900.0, 600.0])
                .with_title("LEAN 4 Game - Desktop Edition"),
            ..Default::default()
        };
        let _ = eframe::run_native(
            "LEAN 4 Game",
            native_options,
            Box::new(|cc| Ok(Box::new(Lean4GameApp::new(cc)))),
        );
    }
}
