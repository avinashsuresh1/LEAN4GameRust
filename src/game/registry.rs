use crate::game::level::Level;
use crate::game::package::GamePackage;
use crate::game::world::World;
use std::path::{Path, PathBuf};

fn s_vec(vals: &[&str]) -> Vec<String> {
    vals.iter().map(|v| v.to_string()).collect()
}

fn s_pairs(vals: &[(&str, &str)]) -> Vec<(String, String)> {
    vals.iter().map(|(a, b)| (a.to_string(), b.to_string())).collect()
}

pub struct CurriculumRegistry;

impl CurriculumRegistry {
    /// Directory for user-created custom games.
    pub fn custom_games_dir() -> PathBuf {
        PathBuf::from("games")
    }

    /// Deserializes a GamePackage from a JSON string.
    pub fn load_game_from_json(json_str: &str) -> Result<GamePackage, String> {
        serde_json::from_str(json_str).map_err(|e| format!("Failed to parse game JSON: {}", e))
    }

    /// Loads a GamePackage from a JSON file.
    pub fn load_game_from_file(path: &Path) -> Result<GamePackage, String> {
        let content = std::fs::read_to_string(path)
            .map_err(|e| format!("Failed to read game file {:?}: {}", path, e))?;
        Self::load_game_from_json(&content)
    }

    /// Scans a directory for *.json files and loads valid custom GamePackages.
    pub fn load_custom_games_from_dir(dir: &Path) -> Vec<GamePackage> {
        let mut custom = Vec::new();
        if dir.exists() && dir.is_dir() {
            if let Ok(entries) = std::fs::read_dir(dir) {
                for entry in entries.flatten() {
                    let path = entry.path();
                    if path.extension().and_then(|e| e.to_str()) == Some("json") {
                        if let Ok(pkg) = Self::load_game_from_file(&path) {
                            custom.push(pkg);
                        }
                    }
                }
            }
        }
        custom
    }

    pub fn get_games() -> Vec<GamePackage> {
        let mut games = vec![
            GamePackage {
                id: "natural_numbers".to_string(),
                title: "Natural Number Game (NNG4)".to_string(),
                description: "Build the arithmetic of natural numbers starting from Peano axioms.".to_string(),
                author: "Kevin Buzzard & Mohammad Pedramfar".to_string(),
                icon: "🌱".to_string(),
                worlds: vec![
                    Self::tutorial_world(),
                    Self::addition_world(),
                    Self::multiplication_world(),
                    Self::power_world(),
                    Self::implication_world(),
                    Self::inequality_world(),
                ],
            },
            GamePackage {
                id: "real_analysis".to_string(),
                title: "Real Analysis, The Game".to_string(),
                description: "Master the foundations of real analysis: metric properties, epsilon-N sequence limits, epsilon-delta continuity, and topological completeness.".to_string(),
                author: "Prof. Alex Kontorovich (Rutgers University)".to_string(),
                icon: "📈".to_string(),
                worlds: vec![
                    Self::real_foundations_world(),
                    Self::sequence_limits_world(),
                    Self::continuity_world(),
                    Self::ivt_world(),
                ],
            },
            GamePackage {
                id: "set_theory".to_string(),
                title: "Set Theory & Logic Game".to_string(),
                description: "Construct modern mathematical set theory: subsets, intersections, unions, and boolean propositions.".to_string(),
                author: "Lean Community".to_string(),
                icon: "🔷".to_string(),
                worlds: vec![
                    Self::subset_world(),
                    Self::boolean_world(),
                ],
            },
        ];

        // Automatically merge any custom games found in ./games/
        let custom = Self::load_custom_games_from_dir(&Self::custom_games_dir());
        for g in custom {
            if !games.iter().any(|existing| existing.id == g.id) {
                games.push(g);
            }
        }

        games
    }

    pub fn get_game(game_id: &str) -> Option<GamePackage> {
        Self::get_games().into_iter().find(|g| g.id == game_id)
    }

    pub fn get_worlds() -> Vec<World> {
        Self::get_games()
            .into_iter()
            .flat_map(|g| g.worlds)
            .collect()
    }

    pub fn find_level(world_id: &str, level_id: &str) -> Option<Level> {
        for world in Self::get_worlds() {
            if world.id == world_id {
                if let Some(level) = world.get_level(level_id) {
                    return Some(level.clone());
                }
            }
        }
        None
    }

    pub fn find_level_by_number(world_id: &str, number: usize) -> Option<Level> {
        for world in Self::get_worlds() {
            if world.id == world_id {
                if let Some(level) = world.get_level_by_number(number) {
                    return Some(level.clone());
                }
            }
        }
        None
    }

