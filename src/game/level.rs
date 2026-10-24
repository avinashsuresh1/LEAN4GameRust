use crate::engine::environment::Theorem;
use crate::engine::parser::Parser;
use crate::engine::proof_state::{Hypothesis, ProofState};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Level {
    pub id: String,
    pub world_id: String,
    pub number: usize,
    pub title: String,
    pub theorem_name: String,
    pub doc: String,
    pub hints: Vec<String>,
    pub context_vars: Vec<(String, String)>,
    pub initial_hypotheses: Vec<Hypothesis>,
    pub target_str: String,
    pub available_tactics: Vec<String>,
    pub reference_solution: String,
}

impl Level {
    pub fn build_initial_state(&self) -> ProofState {
        let mut hypotheses = Vec::new();

        // Variables: e.g. a b c : ℕ
        for (name, typ) in &self.context_vars {
            hypotheses.push(Hypothesis::var(name.clone(), typ.clone()));
        }

        // Additional hypotheses
        hypotheses.extend(self.initial_hypotheses.clone());

        let target = Parser::parse_expr_str(&self.target_str)
            .unwrap_or_else(|e| panic!("Invalid target '{}' in level {}: {}", self.target_str, self.id, e));

        ProofState::new(hypotheses, target)
    }

    pub fn to_theorem(&self) -> Theorem {
        let full_statement = if self.context_vars.is_empty() {
            self.target_str.clone()
        } else {
            let vars_str = self
                .context_vars
                .iter()
                .map(|(v, t)| format!("({} : {})", v, t))
                .collect::<Vec<_>>()
                .join(" ");
            format!("∀ {}, {}", vars_str, self.target_str)
        };

        Theorem::new(
            self.theorem_name.clone(),
            format!("Proved in {}: {}", self.title, self.doc.lines().next().unwrap_or("")),
            &full_statement,
            false,
        )
    }
}
