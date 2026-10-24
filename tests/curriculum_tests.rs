use lean4game::cli::TerminalRunner;
use lean4game::engine::environment::Environment;
use lean4game::engine::verifier::Verifier;
use lean4game::game::registry::CurriculumRegistry;

#[test]
fn test_all_tutorial_world_levels() {
    let world = CurriculumRegistry::find_level("tutorial", "tutorial_1").unwrap();
    assert_eq!(world.title, "Level 1: The `rfl` tactic");

    let worlds = CurriculumRegistry::get_worlds();
    let tutorial = worlds.iter().find(|w| w.id == "tutorial").unwrap();

    let env = Environment::standard_peano();
    for level in &tutorial.levels {
        let state = level.build_initial_state();
        let res = Verifier::verify_script(&state, &level.reference_solution, &env);
        assert!(
            res.is_complete,
            "Failed tutorial level '{}': error = {:?}",
            level.id, res.first_error
        );
    }
}

#[test]
fn test_addition_world_levels() {
    let worlds = CurriculumRegistry::get_worlds();
    let addition = worlds.iter().find(|w| w.id == "addition").unwrap();

    let mut env = Environment::standard_peano();
    for level in &addition.levels {
        let state = level.build_initial_state();
        let res = Verifier::verify_script(&state, &level.reference_solution, &env);
        assert!(
            res.is_complete,
            "Failed addition level '{}': error = {:?}",
            level.id, res.first_error
        );
        // Unlock proved theorem for next levels
        env.add_theorem(level.to_theorem());
    }
}

#[test]
fn test_implication_world_levels() {
    let worlds = CurriculumRegistry::get_worlds();
    let imp = worlds.iter().find(|w| w.id == "implication").unwrap();

    let mut env = Environment::standard_peano();
    for level in &imp.levels {
        let state = level.build_initial_state();
        let res = Verifier::verify_script(&state, &level.reference_solution, &env);
        assert!(
            res.is_complete,
            "Failed implication level '{}': error = {:?}",
            level.id, res.first_error
        );
        env.add_theorem(level.to_theorem());
    }
}

#[test]
fn test_inequality_world_levels() {
    let worlds = CurriculumRegistry::get_worlds();
    let ineq = worlds.iter().find(|w| w.id == "inequality").unwrap();

    let mut env = Environment::standard_peano();
    for level in &ineq.levels {
        let state = level.build_initial_state();
        let res = Verifier::verify_script(&state, &level.reference_solution, &env);
        assert!(
            res.is_complete,
            "Failed inequality level '{}': error = {:?}",
            level.id, res.first_error
        );
        env.add_theorem(level.to_theorem());
    }
}

#[test]
fn test_all_real_analysis_levels() {
    let game = CurriculumRegistry::get_game("real_analysis").expect("Real analysis game package");
    let mut env = Environment::standard_for_game(&game.id);

    for world in &game.worlds {
        for level in &world.levels {
            let state = level.build_initial_state();
            let res = Verifier::verify_script(&state, &level.reference_solution, &env);
            assert!(
                res.is_complete,
                "Failed real analysis level '{}/{}': error = {:?}",
                world.id, level.id, res.first_error
            );
            env.add_theorem(level.to_theorem());
        }
    }
}

#[test]
fn test_all_set_theory_levels() {
    let game = CurriculumRegistry::get_game("set_theory").expect("Set theory game package");
    let mut env = Environment::standard_for_game(&game.id);

    for world in &game.worlds {
        for level in &world.levels {
            let state = level.build_initial_state();
            let res = Verifier::verify_script(&state, &level.reference_solution, &env);
            assert!(
                res.is_complete,
                "Failed set theory level '{}/{}': error = {:?}",
                world.id, level.id, res.first_error
            );
            env.add_theorem(level.to_theorem());
        }
    }
}

#[test]
fn test_curriculum_batch_runner() {
    // Verifies all levels across built-in games (Natural Numbers, Real Analysis, Set Theory)
    let res = TerminalRunner::test_all_curriculum();
    assert!(res.is_ok(), "Curriculum runner failed: {:?}", res.err());
}

#[test]
fn test_custom_game_loading_and_verification() {
    use std::path::Path;

    // Verify custom game deserialization from file
    let path = Path::new("games/example_custom_game.json");
    assert!(path.exists(), "example_custom_game.json should exist");

    let custom_game = CurriculumRegistry::load_game_from_file(path).expect("Valid custom game JSON");
    assert_eq!(custom_game.id, "custom_algebra");
    assert_eq!(custom_game.worlds.len(), 1);
    assert_eq!(custom_game.worlds[0].levels.len(), 2);

    // Verify it automatically appears in CurriculumRegistry::get_games()
    let all_games = CurriculumRegistry::get_games();
    let found = all_games.iter().find(|g| g.id == "custom_algebra");
    assert!(found.is_some(), "Custom game should be automatically registered");

    // Verify solving levels from the custom game
    let env = Environment::standard_peano();
    for level in &custom_game.worlds[0].levels {
        let state = level.build_initial_state();
        let res = Verifier::verify_script(&state, &level.reference_solution, &env);
        assert!(
            res.is_complete,
            "Custom level '{}' should be solvable: {:?}",
            level.id, res.first_error
        );
    }
}
