use crate::engine::environment::Environment;
use crate::engine::verifier::Verifier;
use crate::game::registry::CurriculumRegistry;
use crate::storage::profile::UserProfile;
use crate::storage::save_manager::SaveManager;
use std::io::{self, Write};
use std::path::PathBuf;

pub struct TerminalRunner;

impl TerminalRunner {
    /// Runs an automated batch verification of all canonical level solutions in the curriculum across all games.
    pub fn test_all_curriculum() -> Result<(), String> {
        println!("=======================================================");
        println!("  LEAN 4 Game - Canonical Curriculum Verification Test");
        println!("=======================================================");

        let games = CurriculumRegistry::get_games();
        let mut total_levels = 0;
        let mut passed_levels = 0;

        for game in &games {
            println!("\n=======================================================");
            println!("🎮 Game: {} {} (by {})", game.icon, game.title, game.author);
            println!("   {}", game.description);
            println!("=======================================================");

            let mut env = Environment::standard_for_game(&game.id);

            for world in &game.worlds {
                println!("\n  🌍 World: {} ({})", world.name, world.id);
                for level in &world.levels {
                    total_levels += 1;
                    let initial_state = level.build_initial_state();
                    let result = Verifier::verify_script(&initial_state, &level.reference_solution, &env);

                    if result.is_complete {
                        passed_levels += 1;
                        println!(
                            "    ✅ [{}/{}] {} -> Proved in {} steps",
                            world.id, level.id, level.title, result.step_count
                        );
                        // Add newly proven theorem to environment so subsequent levels can use it!
                        env.add_theorem(level.to_theorem());
                    } else {
                        println!(
                            "    ❌ [{}/{}] {} -> FAILED! Error: {:?}",
                            world.id, level.id, level.title, result.first_error
                        );
                        return Err(format!(
                            "Failed canonical solution in game '{}', world '{}', level '{}': {:?}",
                            game.id, world.id, level.id, result.first_error
                        ));
                    }
                }
            }
        }

        println!("\n-------------------------------------------------------");
        println!("🎉 Verification Summary: {}/{} Levels Passed Across All Games!", passed_levels, total_levels);
        println!("=======================================================");

        Ok(())
    }

    /// Play an interactive level from the terminal.
    pub fn play_interactive(world_id: &str, level_num: usize) -> Result<(), String> {
        let level = CurriculumRegistry::find_level_by_number(world_id, level_num)
            .ok_or_else(|| format!("World '{}' level {} not found", world_id, level_num))?;

        let save_manager = SaveManager::new(SaveManager::default_saves_dir());
        let mut profile = save_manager
            .load_profile("Default")
            .unwrap_or_else(|_| UserProfile::new("Default"));

        let mut env = Environment::standard_all();
        // Load unlocked theorems
        let worlds = CurriculumRegistry::get_worlds();
        for w in &worlds {
            for l in &w.levels {
                if profile.is_level_completed(&w.id, &l.id) {
                    env.add_theorem(l.to_theorem());
                }
            }
        }

        println!("\n=======================================================");
        println!("  LEAN 4 Game - {}", level.title);
        println!("  Theorem {}: {}", level.theorem_name, level.target_str);
        println!("=======================================================");
        println!("{}\n", level.doc);

        let initial_state = level.build_initial_state();
        let mut script = profile
            .get_script(&level.world_id, &level.id)
            .unwrap_or_default();

        if !script.is_empty() {
            println!("Loaded existing script:\n{}", script);
            let res = Verifier::verify_script(&initial_state, &script, &env);
            println!("\nState:\n{}", res.final_state);
        } else {
            println!("Initial State:\n{}", initial_state);
        }

        println!("\nEnter proof script (type 'DONE' on a new line when finished, or 'SOLVE' to use reference solution):");

        let mut lines = Vec::new();
        let stdin = io::stdin();
        loop {
            print!("> ");
            let _ = io::stdout().flush();
            let mut line = String::new();
            if stdin.read_line(&mut line).is_err() {
                break;
            }
            let trimmed = line.trim();
            if trimmed == "DONE" {
                break;
            }
            if trimmed == "SOLVE" {
                script = level.reference_solution.clone();
                break;
            }
            lines.push(trimmed.to_string());
        }

        if script.is_empty() {
            script = lines.join("\n");
        }

        let result = Verifier::verify_script(&initial_state, &script, &env);
        if result.is_complete {
            println!("\n🎉 PROOF COMPLETE! You solved {}!", level.title);
            profile.mark_completed(
                &level.world_id,
                &level.id,
                &script,
                result.step_count,
                Some(&level.theorem_name),
            );
            save_manager.save_profile(&profile)?;
            println!("Progress automatically saved to {:?}", save_manager.base_dir());
        } else {
            println!("\nProof incomplete or has errors:\n{:?}", result.first_error);
            profile.save_draft(&level.world_id, &level.id, &script);
            save_manager.save_profile(&profile)?;
            println!("Draft saved to {:?}", save_manager.base_dir());
        }

        Ok(())
    }

    /// List all profiles in the saves directory.
    pub fn list_profiles(saves_dir: Option<PathBuf>) -> Result<(), String> {
        let dir = saves_dir.unwrap_or_else(SaveManager::default_saves_dir);
        let sm = SaveManager::new(dir);
        let profiles = sm.list_profiles()?;
        println!("Profiles in {:?}:", sm.base_dir());
        for p in profiles {
            let loaded = sm.load_profile(&p)?;
            println!(
                "  - {} (Active Game: '{}', Solved: {} levels, Last played: {})",
                p,
                loaded.active_game_id,
                loaded.total_completed(),
                loaded.last_played
            );
        }
        Ok(())
    }

    /// List all game packages and worlds in the curriculum.
    pub fn list_games() {
        println!("=======================================================");
        println!("  LEAN 4 Game - Available Mathematical Games");
        println!("=======================================================");
        for game in CurriculumRegistry::get_games() {
            println!("\n{} {} (ID: {})", game.icon, game.title, game.id);
            println!("   Author: {}", game.author);
            println!("   {}", game.description);
            println!("   Worlds ({} total levels):", game.total_levels());
            for w in &game.worlds {
                println!(
                    "     • {} {} (ID: '{}', {} levels)",
                    w.icon,
                    w.name,
                    w.id,
                    w.levels.len()
                );
            }
        }
        println!("=======================================================");
    }
}
