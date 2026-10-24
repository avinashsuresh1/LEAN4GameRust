use crate::engine::environment::Environment;
use crate::engine::verifier::{VerificationResult, Verifier};
use crate::game::level::Level;
use crate::game::package::GamePackage;
use crate::game::registry::CurriculumRegistry;
use crate::game::world::World;
use crate::storage::profile::UserProfile;
use crate::storage::save_manager::SaveManager;
use eframe::egui::{self, Color32, RichText, Vec2};
use std::path::PathBuf;

pub struct Lean4GameApp {
    games: Vec<GamePackage>,
    current_game_idx: usize,
    current_world_idx: usize,
    current_level_idx: usize,

    // Persistence
    save_manager: SaveManager,
    profile: UserProfile,
    available_profiles: Vec<String>,
    auto_save_timer: f32,

    // Editor & Verifier State
    editor_script: String,
    verification: Option<VerificationResult>,
    env: Environment,
    status_banner: Option<(String, bool)>, // (message, is_success)

    // UI state
    show_profile_dialog: bool,
    show_export_import_dialog: bool,
    new_profile_input: String,
    export_path_input: String,
    import_path_input: String,
}

impl Lean4GameApp {
    pub fn new(_cc: &eframe::CreationContext<'_>) -> Self {
        let save_manager = SaveManager::new(SaveManager::default_saves_dir());
        let profiles = save_manager.list_profiles().unwrap_or_default();
        let profile_name = profiles.first().cloned().unwrap_or_else(|| "Default".to_string());

        let profile = match save_manager.load_profile(&profile_name) {
            Ok(p) => p,
            Err(_) => {
                let new_p = UserProfile::new(&profile_name);
                let _ = save_manager.save_profile(&new_p);
                new_p
            }
        };

        let games = CurriculumRegistry::get_games();
        let current_game_idx = games
            .iter()
            .position(|g| g.id == profile.active_game_id)
            .unwrap_or(0);

        let mut current_world_idx = 0;
        let mut current_level_idx = 0;
        if let Some(w_pos) = games[current_game_idx]
            .worlds
            .iter()
            .position(|w| w.id == profile.active_world_id)
        {
            current_world_idx = w_pos;
            if let Some(l_pos) = games[current_game_idx].worlds[w_pos]
                .levels
                .iter()
                .position(|l| l.id == profile.active_level_id)
            {
                current_level_idx = l_pos;
            }
        }

        let mut app = Self {
            games,
            current_game_idx,
            current_world_idx,
            current_level_idx,
            save_manager,
            profile,
            available_profiles: profiles,
            auto_save_timer: 0.0,
            editor_script: String::new(),
            verification: None,
            env: Environment::standard_for_game("natural_numbers"),
            status_banner: None,
            show_profile_dialog: false,
            show_export_import_dialog: false,
            new_profile_input: String::new(),
            export_path_input: String::new(),
            import_path_input: String::new(),
        };

        // Sync unlocked theorems from profile
        app.sync_environment_from_profile();
        app.load_active_level();

        app
    }

    pub fn current_game(&self) -> &GamePackage {
        &self.games[self.current_game_idx]
    }

    pub fn current_world(&self) -> &World {
        &self.current_game().worlds[self.current_world_idx]
    }

    pub fn current_level(&self) -> &Level {
        &self.current_world().levels[self.current_level_idx]
    }

    fn sync_environment_from_profile(&mut self) {
        let game_id = self.current_game().id.clone();
        self.env = Environment::standard_for_game(&game_id);
        for game in &self.games {
            for world in &game.worlds {
                for level in &world.levels {
                    if self.profile.is_level_completed(&world.id, &level.id) {
                        self.env.add_theorem(level.to_theorem());
                    }
                }
            }
        }
    }

    fn load_active_level(&mut self) {
        let world_id = self.current_world().id.clone();
        let level_id = self.current_level().id.clone();

        // Restore draft or saved proof script
        if let Some(script) = self.profile.get_script(&world_id, &level_id) {
            self.editor_script = script;
        } else {
            self.editor_script = String::new();
        }

        self.run_verification();
    }