    fn tutorial_world() -> World {
        World {
            id: "tutorial".to_string(),
            name: "Tutorial World".to_string(),
            description: "Learn the foundational mechanics of interactive theorem proving: reflexivity, substitution, and successor arithmetic.".to_string(),
            icon: "🌱".to_string(),
            levels: vec![
                Level {
                    id: "tutorial_1".to_string(),
                    world_id: "tutorial".to_string(),
                    number: 1,
                    title: "Level 1: The `rfl` tactic".to_string(),
                    theorem_name: "example_rfl".to_string(),
                    doc: "Welcome to the Lean 4 Game!\n\nIn Lean, mathematics is constructed through rigorous logical proofs. When the left-hand side of an equality is identical to the right-hand side, we use the `rfl` (reflexivity) tactic to close the goal.".to_string(),
                    hints: s_vec(&[
                        "Type `rfl` and hit Enter or click Verify to solve this level!",
                    ]),
                    context_vars: s_pairs(&[("x", "ℕ")]),
                    initial_hypotheses: vec![],
                    target_str: "x + 0 = x + 0".to_string(),
                    available_tactics: s_vec(&["rfl"]),
                    reference_solution: "rfl".to_string(),
                },
                Level {
                    id: "tutorial_2".to_string(),
                    world_id: "tutorial".to_string(),
                    number: 2,
                    title: "Level 2: The `rw` tactic".to_string(),
                    theorem_name: "example_rw".to_string(),
                    doc: "The rewrite tactic `rw [axiom_name]` allows us to substitute equals for equals using known axioms or theorems.\n\nHere, the axiom `add_zero (n : ℕ) : n + 0 = n` says that adding zero to any number leaves it unchanged.".to_string(),
                    hints: s_vec(&[
                        "Try typing: `rw [add_zero]`",
                    ]),
                    context_vars: s_pairs(&[("x", "ℕ")]),
                    initial_hypotheses: vec![],
                    target_str: "x + 0 = x".to_string(),
                    available_tactics: s_vec(&["rfl", "rw"]),
                    reference_solution: "rw [add_zero]".to_string(),
                },
                Level {
                    id: "tutorial_3".to_string(),
                    world_id: "tutorial".to_string(),
                    number: 3,
                    title: "Level 3: Successors and `add_succ`".to_string(),
                    theorem_name: "example_add_succ".to_string(),
                    doc: "Peano arithmetic defines every natural number in terms of `0` and the successor function `succ`.\n\nThe axiom `add_succ (n m : ℕ) : n + succ m = succ (n + m)` defines addition recursively.".to_string(),
                    hints: s_vec(&[
                        "Use `rw [add_succ]` to transform the left side into the right side.",
                    ]),
                    context_vars: s_pairs(&[("a", "ℕ"), ("b", "ℕ")]),
                    initial_hypotheses: vec![],
                    target_str: "a + succ b = succ (a + b)".to_string(),
                    available_tactics: s_vec(&["rfl", "rw"]),
                    reference_solution: "rw [add_succ]".to_string(),
                },
                Level {
                    id: "tutorial_4".to_string(),
                    world_id: "tutorial".to_string(),
                    number: 4,
                    title: "Level 4: Rewriting in reverse `rw [← ...]`".to_string(),
                    theorem_name: "example_rw_reverse".to_string(),
                    doc: "Sometimes you need to replace the right-hand side of an equation with its left-hand side. You can rewrite backwards by putting a left arrow `←` (or `<-`) before the lemma name: `rw [← add_succ]`.".to_string(),
                    hints: s_vec(&[
                        "Use `rw [← add_succ]` or `rw [<- add_succ]`.",
                    ]),
                    context_vars: s_pairs(&[("a", "ℕ"), ("b", "ℕ")]),
                    initial_hypotheses: vec![],
                    target_str: "succ (a + b) = a + succ b".to_string(),
                    available_tactics: s_vec(&["rfl", "rw"]),
                    reference_solution: "rw [← add_succ]".to_string(),
                },
                Level {
                    id: "tutorial_5".to_string(),
                    world_id: "tutorial".to_string(),
                    number: 5,
                    title: "Level 5: Chaining rewrites".to_string(),
                    theorem_name: "example_chained_rw".to_string(),
                    doc: "You can apply multiple rewrites in a single line by separating them with commas: `rw [lemma1, lemma2]`.".to_string(),
                    hints: s_vec(&[
                        "Use `rw [add_zero, add_zero]` to simplify both additions.",
                    ]),
                    context_vars: s_pairs(&[("a", "ℕ"), ("b", "ℕ"), ("c", "ℕ")]),
                    initial_hypotheses: vec![],
                    target_str: "a + (b + 0) + (c + 0) = a + b + c".to_string(),
                    available_tactics: s_vec(&["rfl", "rw"]),
                    reference_solution: "rw [add_zero, add_zero]".to_string(),
                },
                Level {
                    id: "tutorial_6".to_string(),
                    world_id: "tutorial".to_string(),
                    number: 6,
                    title: "Level 6: Combining axioms".to_string(),
                    theorem_name: "example_combining_axioms".to_string(),
                    doc: "Prove that adding `succ 0` to `a` gives `succ a`. You will need both `add_succ` and `add_zero`.".to_string(),
                    hints: s_vec(&[
                        "First turn `a + succ 0` into `succ (a + 0)` using `add_succ`.",
                        "Then simplify `a + 0` with `add_zero`.",
                    ]),
                    context_vars: s_pairs(&[("a", "ℕ")]),
                    initial_hypotheses: vec![],
                    target_str: "a + succ 0 = succ a".to_string(),
                    available_tactics: s_vec(&["rfl", "rw"]),
                    reference_solution: "rw [add_succ, add_zero]".to_string(),
                },
                Level {
                    id: "tutorial_7".to_string(),
                    world_id: "tutorial".to_string(),
                    number: 7,
                    title: "Level 7: Defining 1 as `succ 0`".to_string(),
                    theorem_name: "add_one".to_string(),
                    doc: "In Peano arithmetic, `1` is defined by the theorem `one_eq_succ_zero : 1 = succ 0`.\n\nProve that `a + 1 = succ a`.".to_string(),
                    hints: s_vec(&[
                        "Rewrite with `one_eq_succ_zero`, then `add_succ`, then `add_zero`.",
                    ]),
                    context_vars: s_pairs(&[("a", "ℕ")]),
                    initial_hypotheses: vec![],
                    target_str: "a + 1 = succ a".to_string(),
                    available_tactics: s_vec(&["rfl", "rw"]),
                    reference_solution: "rw [one_eq_succ_zero, add_succ, add_zero]".to_string(),
                },
                Level {
                    id: "tutorial_8".to_string(),
                    world_id: "tutorial".to_string(),
                    number: 8,
                    title: "Level 8: Numeral two".to_string(),
                    theorem_name: "two_eq_succ_one_thm".to_string(),
                    doc: "Congratulations on reaching the final tutorial level! The numeral `2` is defined as `succ 1`. Since Lean recognizes numeral definitions, `rfl` closes this goal directly.".to_string(),
                    hints: s_vec(&[
                        "Use `rfl` to complete Tutorial World!",
                    ]),
                    context_vars: vec![],
                    initial_hypotheses: vec![],
                    target_str: "2 = succ 1".to_string(),
                    available_tactics: s_vec(&["rfl"]),
                    reference_solution: "rfl".to_string(),
                },
            ],
        }
    }

