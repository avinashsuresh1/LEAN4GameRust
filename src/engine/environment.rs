use crate::engine::expr::Expr;
use crate::engine::parser::Parser;
use serde::{Deserialize, Serialize};
use std::collections::{HashMap, HashSet};

/// An axiom or proved theorem available for rewriting or exact matching.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Theorem {
    pub name: String,
    pub doc: String,
    pub statement: Expr,
    pub is_axiom: bool,
}

impl Theorem {
    pub fn new(name: impl Into<String>, doc: impl Into<String>, statement_str: &str, is_axiom: bool) -> Self {
        let statement = Parser::parse_expr_str(statement_str).expect("Valid theorem statement syntax");
        Self {
            name: name.into(),
            doc: doc.into(),
            statement,
            is_axiom,
        }
    }
}

/// The mathematical environment containing axioms and player-unlocked theorems.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Environment {
    pub theorems: HashMap<String, Theorem>,
}

impl Default for Environment {
    fn default() -> Self {
        Self::standard_peano()
    }
}

impl Environment {
    pub fn new() -> Self {
        Self {
            theorems: HashMap::new(),
        }
    }

    /// Creates the standard Peano arithmetic axiom environment for the Natural Number Game.
    pub fn standard_peano() -> Self {
        let mut env = Self::new();

        // Standard Peano axioms
        env.add_theorem(Theorem::new(
            "add_zero",
            "Axiom of addition with zero: ∀ (n : ℕ), n + 0 = n",
            "∀ (n : ℕ), n + 0 = n",
            true,
        ));

        env.add_theorem(Theorem::new(
            "add_succ",
            "Axiom of addition with successor: ∀ (n m : ℕ), n + succ m = succ (n + m)",
            "∀ (n m : ℕ), n + succ m = succ (n + m)",
            true,
        ));

        env.add_theorem(Theorem::new(
            "mul_zero",
            "Axiom of multiplication with zero: ∀ (n : ℕ), n * 0 = 0",
            "∀ (n : ℕ), n * 0 = 0",
            true,
        ));

        env.add_theorem(Theorem::new(
            "mul_succ",
            "Axiom of multiplication with successor: ∀ (n m : ℕ), n * succ m = n * m + n",
            "∀ (n m : ℕ), n * succ m = n * m + n",
            true,
        ));

        env.add_theorem(Theorem::new(
            "pow_zero",
            "Axiom of power with zero: ∀ (a : ℕ), a ^ 0 = 1",
            "∀ (a : ℕ), a ^ 0 = 1",
            true,
        ));

        env.add_theorem(Theorem::new(
            "pow_succ",
            "Axiom of power with successor: ∀ (a m : ℕ), a ^ succ m = a ^ m * a",
            "∀ (a m : ℕ), a ^ succ m = a ^ m * a",
            true,
        ));

        env.add_theorem(Theorem::new(
            "one_eq_succ_zero",
            "Definition of numeral 1: 1 = succ 0",
            "1 = succ 0",
            true,
        ));

        env.add_theorem(Theorem::new(
            "two_eq_succ_one",
            "Definition of numeral 2: 2 = succ 1",
            "2 = succ 1",
            true,
        ));

        env.add_theorem(Theorem::new(
            "succ_inj",
            "Successor is injective: ∀ (n m : ℕ), succ n = succ m → n = m",
            "∀ (n m : ℕ), succ n = succ m → n = m",
            true,
        ));

        env.add_theorem(Theorem::new(
            "zero_ne_succ",
            "Zero is not a successor: ∀ (n : ℕ), 0 ≠ succ n",
            "∀ (n : ℕ), 0 ≠ succ n",
            true,
        ));

        env.add_theorem(Theorem::new(
            "le_refl",
            "Reflexivity of less than or equal to: ∀ (x : ℕ), x ≤ x",
            "∀ (x : ℕ), x ≤ x",
            true,
        ));

        env.add_theorem(Theorem::new(
            "le_succ",
            "Every number is less than or equal to its successor: ∀ (x : ℕ), x ≤ succ x",
            "∀ (x : ℕ), x ≤ succ x",
            true,
        ));

        env.add_theorem(Theorem::new(
            "zero_le",
            "Zero is less than or equal to every number: ∀ (x : ℕ), 0 ≤ x",
            "∀ (x : ℕ), 0 ≤ x",
            true,
        ));

        env
    }