    fn run_verification(&mut self) {
        let level = self.current_level().clone();
        let initial_state = level.build_initial_state();
        let result = Verifier::verify_script(&initial_state, &self.editor_script, &self.env);

        let was_completed = self.profile.is_level_completed(&level.world_id, &level.id);

        if result.is_complete && !was_completed {
            self.profile.mark_completed(
                &level.world_id,
                &level.id,
                &self.editor_script,
                result.step_count,
                Some(&level.theorem_name),
            );
            self.env.add_theorem(level.to_theorem());
            let _ = self.save_manager.save_profile(&self.profile);
            self.status_banner = Some((
                format!(
                    "Level Complete! Theorem '{}' unlocked and saved to filesystem.",
                    level.theorem_name
                ),
                true,
            ));
        }

        self.verification = Some(result);
    }

    fn save_current_state(&mut self) {
        let game_id = self.current_game().id.clone();
        let world_id = self.current_world().id.clone();
        let level_id = self.current_level().id.clone();
        self.profile.save_draft(&world_id, &level_id, &self.editor_script);
        self.profile.active_game_id = game_id;
        self.profile.active_world_id = world_id;
        self.profile.active_level_id = level_id;
        let _ = self.save_manager.save_profile(&self.profile);
    }
}

impl eframe::App for Lean4GameApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        // Auto-save drafts every 2 seconds if changed
        self.auto_save_timer += ctx.input(|i| i.stable_dt);
        if self.auto_save_timer > 2.0 {
            self.save_current_state();
            self.auto_save_timer = 0.0;
        }

        // Top Navigation & Stats Bar
        let mut switch_game: Option<usize> = None;
        egui::TopBottomPanel::top("top_header").show(ctx, |ui| {
            ui.add_space(4.0);
            ui.horizontal(|ui| {
                ui.heading(RichText::new("📐 LEAN 4 Game - Desktop").strong());
                ui.separator();

                // Profile menu
                if ui
                    .button(format!("👤 Profile: {}", self.profile.name))
                    .clicked()
                {
                    self.available_profiles = self.save_manager.list_profiles().unwrap_or_default();
                    self.show_profile_dialog = true;
                }

                // Saves directory indicator
                ui.label(
                    RichText::new(format!(
                        "💾 Saves: {}",
                        self.save_manager.base_dir().display()
                    ))
                    .color(Color32::from_rgb(120, 200, 120))
                    .small(),
                );

                if ui.button("Backup / Export / Import").clicked() {
                    self.show_export_import_dialog = true;
                }

                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                    let total_levels: usize = self.games.iter().map(|g| g.total_levels()).sum();
                    let game_levels = self.current_game().total_levels();
                    let game_completed = self
                        .current_game()
                        .worlds
                        .iter()
                        .flat_map(|w| &w.levels)
                        .filter(|l| self.profile.is_level_completed(&l.world_id, &l.id))
                        .count();
                    let total_completed = self.profile.total_completed();
                    ui.label(
                        RichText::new(format!(
                            "⭐ Game: {}/{} | Total: {}/{} Solved",
                            game_completed, game_levels, total_completed, total_levels
                        ))
                        .strong()
                        .color(Color32::from_rgb(255, 215, 0)),
                    );
                });
            });

            ui.separator();

            // Game Suite Switcher Tabs
            ui.horizontal(|ui| {
                ui.label(RichText::new("Game Suites:").small().color(Color32::LIGHT_GRAY));
                for (g_idx, game) in self.games.iter().enumerate() {
                    let is_active = g_idx == self.current_game_idx;
                    let text = format!("{} {}", game.icon, game.title);
                    let resp = ui.selectable_label(
                        is_active,
                        RichText::new(text).strong().color(if is_active {
                            Color32::from_rgb(120, 220, 255)
                        } else {
                            Color32::WHITE
                        }),
                    );
                    if resp.clicked() && !is_active {
                        switch_game = Some(g_idx);
                    }
                }
            });

            ui.add_space(4.0);
        });

        if let Some(g_idx) = switch_game {
            self.save_current_state();
            self.current_game_idx = g_idx;
            self.current_world_idx = 0;
            self.current_level_idx = 0;
            self.profile.active_game_id = self.current_game().id.clone();
            self.sync_environment_from_profile();
            self.load_active_level();
        }

        // Modals
        self.render_profile_modal(ctx);
        self.render_export_import_modal(ctx);

        // Left Panel: World Map & Level Selector
        let mut switch_to: Option<(usize, usize)> = None;
        let mut switch_world: Option<usize> = None;

        egui::SidePanel::left("world_selector")
            .resizable(true)
            .default_width(260.0)
            .show(ctx, |ui| {
                ui.heading(format!("{} {}", self.current_game().icon, self.current_game().title));
                ui.label(
                    RichText::new(&self.current_game().description)
                        .small()
                        .color(Color32::LIGHT_GRAY),
                );
                ui.separator();

                egui::ScrollArea::vertical().show(ui, |ui| {
                    for (w_idx, world) in self.current_game().worlds.iter().enumerate() {
                        let is_active_world = w_idx == self.current_world_idx;
                        let completed_in_world = world
                            .levels
                            .iter()
                            .filter(|l| self.profile.is_level_completed(&world.id, &l.id))
                            .count();

                        let header_text = format!(
                            "{} {} ({}/{})",
                            world.icon,
                            world.name,
                            completed_in_world,
                            world.levels.len()
                        );

                        let collapse = egui::CollapsingHeader::new(
                            RichText::new(header_text).strong().color(if is_active_world {
                                Color32::from_rgb(100, 180, 255)
                            } else {
                                Color32::WHITE
                            }),
                        )
                        .default_open(is_active_world)
                        .show(ui, |ui| {
                            for (l_idx, level) in world.levels.iter().enumerate() {
                                let is_active_level =
                                    is_active_world && l_idx == self.current_level_idx;
                                let is_done =
                                    self.profile.is_level_completed(&world.id, &level.id);

                                let icon = if is_done { "✅ " } else { "⚪ " };
                                let label_text = format!("{}{}", icon, level.title);

                                let response = ui.selectable_label(
                                    is_active_level,
                                    RichText::new(label_text).color(if is_done {
                                        Color32::from_rgb(120, 220, 120)
                                    } else {
                                        Color32::LIGHT_GRAY
                                    }),
                                );

                                if response.clicked() {
                                    switch_to = Some((w_idx, l_idx));
                                }
                            }
                        });

                        if collapse.header_response.clicked() {
                            switch_world = Some(w_idx);
                        }

                        ui.add_space(2.0);
                    }
                });
            });

        if let Some((w_idx, l_idx)) = switch_to {
            self.save_current_state();
            self.current_world_idx = w_idx;
            self.current_level_idx = l_idx;
            self.load_active_level();
        } else if let Some(w_idx) = switch_world {
            self.current_world_idx = w_idx;
        }

        // Right Panel: Infoview & Theorem Inventory
        egui::SidePanel::right("infoview_panel")
            .resizable(true)
            .default_width(320.0)
            .show(ctx, |ui| {
                ui.heading("Lean 4 Infoview");
                ui.separator();

                // Infoview Status
                egui::Frame::group(ui.style()).show(ui, |ui| {
                    if let Some(res) = &self.verification {
                        if res.is_complete {
                            ui.label(
                                RichText::new("🎉 Goals accomplished!\nLevel Completed! 🏆")
                                    .color(Color32::from_rgb(100, 255, 120))
                                    .strong()
                                    .size(16.0),
                            );
                            ui.label(
                                RichText::new(format!(
                                    "Proof completed in {} tactic steps.",
                                    res.step_count
                                ))
                                .small(),
                            );
                        } else if let Some(err) = &res.first_error {
                            ui.label(
                                RichText::new("❌ Proof Error")
                                    .color(Color32::from_rgb(255, 100, 100))
                                    .strong(),
                            );
                            ui.label(
                                RichText::new(err)
                                    .color(Color32::from_rgb(255, 150, 150))
                                    .monospace(),
                            );
                            ui.add_space(4.0);
                            ui.label(RichText::new("Current Goal State:").strong());
                            ui.label(
                                RichText::new(format!("{}", res.final_state))
                                    .monospace()
                                    .color(Color32::WHITE),
                            );
                        } else {
                            ui.label(
                                RichText::new(format!("{}", res.final_state))
                                    .monospace()
                                    .size(14.0)
                                    .color(Color32::WHITE),
                            );
                        }
                    } else {
                        let level = self.current_level();
                        let init_state = level.build_initial_state();
                        ui.label(
                            RichText::new(format!("{}", init_state))
                                .monospace()
                                .size(14.0)
                                .color(Color32::WHITE),
                        );
                    }
                });

                ui.add_space(8.0);
                ui.heading("Inventory (Theorems & Axioms)");
                ui.separator();

                let mut insert_thm_code: Option<String> = None;
                let mut sorted_theorems: Vec<_> = self.env.theorems.values().cloned().collect();
                sorted_theorems.sort_by(|a, b| a.name.cmp(&b.name));

                egui::ScrollArea::vertical().show(ui, |ui| {
                    for thm in &sorted_theorems {
                        let is_unlocked = thm.is_axiom
                            || self.profile.unlocked_theorems.contains(&thm.name);

                        let prefix = if thm.is_axiom { "📜 [Axiom]" } else { "⭐ [Theorem]" };
                        ui.collapsing(
                            RichText::new(format!("{} {}", prefix, thm.name)).color(
                                if is_unlocked {
                                    Color32::from_rgb(255, 215, 100)
                                } else {
                                    Color32::GRAY
                                },
                            ),
                            |ui| {
                                ui.label(
                                    RichText::new(format!(": {}", thm.statement))
                                        .monospace()
                                        .color(Color32::from_rgb(180, 220, 255)),
                                );
                                ui.label(RichText::new(&thm.doc).small());
                                if ui.small_button(format!("Insert `rw [{}]`", thm.name)).clicked() {
                                    insert_thm_code = Some(thm.name.clone());
                                }
                            },
                        );
                    }
                });

                if let Some(name) = insert_thm_code {
                    if !self.editor_script.is_empty() && !self.editor_script.ends_with('\n') {
                        self.editor_script.push('\n');
                    }
                    self.editor_script.push_str(&format!("rw [{}]\n", name));
                    self.run_verification();
                }
            });

        // Center Panel: Level Header, Lore, Tactic Bar, Code Editor
        egui::CentralPanel::default().show(ctx, |ui| {
            let level = self.current_level().clone();

            // Status Banner if any
            if let Some((msg, is_success)) = &self.status_banner {
                let color = if *is_success {
                    Color32::from_rgb(40, 120, 40)
                } else {
                    Color32::from_rgb(140, 40, 40)
                };
                egui::Frame::none().fill(color).show(ui, |ui| {
                    ui.label(RichText::new(msg).color(Color32::WHITE).strong());
                });
                ui.add_space(4.0);
            }

            ui.horizontal(|ui| {
                ui.heading(RichText::new(&level.title).size(20.0).strong());
                if self.profile.is_level_completed(&level.world_id, &level.id) {
                    ui.label(
                        RichText::new("SOLVED")
                            .color(Color32::BLACK)
                            .background_color(Color32::from_rgb(100, 255, 120))
                            .strong(),
                    );
                }
            });

            ui.horizontal(|ui| {
                ui.label(
                    RichText::new(format!("Theorem {}: {}", level.theorem_name, level.target_str))
                        .monospace()
                        .size(15.0)
                        .color(Color32::from_rgb(150, 200, 255)),
                );
            });

            ui.separator();

            // Lore / Description
            egui::CollapsingHeader::new(RichText::new("📖 Mathematical Briefing").strong())
                .default_open(true)
                .show(ui, |ui| {
                    ui.label(&level.doc);
                });

            // Hints
            if !level.hints.is_empty() {
                egui::CollapsingHeader::new(RichText::new("💡 Hints & Suggestions").strong())
                    .default_open(false)
                    .show(ui, |ui| {
                        for (idx, hint) in level.hints.iter().enumerate() {
                            ui.label(format!("{}. {}", idx + 1, hint));
                        }
                    });
            }

            ui.add_space(4.0);

            // Quick Tactic Palette
            ui.horizontal_wrapped(|ui| {
                ui.label(RichText::new("Tactics:").small());
                let quick_tactics = [
                    ("rfl", "rfl\n"),
                    ("rw", "rw ["),
                    ("rw ←", "rw [← "),
                    ("induction", "induction "),
                    ("exact", "exact "),
                    ("intro", "intro "),
                    ("apply", "apply "),
                    ("use", "use "),
                    ("constructor", "constructor\n"),
                    ("rcases", "rcases "),
                    ("linarith", "linarith\n"),
                    ("dsimp", "dsimp\n"),
                    ("symm", "symm\n"),
                    ("·", "· "),
                    ("sorry", "sorry\n"),
                ];

                for (label, snippet) in quick_tactics {
                    if ui.button(label).clicked() {
                        self.editor_script.push_str(snippet);
                        self.run_verification();
                    }
                }
            });

            ui.separator();

            // Editor Actions
            ui.horizontal(|ui| {
                if ui.button(RichText::new("▶ Verify Proof").strong().color(Color32::WHITE)).clicked() {
                    self.run_verification();
                    self.save_current_state();
                }

                if ui.button("Clear Script").clicked() {
                    self.editor_script.clear();
                    self.run_verification();
                    self.save_current_state();
                }

                if ui.button("Insert Canonical Solution").clicked() {
                    self.editor_script = level.reference_solution.clone();
                    self.run_verification();
                    self.save_current_state();
                }

                // Next level / Next World / Next Game button
                if self.current_level_idx + 1 < self.current_world().levels.len() {
                    if ui.button("Next Level ➡").clicked() {
                        self.save_current_state();
                        self.current_level_idx += 1;
                        self.load_active_level();
                    }
                } else if self.current_world_idx + 1 < self.current_game().worlds.len() {
                    if ui.button("Next World ➡").clicked() {
                        self.save_current_state();
                        self.current_world_idx += 1;
                        self.current_level_idx = 0;
                        self.load_active_level();
                    }
                } else if self.current_game_idx + 1 < self.games.len() {
                    if ui.button("Next Game ➡").clicked() {
                        self.save_current_state();
                        self.current_game_idx += 1;
                        self.current_world_idx = 0;
                        self.current_level_idx = 0;
                        self.profile.active_game_id = self.current_game().id.clone();
                        self.sync_environment_from_profile();
                        self.load_active_level();
                    }
                }
            });

            ui.add_space(4.0);

            // Monospace Proof Editor
            egui::ScrollArea::vertical().show(ui, |ui| {
                let text_edit = egui::TextEdit::multiline(&mut self.editor_script)
                    .font(egui::TextStyle::Monospace)
                    .desired_rows(14)
                    .desired_width(f32::INFINITY);

                let response = ui.add(text_edit);
                if response.changed() {
                    self.run_verification();
                }
            });
        });
    }
}