    fn addition_world() -> World {
        World {
            id: "addition".to_string(),
            name: "Addition World".to_string(),
            description: "Master mathematical induction on natural numbers to prove commutativity and associativity of addition.".to_string(),
            icon: "➕".to_string(),
            levels: vec![
                Level {
                    id: "zero_add".to_string(),
                    world_id: "addition".to_string(),
                    number: 1,
                    title: "Level 1: `zero_add`".to_string(),
                    theorem_name: "zero_add".to_string(),
                    doc: "We know `n + 0 = n` by the `add_zero` axiom, but is `0 + n = n`? In Peano arithmetic, addition is defined by recursion on the right argument, so `0 + n = n` requires mathematical induction!\n\nUse `induction n with d hd` to split into base case `0` and inductive step `succ d`.".to_string(),
                    hints: s_vec(&[
                        "Line 1: `induction n with d hd`",
                        "Line 2: `· rw [add_zero]`",
                        "Line 3: `· rw [add_succ, hd]`",
                    ]),
                    context_vars: s_pairs(&[("n", "ℕ")]),
                    initial_hypotheses: vec![],
                    target_str: "0 + n = n".to_string(),
                    available_tactics: s_vec(&["rfl", "rw", "induction"]),
                    reference_solution: "induction n with d hd\n· rw [add_zero]\n· rw [add_succ, hd]".to_string(),
                },
                Level {
                    id: "add_assoc".to_string(),
                    world_id: "addition".to_string(),
                    number: 2,
                    title: "Level 2: `add_assoc`".to_string(),
                    theorem_name: "add_assoc".to_string(),
                    doc: "Prove that addition is associative: `(a + b) + c = a + (b + c)`.\n\nSince addition is defined recursively on the right, perform induction on `c`!".to_string(),
                    hints: s_vec(&[
                        "Use `induction c with d hd`.",
                        "For base case: `rw [add_zero, add_zero]`.",
                        "For step: `rw [add_succ, add_succ, add_succ, hd]`.",
                    ]),
                    context_vars: s_pairs(&[("a", "ℕ"), ("b", "ℕ"), ("c", "ℕ")]),
                    initial_hypotheses: vec![],
                    target_str: "(a + b) + c = a + (b + c)".to_string(),
                    available_tactics: s_vec(&["rfl", "rw", "induction"]),
                    reference_solution: "induction c with d hd\n· rw [add_zero, add_zero]\n· rw [add_succ, add_succ, add_succ, hd]".to_string(),
                },
                Level {
                    id: "succ_add".to_string(),
                    world_id: "addition".to_string(),
                    number: 3,
                    title: "Level 3: `succ_add`".to_string(),
                    theorem_name: "succ_add".to_string(),
                    doc: "The axiom `add_succ` tells us `a + succ b = succ (a + b)`. Now prove that the successor can also be pulled out of the first argument: `succ a + b = succ (a + b)`.".to_string(),
                    hints: s_vec(&[
                        "Induction on `b`!",
                    ]),
                    context_vars: s_pairs(&[("a", "ℕ"), ("b", "ℕ")]),
                    initial_hypotheses: vec![],
                    target_str: "succ a + b = succ (a + b)".to_string(),
                    available_tactics: s_vec(&["rfl", "rw", "induction"]),
                    reference_solution: "induction b with d hd\n· rw [add_zero, add_zero]\n· rw [add_succ, add_succ, hd]".to_string(),
                },
                Level {
                    id: "add_comm".to_string(),
                    world_id: "addition".to_string(),
                    number: 4,
                    title: "Level 4: `add_comm` (Commutativity of Addition)".to_string(),
                    theorem_name: "add_comm".to_string(),
                    doc: "The crown jewel of Addition World! Prove `a + b = b + a`.\n\nYou can use `zero_add` and `succ_add` that you proved in previous levels!".to_string(),
                    hints: s_vec(&[
                        "Induction on `b` with `d` and `hd`.",
                        "Base case: `rw [add_zero, zero_add]`.",
                        "Inductive step: `rw [add_succ, succ_add, hd]`.",
                    ]),
                    context_vars: s_pairs(&[("a", "ℕ"), ("b", "ℕ")]),
                    initial_hypotheses: vec![],
                    target_str: "a + b = b + a".to_string(),
                    available_tactics: s_vec(&["rfl", "rw", "induction"]),
                    reference_solution: "induction b with d hd\n· rw [add_zero, zero_add]\n· rw [add_succ, succ_add, hd]".to_string(),
                },
                Level {
                    id: "add_right_comm".to_string(),
                    world_id: "addition".to_string(),
                    number: 5,
                    title: "Level 5: `add_right_comm`".to_string(),
                    theorem_name: "add_right_comm".to_string(),
                    doc: "Prove that `(a + b) + c = (a + c) + b`. You can prove this using induction on `c`!".to_string(),
                    hints: s_vec(&[
                        "Induction on `c with d hd`.",
                        "Base case: `rw [add_zero, add_zero]`.",
                        "Step: `rw [add_succ, add_succ, succ_add, hd]`.",
                    ]),
                    context_vars: s_pairs(&[("a", "ℕ"), ("b", "ℕ"), ("c", "ℕ")]),
                    initial_hypotheses: vec![],
                    target_str: "(a + b) + c = (a + c) + b".to_string(),
                    available_tactics: s_vec(&["rfl", "rw", "induction"]),
                    reference_solution: "induction c with d hd\n· rw [add_zero, add_zero]\n· rw [add_succ, add_succ, succ_add, hd]".to_string(),
                },
            ],
        }
    }

