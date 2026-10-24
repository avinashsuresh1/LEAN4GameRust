use crate::desktop::bridge::DesktopPersistenceBridge;
use std::path::PathBuf;
use tao::{
    event::{Event, StartCause, WindowEvent},
    event_loop::{ControlFlow, EventLoop},
    window::WindowBuilder,
};
use wry::WebViewBuilder;

pub struct DesktopApp;

impl DesktopApp {
    /// Resolves game aliases and identifiers to official Lean 4 Game URLs.
    pub fn resolve_game_url(input: &str) -> String {
        let trimmed = input.trim();
        let lower = trimmed.to_lowercase();
        match lower.as_str() {
            "nng" | "nng4" | "natural-numbers" | "natural-number-game" => {
                "https://adam.math.hhu.de/#/g/leanprover-community/nng4".to_string()
            }
            "real-analysis" | "real_analysis" | "ra" => {
                "https://adam.math.hhu.de/#/g/tomyun/lean-game-real-analysis".to_string()
            }
            "set-theory" | "set_theory" | "st" | "formalising-mathematics" => {
                "https://adam.math.hhu.de/#/g/ImperialCollegeLondon/formalising-mathematics-2024".to_string()
            }
            "hub" | "home" | "all" | "" => {
                "https://adam.math.hhu.de/".to_string()
            }
            _ => {
                if trimmed.starts_with("http://") || trimmed.starts_with("https://") {
                    trimmed.to_string()
                } else {
                    format!("https://adam.math.hhu.de/#/g/{}", trimmed)
                }
            }
        }
    }

    /// Launches the exact official Lean 4 Game client in a native desktop window
    /// with bidirectional local filesystem persistence.
    pub fn launch(target_url: Option<&str>, save_path: Option<PathBuf>) -> Result<(), String> {
        let resolved_url = target_url.map(Self::resolve_game_url).unwrap_or_else(|| "https://adam.math.hhu.de/".to_string());
        let save_file = save_path.unwrap_or_else(DesktopPersistenceBridge::default_save_path);
        println!("=======================================================");
        println!("  LEAN 4 Game - Official Desktop Edition");
        println!("=======================================================");
        println!("  🌐 Connecting to official Lean 4 Game engine: {}", resolved_url);
        println!("  💾 Local persistence file: {:?}", save_file);
        println!("=======================================================");

        let event_loop = EventLoop::new();
        let window = WindowBuilder::new()
            .with_title("LEAN 4 Game - Desktop Edition (Official Engine & Local Filesystem Saves)")
            .with_inner_size(tao::dpi::LogicalSize::new(1350.0, 850.0))
            .build(&event_loop)
            .map_err(|e| format!("Failed to create desktop window: {}", e))?;

        let saved_state = DesktopPersistenceBridge::load_saved_state(&save_file);
        println!(
            "  📂 Hydrated {} items from local disk into game state",
            saved_state.len()
        );

        let injection_script = DesktopPersistenceBridge::generate_injection_script(&saved_state);
        let save_file_clone = save_file.clone();

        // Fix WebKitGTK 2.40+ bugs on Linux:
        // 1. DMA-BUF renderer conflicts with graphics drivers/Wayland.
        // 2. Bubblewrap sandbox crashes when unprivileged user namespaces are restricted (e.g. Ubuntu 24.04).
        #[cfg(target_os = "linux")]
        {
            if std::env::var_os("WEBKIT_DISABLE_DMABUF_RENDERER").is_none() {
                unsafe {
                    std::env::set_var("WEBKIT_DISABLE_DMABUF_RENDERER", "1");
                }
            }
            if std::env::var_os("WEBKIT_FORCE_SANDBOX").is_none() {
                unsafe {
                    std::env::set_var("WEBKIT_FORCE_SANDBOX", "0");
                }
            }
        }

        let webview = WebViewBuilder::new()
            .with_url(&resolved_url)
            .with_devtools(true)
            .with_on_page_load_handler(|event, url| {
                let status = match event {
                    wry::PageLoadEvent::Started => "Started",
                    wry::PageLoadEvent::Finished => "Finished",
                };
                println!("🌐 [Page Load {}] {}", status, url);
            })
            .with_initialization_script(&injection_script)
            .with_ipc_handler(move |req: wry::http::Request<String>| {
                let msg = req.body();
                match DesktopPersistenceBridge::handle_ipc_message(msg, &save_file_clone) {
                    Ok(count) => {
                        println!(
                            "💾 [Persistence] Auto-saved {} items to {:?}",
                            count, save_file_clone
                        );
                    }
                    Err(e) => {
                        eprintln!("❌ [Persistence Error]: {}", e);
                    }
                }
            })
            .build(&window)
            .map_err(|e| format!("Failed to initialize webview: {}", e))?;

        event_loop.run(move |event, _, control_flow| {
            *control_flow = ControlFlow::Wait;
            // Retain webview in closure so it is NOT dropped on GTK/Linux
            let _ = &webview;

            match event {
                Event::NewEvents(StartCause::Init) => {
                    println!("🚀 Desktop application initialized successfully.");
                }
                Event::WindowEvent {
                    event: WindowEvent::CloseRequested,
                    ..
                } => {
                    println!("👋 Closing LEAN 4 Game. Progress is safely saved to disk.");
                    *control_flow = ControlFlow::Exit;
                }
                _ => (),
            }
        });
    }
}