impl Lean4GameApp {
    fn render_profile_modal(&mut self, ctx: &egui::Context) {
        if !self.show_profile_dialog {
            return;
        }

        let profiles = self.available_profiles.clone();
        let mut switch_profile: Option<String> = None;
        let mut create_new: Option<String> = None;

        egui::Window::new("Select / Create Profile")
            .collapsible(false)
            .resizable(false)
            .anchor(egui::Align2::CENTER_CENTER, Vec2::ZERO)
            .show(ctx, |ui| {
                ui.heading("Profiles in Local Filesystem");
                ui.label(format!("Directory: {:?}", self.save_manager.base_dir()));
                ui.separator();

                for p_name in &profiles {
                    ui.horizontal(|ui| {
                        let is_current = p_name == &self.profile.name;
                        if ui.selectable_label(is_current, p_name).clicked() && !is_current {
                            switch_profile = Some(p_name.clone());
                        }
                    });
                }

                ui.separator();
                ui.horizontal(|ui| {
                    ui.label("New Profile:");
                    ui.text_edit_singleline(&mut self.new_profile_input);
                    if ui.button("Create").clicked() && !self.new_profile_input.trim().is_empty() {
                        create_new = Some(self.new_profile_input.trim().to_string());
                    }
                });

                ui.add_space(8.0);
                if ui.button("Close").clicked() {
                    self.show_profile_dialog = false;
                }
            });

        if let Some(p_name) = switch_profile {
            if let Ok(loaded) = self.save_manager.load_profile(&p_name) {
                self.profile = loaded;
                self.current_game_idx = self
                    .games
                    .iter()
                    .position(|g| g.id == self.profile.active_game_id)
                    .unwrap_or(0);
                self.current_world_idx = 0;
                self.current_level_idx = 0;
                self.sync_environment_from_profile();
                self.load_active_level();
            }
        }

        if let Some(name) = create_new {
            if let Ok(new_p) = self.save_manager.create_profile(&name) {
                self.profile = new_p;
                self.current_game_idx = self
                    .games
                    .iter()
                    .position(|g| g.id == self.profile.active_game_id)
                    .unwrap_or(0);
                self.current_world_idx = 0;
                self.current_level_idx = 0;
                self.available_profiles = self.save_manager.list_profiles().unwrap_or_default();
                self.new_profile_input.clear();
                self.sync_environment_from_profile();
                self.load_active_level();
            }
        }
    }