    fn multiplication_world() -> World {
        World {
            id: "multiplication".to_string(),
            name: "Multiplication World".to_string(),
            description: "Explore multiplication defined as iterated addition, proving distributivity, associativity, and commutativity.".to_string(),
            icon: "✖️".to_string(),
            levels: vec![
                Level {
                    id: "zero_mul".to_string(),
                    world_id: "multiplication".to_string(),
                    number: 1,
                    title: "Level 1: `zero_mul`".to_string(),
                    theorem_name: "zero_mul".to_string(),
                    doc: "Axiom `mul_zero` gives `n * 0 = 0`. Prove that `0 * n = 0` for all `n : ℕ` using induction on `n`.".to_string(),
                    hints: s_vec(&[
                        "Induction on `n with d hd`.",
                        "Base case: `rw [mul_zero]`.",
                        "Step: `rw [mul_succ, hd, add_zero]`.",
                    ]),
                    context_vars: s_pairs(&[("n", "ℕ")]),
                    initial_hypotheses: vec![],
                    target_str: "0 * n = 0".to_string(),
                    available_tactics: s_vec(&["rfl", "rw", "induction"]),
                    reference_solution: "induction n with d hd\n· rw [mul_zero]\n· rw [mul_succ, hd, add_zero]".to_string(),
                },
                Level {
                    id: "mul_one".to_string(),
                    world_id: "multiplication".to_string(),
                    number: 2,
                    title: "Level 2: `mul_one`".to_string(),
                    theorem_name: "mul_one".to_string(),
                    doc: "Prove that `1` is a right identity for multiplication: `n * 1 = n`.".to_string(),
                    hints: s_vec(&[
                        "Use `rw [one_eq_succ_zero, mul_succ, mul_zero, zero_add]`.",
                    ]),
                    context_vars: s_pairs(&[("n", "ℕ")]),
                    initial_hypotheses: vec![],
                    target_str: "n * 1 = n".to_string(),
                    available_tactics: s_vec(&["rfl", "rw"]),
                    reference_solution: "rw [one_eq_succ_zero, mul_succ, mul_zero, zero_add]".to_string(),
                },
                Level {
                    id: "succ_mul".to_string(),
                    world_id: "multiplication".to_string(),
                    number: 3,
                    title: "Level 3: `succ_mul`".to_string(),
                    theorem_name: "succ_mul".to_string(),
                    doc: "Prove that `succ a * b = a * b + b` by induction on `b`.".to_string(),
                    hints: s_vec(&[
                        "Induction on `b with d hd`.",
                    ]),
                    context_vars: s_pairs(&[("a", "ℕ"), ("b", "ℕ")]),
                    initial_hypotheses: vec![],
                    target_str: "succ a * b = a * b + b".to_string(),
                    available_tactics: s_vec(&["rfl", "rw", "induction"]),
                    reference_solution: "induction b with d hd\n· rw [mul_zero, mul_zero, add_zero]\n· rw [mul_succ, hd, add_succ, mul_succ, add_succ, add_right_comm]".to_string(),
                },
                Level {
                    id: "mul_add".to_string(),
                    world_id: "multiplication".to_string(),
                    number: 4,
                    title: "Level 4: `mul_add` (Distributivity)".to_string(),
                    theorem_name: "mul_add".to_string(),
                    doc: "Prove left distributivity of multiplication over addition: `a * (b + c) = a * b + a * c`.".to_string(),
                    hints: s_vec(&[
                        "Induction on `c with d hd`.",
                    ]),
                    context_vars: s_pairs(&[("a", "ℕ"), ("b", "ℕ"), ("c", "ℕ")]),
                    initial_hypotheses: vec![],
                    target_str: "a * (b + c) = a * b + a * c".to_string(),
                    available_tactics: s_vec(&["rfl", "rw", "induction"]),
                    reference_solution: "induction c with d hd\n· rw [add_zero, mul_zero, add_zero]\n· rw [add_succ, mul_succ, hd, mul_succ, add_assoc]".to_string(),
                },
                Level {
                    id: "mul_assoc".to_string(),
                    world_id: "multiplication".to_string(),
                    number: 5,
                    title: "Level 5: `mul_assoc`".to_string(),
                    theorem_name: "mul_assoc".to_string(),
                    doc: "Prove that multiplication is associative: `(a * b) * c = a * (b * c)`.".to_string(),
                    hints: s_vec(&[
                        "Induction on `c with d hd`.",
                        "Use `mul_add` in the inductive step.",
                    ]),
                    context_vars: s_pairs(&[("a", "ℕ"), ("b", "ℕ"), ("c", "ℕ")]),
                    initial_hypotheses: vec![],
                    target_str: "(a * b) * c = a * (b * c)".to_string(),
                    available_tactics: s_vec(&["rfl", "rw", "induction"]),
                    reference_solution: "induction c with d hd\n· rw [mul_zero, mul_zero, mul_zero]\n· rw [mul_succ, hd, mul_succ, mul_add]".to_string(),
                },
                Level {
                    id: "mul_comm".to_string(),
                    world_id: "multiplication".to_string(),
                    number: 6,
                    title: "Level 6: `mul_comm` (Commutativity of Multiplication)".to_string(),
                    theorem_name: "mul_comm".to_string(),
                    doc: "Prove `a * b = b * a` using induction on `b` together with `zero_mul` and `succ_mul`.".to_string(),
                    hints: s_vec(&[
                        "Induction on `b with d hd`.",
                        "Base case: `rw [mul_zero, zero_mul]`.",
                        "Step: `rw [mul_succ, succ_mul, hd]`.",
                    ]),
                    context_vars: s_pairs(&[("a", "ℕ"), ("b", "ℕ")]),
                    initial_hypotheses: vec![],
                    target_str: "a * b = b * a".to_string(),
                    available_tactics: s_vec(&["rfl", "rw", "induction"]),
                    reference_solution: "induction b with d hd\n· rw [mul_zero, zero_mul]\n· rw [mul_succ, hd, succ_mul]".to_string(),
                },
            ],
        }
    }

