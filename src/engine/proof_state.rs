use crate::engine::expr::Expr;
use serde::{Deserialize, Serialize};
use std::fmt;

/// A hypothesis in the local proof context.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Hypothesis {
    pub name: String,
    pub typ: Expr,
    pub is_var: bool, // true if it's a variable declaration like `n : ℕ`, false if proposition `h : a + b = c`
}

impl Hypothesis {
    pub fn var(name: impl Into<String>, typ: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            typ: Expr::Var(typ.into()),
            is_var: true,
        }
    }

    pub fn prop(name: impl Into<String>, prop: Expr) -> Self {
        Self {
            name: name.into(),
            typ: prop,
            is_var: false,
        }
    }
}

/// An individual goal to be solved.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Goal {
    pub id: usize,
    pub tag: Option<String>,
    pub hypotheses: Vec<Hypothesis>,
    pub target: Expr,
}

impl Goal {
    pub fn new(id: usize, hypotheses: Vec<Hypothesis>, target: Expr) -> Self {
        Self {
            id,
            tag: None,
            hypotheses,
            target,
        }
    }

    pub fn with_tag(mut self, tag: impl Into<String>) -> Self {
        self.tag = Some(tag.into());
        self
    }

    /// Find hypothesis by name.
    pub fn get_hyp(&self, name: &str) -> Option<&Hypothesis> {
        self.hypotheses.iter().find(|h| h.name == name)
    }

    /// Add or replace hypothesis.
    pub fn insert_hyp(&mut self, hyp: Hypothesis) {
        if let Some(pos) = self.hypotheses.iter().position(|h| h.name == hyp.name) {
            self.hypotheses[pos] = hyp;
        } else {
            self.hypotheses.push(hyp);
        }
    }
}

/// Complete proof state holding remaining goals.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ProofState {
    pub goals: Vec<Goal>,
    next_goal_id: usize,
}

impl ProofState {
    pub fn new(initial_hypotheses: Vec<Hypothesis>, initial_target: Expr) -> Self {
        let initial_goal = Goal::new(1, initial_hypotheses, initial_target);
        Self {
            goals: vec![initial_goal],
            next_goal_id: 2,
        }
    }

    pub fn alloc_goal_id(&mut self) -> usize {
        let id = self.next_goal_id;
        self.next_goal_id += 1;
        id
    }

    pub fn is_solved(&self) -> bool {
        self.goals.is_empty()
    }

    pub fn active_goal(&self) -> Option<&Goal> {
        self.goals.first()
    }

    pub fn active_goal_mut(&mut self) -> Option<&mut Goal> {
        self.goals.first_mut()
    }
}

impl fmt::Display for ProofState {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        if self.is_solved() {
            return write!(f, "🎉 Proof complete! All goals solved.");
        }

        let num_goals = self.goals.len();
        if num_goals == 1 {
            writeln!(f, "1 goal")?;
        } else {
            writeln!(f, "{} goals", num_goals)?;
        }

        if let Some(active) = self.active_goal() {
            if let Some(tag) = &active.tag {
                writeln!(f, "case {}", tag)?;
            }

            // Print variables grouped if possible
            let mut vars_by_type: std::collections::BTreeMap<String, Vec<String>> =
                std::collections::BTreeMap::new();
            let mut props = Vec::new();

            for h in &active.hypotheses {
                if h.is_var {
                    vars_by_type
                        .entry(format!("{}", h.typ))
                        .or_default()
                        .push(h.name.clone());
                } else {
                    props.push(h);
                }
            }

            for (typ, names) in vars_by_type {
                writeln!(f, "{} : {}", names.join(" "), typ)?;
            }

            for h in props {
                writeln!(f, "{} : {}", h.name, h.typ)?;
            }

            writeln!(f, "⊢ {}", active.target)?;
        }

        Ok(())
    }
}
