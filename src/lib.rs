pub mod cli;
pub mod desktop;
pub mod engine;
pub mod game;
pub mod storage;
pub mod ui;

pub use cli::TerminalRunner;
pub use desktop::{DesktopApp, DesktopPersistenceBridge};
pub use engine::*;
pub use game::*;
pub use storage::*;
pub use ui::Lean4GameApp;