    fn power_world() -> World {
        World {
            id: "power".to_string(),
            name: "Power World".to_string(),
            description: "Explore powers and exponentiation, proving laws of exponents starting from zero power.".to_string(),
            icon: "⚡".to_string(),
            levels: vec![
                Level {
                    id: "zero_pow_zero".to_string(),
                    world_id: "power".to_string(),
                    number: 1,
                    title: "Level 1: `zero_pow_zero`".to_string(),
                    theorem_name: "zero_pow_zero".to_string(),
                    doc: "In Peano arithmetic, `a ^ 0 = 1` for all `a : ℕ`, including `0 ^ 0 = 1` by the `pow_zero` axiom.".to_string(),
                    hints: s_vec(&["Use `rw [pow_zero]`."]),
                    context_vars: vec![],
                    initial_hypotheses: vec![],
                    target_str: "0 ^ 0 = 1".to_string(),
                    available_tactics: s_vec(&["rfl", "rw"]),
                    reference_solution: "rw [pow_zero]".to_string(),
                },
                Level {
                    id: "pow_one".to_string(),
                    world_id: "power".to_string(),
                    number: 2,
                    title: "Level 2: `pow_one`".to_string(),
                    theorem_name: "pow_one".to_string(),
                    doc: "Prove that `a ^ 1 = a` for any `a : ℕ`.".to_string(),
                    hints: s_vec(&[
                        "Rewrite `1` with `one_eq_succ_zero`, then `pow_succ`, then `pow_zero`, then `zero_add` / `mul_one`.",
                    ]),
                    context_vars: s_pairs(&[("a", "ℕ")]),
                    initial_hypotheses: vec![],
                    target_str: "a ^ 1 = a".to_string(),
                    available_tactics: s_vec(&["rfl", "rw"]),
                    reference_solution: "rw [one_eq_succ_zero, pow_succ, pow_zero, mul_comm, mul_one]".to_string(),
                },
                Level {
                    id: "one_pow".to_string(),
                    world_id: "power".to_string(),
                    number: 3,
                    title: "Level 3: `one_pow`".to_string(),
                    theorem_name: "one_pow".to_string(),
                    doc: "Prove that `1 ^ a = 1` for all `a : ℕ` using induction on `a`.".to_string(),
                    hints: s_vec(&[
                        "Induction on `a with d hd`.",
                        "Base case: `rw [pow_zero]`.",
                        "Step: `rw [pow_succ, hd, mul_one]`.",
                    ]),
                    context_vars: s_pairs(&[("a", "ℕ")]),
                    initial_hypotheses: vec![],
                    target_str: "1 ^ a = 1".to_string(),
                    available_tactics: s_vec(&["rfl", "rw", "induction"]),
                    reference_solution: "induction a with d hd\n· rw [pow_zero]\n· rw [pow_succ, hd, mul_one]".to_string(),
                },
                Level {
                    id: "pow_add".to_string(),
                    world_id: "power".to_string(),
                    number: 4,
                    title: "Level 4: `pow_add`".to_string(),
                    theorem_name: "pow_add".to_string(),
                    doc: "Prove that `a ^ (m + n) = (a ^ m) * (a ^ n)`.".to_string(),
                    hints: s_vec(&[
                        "Induction on `n with d hd`.",
                    ]),
                    context_vars: s_pairs(&[("a", "ℕ"), ("m", "ℕ"), ("n", "ℕ")]),
                    initial_hypotheses: vec![],
                    target_str: "a ^ (m + n) = (a ^ m) * (a ^ n)".to_string(),
                    available_tactics: s_vec(&["rfl", "rw", "induction"]),
                    reference_solution: "induction n with d hd\n· rw [add_zero, pow_zero, mul_one]\n· rw [add_succ, pow_succ, hd, pow_succ, mul_assoc]".to_string(),
                },
            ],
        }
    }

    fn implication_world() -> World {
        World {
            id: "implication".to_string(),
            name: "Implication World".to_string(),
            description: "Learn constructive logic and deductive reasoning using `intro`, `exact`, and `apply`.".to_string(),
            icon: "🏹".to_string(),
            levels: vec![
                Level {
                    id: "exact_level".to_string(),
                    world_id: "implication".to_string(),
                    number: 1,
                    title: "Level 1: Identity of Implication".to_string(),
                    theorem_name: "imp_self".to_string(),
                    doc: "To prove `P → P`, use `intro h` to assume `P`, then use `exact h` to close the goal.".to_string(),
                    hints: s_vec(&[
                        "Line 1: `intro h`",
                        "Line 2: `exact h`",
                    ]),
                    context_vars: vec![],
                    initial_hypotheses: vec![],
                    target_str: "P → P".to_string(),
                    available_tactics: s_vec(&["intro", "exact"]),
                    reference_solution: "intro h\nexact h".to_string(),
                },
                Level {
                    id: "intro_level".to_string(),
                    world_id: "implication".to_string(),
                    number: 2,
                    title: "Level 2: Weakening (`P → Q → P`)".to_string(),
                    theorem_name: "imp_intro".to_string(),
                    doc: "Prove that `P` implies `Q → P`. Introduce both assumptions!".to_string(),
                    hints: s_vec(&[
                        "Type `intro h1 h2` then `exact h1`.",
                    ]),
                    context_vars: vec![],
                    initial_hypotheses: vec![],
                    target_str: "P → Q → P".to_string(),
                    available_tactics: s_vec(&["intro", "exact"]),
                    reference_solution: "intro h1 h2\nexact h1".to_string(),
                },
                Level {
                    id: "apply_level".to_string(),
                    world_id: "implication".to_string(),
                    number: 3,
                    title: "Level 3: Modus Ponens (`apply`)".to_string(),
                    theorem_name: "modus_ponens".to_string(),
                    doc: "If you have an implication `f : P → Q` and want to prove `Q`, the `apply f` tactic turns the goal into `P`.".to_string(),
                    hints: s_vec(&[
                        "Type `intro f p`, then `apply f`, then `exact p`.",
                    ]),
                    context_vars: vec![],
                    initial_hypotheses: vec![],
                    target_str: "(P → Q) → P → Q".to_string(),
                    available_tactics: s_vec(&["intro", "apply", "exact"]),
                    reference_solution: "intro f p\napply f\nexact p".to_string(),
                },
                Level {
                    id: "imp_trans".to_string(),
                    world_id: "implication".to_string(),
                    number: 4,
                    title: "Level 4: Transitivity of Implication".to_string(),
                    theorem_name: "imp_trans".to_string(),
                    doc: "Prove hypothetical syllogism / transitivity: `(P → Q) → (Q → R) → (P → R)`.".to_string(),
                    hints: s_vec(&[
                        "Intro all assumptions: `intro hpq hqr p`.",
                        "Work backwards: `apply hqr`, `apply hpq`, `exact p`.",
                    ]),
                    context_vars: vec![],
                    initial_hypotheses: vec![],
                    target_str: "(P → Q) → (Q → R) → P → R".to_string(),
                    available_tactics: s_vec(&["intro", "apply", "exact"]),
                    reference_solution: "intro hpq hqr p\napply hqr\napply hpq\nexact p".to_string(),
                },
            ],
        }
    }