    fn render_export_import_modal(&mut self, ctx: &egui::Context) {
        if !self.show_export_import_dialog {
            return;
        }

        egui::Window::new("Backup / Export / Import Progress")
            .collapsible(false)
            .resizable(false)
            .anchor(egui::Align2::CENTER_CENTER, Vec2::ZERO)
            .show(ctx, |ui| {
                ui.heading("Filesystem Backup & Portability");
                ui.separator();

                ui.label(RichText::new("Export Current Profile to File:").strong());
                ui.horizontal(|ui| {
                    ui.text_edit_singleline(&mut self.export_path_input);
                    if ui.button("Export JSON").clicked() {
                        let path = PathBuf::from(&self.export_path_input);
                        match self.save_manager.export_profile_to_file(&self.profile.name, &path) {
                            Ok(_) => {
                                self.status_banner = Some((
                                    format!("Successfully exported profile to {:?}", path),
                                    true,
                                ));
                            }
                            Err(e) => {
                                self.status_banner = Some((format!("Export failed: {}", e), false));
                            }
                        }
                    }
                });

                ui.separator();
                ui.label(RichText::new("Import Profile from File:").strong());
                ui.horizontal(|ui| {
                    ui.text_edit_singleline(&mut self.import_path_input);
                    if ui.button("Import JSON").clicked() {
                        let path = PathBuf::from(&self.import_path_input);
                        match self.save_manager.import_profile_from_file(&path) {
                            Ok(imported) => {
                                self.profile = imported;
                                self.current_game_idx = self
                                    .games
                                    .iter()
                                    .position(|g| g.id == self.profile.active_game_id)
                                    .unwrap_or(0);
                                self.current_world_idx = 0;
                                self.current_level_idx = 0;
                                self.sync_environment_from_profile();
                                self.load_active_level();
                                self.status_banner = Some((
                                    format!("Successfully imported profile '{}'", self.profile.name),
                                    true,
                                ));
                            }
                            Err(e) => {
                                self.status_banner = Some((format!("Import failed: {}", e), false));
                            }
                        }
                    }
                });

                ui.add_space(8.0);
                if ui.button("Done").clicked() {
                    self.show_export_import_dialog = false;
                }
            });
    }
}