    /// Real Analysis axioms: Ordered Field and Metric Space of ℝ
    pub fn standard_real_analysis() -> Self {
        let mut env = Self::new();

        env.add_theorem(Theorem::new(
            "sub_self_real",
            "Self subtraction is zero: ∀ (x : ℝ), x - x = 0",
            "∀ (x : ℝ), x - x = 0",
            true,
        ));

        env.add_theorem(Theorem::new(
            "add_zero_real",
            "Additive identity of real numbers: ∀ (x : ℝ), x + 0 = x",
            "∀ (x : ℝ), x + 0 = x",
            true,
        ));

        env.add_theorem(Theorem::new(
            "abs_zero",
            "Absolute value of zero is zero: |0| = 0",
            "|0| = 0",
            true,
        ));

        env.add_theorem(Theorem::new(
            "abs_sub_comm",
            "Symmetry of distance: ∀ (a b : ℝ), |a - b| = |b - a|",
            "∀ (a b : ℝ), |a - b| = |b - a|",
            true,
        ));

        env.add_theorem(Theorem::new(
            "triangle_ineq",
            "Triangle inequality: ∀ (a b : ℝ), |a + b| ≤ |a| + |b|",
            "∀ (a b : ℝ), |a + b| ≤ |a| + |b|",
            true,
        ));

        env.add_theorem(Theorem::new(
            "triangle_sub",
            "Triangle inequality for differences: ∀ (a b c : ℝ), |a - c| ≤ |a - b| + |b - c|",
            "∀ (a b c : ℝ), |a - c| ≤ |a - b| + |b - c|",
            true,
        ));

        env.add_theorem(Theorem::new(
            "abs_nonneg",
            "Absolute value is non-negative: ∀ (x : ℝ), |x| ≥ 0",
            "∀ (x : ℝ), |x| ≥ 0",
            true,
        ));

        env.add_theorem(Theorem::new(
            "le_refl_real",
            "Reflexivity of ≤ on reals: ∀ (x : ℝ), x ≤ x",
            "∀ (x : ℝ), x ≤ x",
            true,
        ));

        env.add_theorem(Theorem::new(
            "le_trans_real",
            "Transitivity of ≤ on reals: ∀ (a b c : ℝ), a ≤ b → b ≤ c → a ≤ c",
            "∀ (a b c : ℝ), a ≤ b → b ≤ c → a ≤ c",
            true,
        ));

        env.add_theorem(Theorem::new(
            "half_pos",
            "Half of a positive number is positive: ∀ (ε : ℝ), ε > 0 → ε / 2 > 0",
            "∀ (ε : ℝ), ε > 0 → ε / 2 > 0",
            true,
        ));

        env.add_theorem(Theorem::new(
            "zero_lt_one",
            "Zero is strictly less than one: 0 < 1",
            "0 < 1",
            true,
        ));

        env
    }

    /// Combined environment containing both Peano Arithmetic and Real Analysis
    pub fn standard_all() -> Self {
        let mut env = Self::standard_peano();
        for (k, v) in Self::standard_real_analysis().theorems {
            env.theorems.insert(k, v);
        }
        env
    }

    /// Selects the appropriate standard axiom environment for a specific game
    pub fn standard_for_game(game_id: &str) -> Self {
        match game_id {
            "natural_numbers" => Self::standard_peano(),
            _ => Self::standard_all(),
        }
    }

    pub fn add_theorem(&mut self, thm: Theorem) {
        self.theorems.insert(thm.name.clone(), thm);
    }

    pub fn get(&self, name: &str) -> Option<&Theorem> {
        self.theorems.get(name)
    }

    /// Extract equality (lhs, rhs, quantified_variables) from theorem or hypothesis
    pub fn get_equality_rule(&self, name: &str, active_hypotheses: &[crate::engine::proof_state::Hypothesis]) -> Option<(Expr, Expr, HashSet<String>)> {
        // Check hypotheses first
        if let Some(h) = active_hypotheses.iter().find(|h| h.name == name) {
            let (vars, stripped) = h.typ.peel_forall();
            if let Some((lhs, rhs)) = stripped.as_eq() {
                let var_set: HashSet<String> = vars.into_iter().map(|(v, _)| v).collect();
                return Some((lhs.clone(), rhs.clone(), var_set));
            }
        }

        // Check global theorems / axioms
        if let Some(thm) = self.theorems.get(name) {
            let (vars, stripped) = thm.statement.peel_forall();
            if let Some((lhs, rhs)) = stripped.as_eq() {
                let var_set: HashSet<String> = vars.into_iter().map(|(v, _)| v).collect();
                return Some((lhs.clone(), rhs.clone(), var_set));
            }
        }

        None
    }
}