    fn inequality_world() -> World {
        World {
            id: "inequality".to_string(),
            name: "Inequality World".to_string(),
            description: "Reason about order, reflexivity, and monotonicity of natural numbers with ≤.".to_string(),
            icon: "⚖️".to_string(),
            levels: vec![
                Level {
                    id: "le_refl_level".to_string(),
                    world_id: "inequality".to_string(),
                    number: 1,
                    title: "Level 1: Reflexivity of `≤`".to_string(),
                    theorem_name: "le_refl_user".to_string(),
                    doc: "Every natural number is less than or equal to itself: `x ≤ x`.".to_string(),
                    hints: s_vec(&["Use `exact le_refl` or `rfl`!"]),
                    context_vars: s_pairs(&[("x", "ℕ")]),
                    initial_hypotheses: vec![],
                    target_str: "x ≤ x".to_string(),
                    available_tactics: s_vec(&["rfl", "exact"]),
                    reference_solution: "exact le_refl".to_string(),
                },
                Level {
                    id: "le_succ_level".to_string(),
                    world_id: "inequality".to_string(),
                    number: 2,
                    title: "Level 2: Successor inequality".to_string(),
                    theorem_name: "le_succ_user".to_string(),
                    doc: "Prove that `x ≤ succ x` using the axiom `le_succ`.".to_string(),
                    hints: s_vec(&["Use `exact le_succ`."]),
                    context_vars: s_pairs(&[("x", "ℕ")]),
                    initial_hypotheses: vec![],
                    target_str: "x ≤ succ x".to_string(),
                    available_tactics: s_vec(&["exact"]),
                    reference_solution: "exact le_succ".to_string(),
                },
                Level {
                    id: "zero_le_level".to_string(),
                    world_id: "inequality".to_string(),
                    number: 3,
                    title: "Level 3: Zero is least".to_string(),
                    theorem_name: "zero_le_user".to_string(),
                    doc: "Zero is smaller than or equal to any natural number: `0 ≤ x`.".to_string(),
                    hints: s_vec(&["Use `exact zero_le`."]),
                    context_vars: s_pairs(&[("x", "ℕ")]),
                    initial_hypotheses: vec![],
                    target_str: "0 ≤ x".to_string(),
                    available_tactics: s_vec(&["exact"]),
                    reference_solution: "exact zero_le".to_string(),
                },
            ],
        }
    }

    fn real_foundations_world() -> World {
        World {
            id: "real_foundations".to_string(),
            name: "Real Foundations & Metrics".to_string(),
            description: "Explore the metric and ordered field properties of real numbers ℝ: absolute values, distances, and the triangle inequality.".to_string(),
            icon: "📏".to_string(),
            levels: vec![
                Level {
                    id: "abs_sub_self_level".to_string(),
                    world_id: "real_foundations".to_string(),
                    number: 1,
                    title: "Level 1: Distance to self is zero".to_string(),
                    theorem_name: "dist_self_zero".to_string(),
                    doc: "In real analysis, distance is defined by |a - b|. Prove that the distance from x to itself is zero.".to_string(),
                    hints: s_vec(&["Rewrite with `sub_self_real`, then `abs_zero`."]),
                    context_vars: s_pairs(&[("x", "ℝ")]),
                    initial_hypotheses: vec![],
                    target_str: "|x - x| = 0".to_string(),
                    available_tactics: s_vec(&["rfl", "rw"]),
                    reference_solution: "rw [sub_self_real, abs_zero]".to_string(),
                },
                Level {
                    id: "abs_symmetry_level".to_string(),
                    world_id: "real_foundations".to_string(),
                    number: 2,
                    title: "Level 2: Distance symmetry".to_string(),
                    theorem_name: "dist_symm".to_string(),
                    doc: "Distance is symmetric: |a - b| = |b - a|.".to_string(),
                    hints: s_vec(&["Use `exact abs_sub_comm`."]),
                    context_vars: s_pairs(&[("a", "ℝ"), ("b", "ℝ")]),
                    initial_hypotheses: vec![],
                    target_str: "|a - b| = |b - a|".to_string(),
                    available_tactics: s_vec(&["exact"]),
                    reference_solution: "exact abs_sub_comm".to_string(),
                },
                Level {
                    id: "triangle_ineq_level".to_string(),
                    world_id: "real_foundations".to_string(),
                    number: 3,
                    title: "Level 3: Triangle Inequality".to_string(),
                    theorem_name: "triangle_inequality".to_string(),
                    doc: "The foundational inequality of analysis: |a + b| ≤ |a| + |b|.".to_string(),
                    hints: s_vec(&["Use `exact triangle_ineq`."]),
                    context_vars: s_pairs(&[("a", "ℝ"), ("b", "ℝ")]),
                    initial_hypotheses: vec![],
                    target_str: "|a + b| ≤ |a| + |b|".to_string(),
                    available_tactics: s_vec(&["exact"]),
                    reference_solution: "exact triangle_ineq".to_string(),
                },
                Level {
                    id: "triangle_sub_level".to_string(),
                    world_id: "real_foundations".to_string(),
                    number: 4,
                    title: "Level 4: Metric Triangle Inequality".to_string(),
                    theorem_name: "triangle_sub_thm".to_string(),
                    doc: "Inserting an intermediate point b: |a - c| ≤ |a - b| + |b - c|.".to_string(),
                    hints: s_vec(&["Use `exact triangle_sub`."]),
                    context_vars: s_pairs(&[("a", "ℝ"), ("b", "ℝ"), ("c", "ℝ")]),
                    initial_hypotheses: vec![],
                    target_str: "|a - c| ≤ |a - b| + |b - c|".to_string(),
                    available_tactics: s_vec(&["exact"]),
                    reference_solution: "exact triangle_sub".to_string(),
                },
            ],
        }
    }

    fn sequence_limits_world() -> World {
        World {
            id: "sequence_limits".to_string(),
            name: "Sequences & ε-N Limits".to_string(),
            description: "Master rigorous epsilon-N definitions of sequence convergence from Alex Kontorovich's Real Analysis Game.".to_string(),
            icon: "🎯".to_string(),
            levels: vec![
                Level {
                    id: "const_seq_limit_level".to_string(),
                    world_id: "sequence_limits".to_string(),
                    number: 1,
                    title: "Level 1: Constant sequence limit".to_string(),
                    theorem_name: "const_seq_limit".to_string(),
                    doc: "Prove that any constant sequence an = c converges to c. For any ε > 0, pick witness N = 0 using `use 0`!".to_string(),
                    hints: s_vec(&[
                        "Line 1: `intro c eps h_eps`",
                        "Line 2: `use 0`",
                        "Line 3: `intro n hn`",
                        "Line 4: `rw [sub_self_real, abs_zero]`",
                        "Line 5: `exact h_eps`",
                    ]),
                    context_vars: vec![],
                    initial_hypotheses: vec![],
                    target_str: "∀ (c : ℝ) (ε : ℝ), ε > 0 → ∃ (N : ℕ), ∀ (n : ℕ), n ≥ N → |c - c| < ε".to_string(),
                    available_tactics: s_vec(&["intro", "use", "rw", "exact"]),
                    reference_solution: "intro c eps h_eps\nuse 0\nintro n hn\nrw [sub_self_real, abs_zero]\nexact h_eps".to_string(),
                },
                Level {
                    id: "seq_shift_level".to_string(),
                    world_id: "sequence_limits".to_string(),
                    number: 2,
                    title: "Level 2: Sequence index shifting".to_string(),
                    theorem_name: "seq_shifted".to_string(),
                    doc: "A convergent sequence satisfies the limit definition for any index witness N ≥ 1.".to_string(),
                    hints: s_vec(&["Use `use 1` as the witness."]),
                    context_vars: vec![],
                    initial_hypotheses: vec![],
                    target_str: "∀ (L : ℝ) (ε : ℝ), ε > 0 → ∃ (N : ℕ), ∀ (n : ℕ), n ≥ N → |L - L| < ε".to_string(),
                    available_tactics: s_vec(&["intro", "use", "rw", "exact"]),
                    reference_solution: "intro L eps h_eps\nuse 1\nintro n hn\nrw [sub_self_real, abs_zero]\nexact h_eps".to_string(),
                },
                Level {
                    id: "unpack_bound_level".to_string(),
                    world_id: "sequence_limits".to_string(),
                    number: 3,
                    title: "Level 3: Unpacking existential bounds (`rcases`)".to_string(),
                    theorem_name: "limit_bound_unpack".to_string(),
                    doc: "Unpack an existential sequence hypothesis using `rcases h with ⟨N1, h1⟩`, then provide the witness with `use N1`.".to_string(),
                    hints: s_vec(&[
                        "`intro h`",
                        "`rcases h with ⟨N1, h1⟩`",
                        "`use N1`",
                        "`apply h1`",
                        "`exact le_refl`",
                    ]),
                    context_vars: vec![],
                    initial_hypotheses: vec![],
                    target_str: "(∃ (N1 : ℕ), ∀ (n : ℕ), n ≥ N1 → P n) → ∃ (N : ℕ), P N".to_string(),
                    available_tactics: s_vec(&["intro", "rcases", "use", "apply", "exact"]),
                    reference_solution: "intro h\nrcases h with ⟨N1, h1⟩\nuse N1\napply h1\nexact le_refl".to_string(),
                },
            ],
        }
    }

    fn continuity_world() -> World {
        World {
            id: "continuity_world".to_string(),
            name: "Continuity & ε-δ Limits".to_string(),
            description: "Rigorous epsilon-delta continuity proofs on the real line.".to_string(),
            icon: "〰️".to_string(),
            levels: vec![
                Level {
                    id: "continuity_id_level".to_string(),
                    world_id: "continuity_world".to_string(),
                    number: 1,
                    title: "Level 1: Continuity of Identity function".to_string(),
                    theorem_name: "continuous_identity".to_string(),
                    doc: "Prove f(x) = x is continuous at c. For any ε > 0, choose δ = ε using `use eps`. Then split the conjunction with `constructor`!".to_string(),
                    hints: s_vec(&[
                        "`intro c eps h_eps`",
                        "`use eps`",
                        "`constructor`",
                        "`exact h_eps`",
                        "`intro x h_delta`",
                        "`exact h_delta`",
                    ]),
                    context_vars: vec![],
                    initial_hypotheses: vec![],
                    target_str: "∀ (c : ℝ) (ε : ℝ), ε > 0 → ∃ (δ : ℝ), δ > 0 ∧ ∀ (x : ℝ), |x - c| < δ → |x - c| < ε".to_string(),
                    available_tactics: s_vec(&["intro", "use", "constructor", "exact"]),
                    reference_solution: "intro c eps h_eps\nuse eps\nconstructor\n· exact h_eps\n· intro x h_delta\n  exact h_delta".to_string(),
                },
                Level {
                    id: "continuity_const_level".to_string(),
                    world_id: "continuity_world".to_string(),
                    number: 2,
                    title: "Level 2: Continuity of Constant function".to_string(),
                    theorem_name: "continuous_constant".to_string(),
                    doc: "A constant function f(x) = k is continuous everywhere. Pick δ = 1, then |k - k| = |0| = 0 < ε.".to_string(),
                    hints: s_vec(&[
                        "`intro k c eps h_eps`",
                        "`use 1`",
                        "`constructor`",
                        "`exact zero_lt_one`",
                        "`intro x h_delta`",
                        "`rw [sub_self_real, abs_zero]`",
                        "`exact h_eps`",
                    ]),
                    context_vars: vec![],
                    initial_hypotheses: vec![],
                    target_str: "∀ (k c : ℝ) (ε : ℝ), ε > 0 → ∃ (δ : ℝ), δ > 0 ∧ ∀ (x : ℝ), |x - c| < δ → |k - k| < ε".to_string(),
                    available_tactics: s_vec(&["intro", "use", "constructor", "rw", "exact"]),
                    reference_solution: "intro k c eps h_eps\nuse 1\nconstructor\n· exact zero_lt_one\n· intro x h_delta\n  rw [sub_self_real, abs_zero]\n  exact h_eps".to_string(),
                },
            ],
        }
    }

    fn ivt_world() -> World {
        World {
            id: "ivt_world".to_string(),
            name: "Intermediate Value & Order".to_string(),
            description: "Foundations of ordering, transitivity, and interval bisection in analysis.".to_string(),
            icon: "⚖️".to_string(),
            levels: vec![
                Level {
                    id: "ivt_prelim_level".to_string(),
                    world_id: "ivt_world".to_string(),
                    number: 1,
                    title: "Level 1: Strict order identity".to_string(),
                    theorem_name: "order_id".to_string(),
                    doc: "Assume a < b and conclude a < b directly.".to_string(),
                    hints: s_vec(&["`intro a b hab` then `exact hab`."]),
                    context_vars: vec![],
                    initial_hypotheses: vec![],
                    target_str: "∀ (a b : ℝ), a < b → a < b".to_string(),
                    available_tactics: s_vec(&["intro", "exact"]),
                    reference_solution: "intro a b hab\nexact hab".to_string(),
                },
                Level {
                    id: "order_trans_level".to_string(),
                    world_id: "ivt_world".to_string(),
                    number: 2,
                    title: "Level 2: Transitivity of real inequality".to_string(),
                    theorem_name: "order_trans".to_string(),
                    doc: "If a ≤ b and b ≤ c, then a ≤ c by `le_trans_real`.".to_string(),
                    hints: s_vec(&["Use `apply le_trans_real`."]),
                    context_vars: vec![],
                    initial_hypotheses: vec![],
                    target_str: "∀ (a b c : ℝ), a ≤ b → b ≤ c → a ≤ c".to_string(),
                    available_tactics: s_vec(&["intro", "apply", "exact"]),
                    reference_solution: "intro a b c hab hbc\napply le_trans_real\n· exact hab\n· exact hbc".to_string(),
                },
            ],
        }
    }

    fn subset_world() -> World {
        World {
            id: "subset_world".to_string(),
            name: "Subsets & Inclusions".to_string(),
            description: "Reasoning with set inclusions, subsets, and elements.".to_string(),
            icon: "⊆".to_string(),
            levels: vec![
                Level {
                    id: "subset_refl_level".to_string(),
                    world_id: "subset_world".to_string(),
                    number: 1,
                    title: "Level 1: Subset reflexivity (A ⊆ A)".to_string(),
                    theorem_name: "subset_refl".to_string(),
                    doc: "Every set is a subset of itself: x ∈ A implies x ∈ A.".to_string(),
                    hints: s_vec(&["`intro x` then `exact x`."]),
                    context_vars: vec![],
                    initial_hypotheses: vec![],
                    target_str: "A → A".to_string(),
                    available_tactics: s_vec(&["intro", "exact"]),
                    reference_solution: "intro x\nexact x".to_string(),
                },
                Level {
                    id: "subset_trans_level".to_string(),
                    world_id: "subset_world".to_string(),
                    number: 2,
                    title: "Level 2: Subset transitivity".to_string(),
                    theorem_name: "subset_trans".to_string(),
                    doc: "If A ⊆ B and B ⊆ C, then A ⊆ C.".to_string(),
                    hints: s_vec(&["`intro hab hbc a` then chain `apply`."]),
                    context_vars: vec![],
                    initial_hypotheses: vec![],
                    target_str: "(A → B) → (B → C) → A → C".to_string(),
                    available_tactics: s_vec(&["intro", "apply", "exact"]),
                    reference_solution: "intro hab hbc a\napply hbc\napply hab\nexact a".to_string(),
                },
            ],
        }
    }

    fn boolean_world() -> World {
        World {
            id: "boolean_world".to_string(),
            name: "Intersections & Unions".to_string(),
            description: "Working with conjunctions, disjunctions, and set intersections.".to_string(),
            icon: "∩".to_string(),
            levels: vec![
                Level {
                    id: "inter_left_level".to_string(),
                    world_id: "boolean_world".to_string(),
                    number: 1,
                    title: "Level 1: Intersection elimination (left)".to_string(),
                    theorem_name: "inter_left".to_string(),
                    doc: "If x ∈ A ∩ B, then x ∈ A.".to_string(),
                    hints: s_vec(&["`intro h` then `rcases h with ⟨ha, hb⟩` then `exact ha`."]),
                    context_vars: vec![],
                    initial_hypotheses: vec![],
                    target_str: "A ∧ B → A".to_string(),
                    available_tactics: s_vec(&["intro", "rcases", "exact"]),
                    reference_solution: "intro h\nrcases h with ⟨ha, hb⟩\nexact ha".to_string(),
                },
                Level {
                    id: "inter_right_level".to_string(),
                    world_id: "boolean_world".to_string(),
                    number: 2,
                    title: "Level 2: Intersection elimination (right)".to_string(),
                    theorem_name: "inter_right".to_string(),
                    doc: "If x ∈ A ∩ B, then x ∈ B.".to_string(),
                    hints: s_vec(&["`intro h` then `rcases h with ⟨ha, hb⟩` then `exact hb`."]),
                    context_vars: vec![],
                    initial_hypotheses: vec![],
                    target_str: "A ∧ B → B".to_string(),
                    available_tactics: s_vec(&["intro", "rcases", "exact"]),
                    reference_solution: "intro h\nrcases h with ⟨ha, hb⟩\nexact hb".to_string(),
                },
                Level {
                    id: "inter_comm_level".to_string(),
                    world_id: "boolean_world".to_string(),
                    number: 3,
                    title: "Level 3: Intersection commutativity".to_string(),
                    theorem_name: "inter_comm".to_string(),
                    doc: "Prove A ∩ B ⊆ B ∩ A using `constructor` to assemble the two halves.".to_string(),
                    hints: s_vec(&["`rcases h with ⟨ha, hb⟩` then `constructor`."]),
                    context_vars: vec![],
                    initial_hypotheses: vec![],
                    target_str: "A ∧ B → B ∧ A".to_string(),
                    available_tactics: s_vec(&["intro", "rcases", "constructor", "exact"]),
                    reference_solution: "intro h\nrcases h with ⟨ha, hb⟩\nconstructor\n· exact hb\n· exact ha".to_string(),
                },
            ],
        }
    }
}
