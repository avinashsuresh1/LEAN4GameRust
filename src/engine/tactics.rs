use crate::engine::environment::Environment;
use crate::engine::expr::Expr;
use crate::engine::parser::{RwStep, Tactic};
use crate::engine::proof_state::{Goal, Hypothesis, ProofState};
use std::collections::{HashMap, HashSet};

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum TacticResult {
    Success,
    ProofComplete,
    Error(String),
}

pub struct TacticEngine;

impl TacticEngine {
    /// Applies a parsed tactic to the current proof state.
    pub fn apply_tactic(
        tactic: &Tactic,
        state: &mut ProofState,
        env: &Environment,
    ) -> Result<String, String> {
        if state.is_solved() {
            return Err("Proof is already complete! No more goals.".to_string());
        }

        match tactic {
            Tactic::Bullet => {
                // Bullet point is syntactical grouping in Lean 4; passes as a no-op check
                Ok("Focus on current subgoal".to_string())
            }
            Tactic::Rfl => Self::apply_rfl(state),
            Tactic::Rw { lemmas, at_hyp } => Self::apply_rw(lemmas, at_hyp.as_deref(), state, env),
            Tactic::Induction {
                var,
                step_var,
                step_hyp,
                ..
            } => Self::apply_induction(var, step_var, step_hyp, state),
            Tactic::Exact(term) => Self::apply_exact(term, state, env),
            Tactic::Intro(names) => Self::apply_intro(names, state),
            Tactic::Apply(name) => Self::apply_apply(name, state, env),
            Tactic::Symm => Self::apply_symm(state),
            Tactic::Sorry => Self::apply_sorry(state),
            Tactic::Repeat(inner) => {
                let mut count = 0;
                while count < 50 {
                    let mut temp_state = state.clone();
                    if let Ok(_) = Self::apply_tactic(inner, &mut temp_state, env) {
                        *state = temp_state;
                        count += 1;
                        if state.is_solved() {
                            break;
                        }
                    } else {
                        break;
                    }
                }
                if count > 0 {
                    Ok(format!("Repeated tactic {} times", count))
                } else {
                    Err("Repeat made no progress".to_string())
                }
            }
            Tactic::Have { name, prop } => Self::apply_have(name, prop, state),
            Tactic::Cases { var, names } => Self::apply_cases(var, names, state),
            Tactic::Use(witness) => Self::apply_use(witness, state),
            Tactic::Constructor => Self::apply_constructor(state),
            Tactic::Rcases { hyp, names } => Self::apply_rcases(hyp, names, state),
            Tactic::Dsimp => Self::apply_dsimp(state),
            Tactic::Linarith => Self::apply_linarith(state),
        }
    }

    fn apply_rfl(state: &mut ProofState) -> Result<String, String> {
        let active = state
            .active_goal()
            .ok_or_else(|| "No active goal".to_string())?;

        match &active.target {
            Expr::Eq(lhs, rhs) => {
                if lhs == rhs || Self::are_definitionally_equal(lhs, rhs) {
                    state.goals.remove(0);
                    Ok("Goal solved by reflexivity".to_string())
                } else {
                    Err(format!(
                        "rfl failed: LHS '{}' is not syntactically equal to RHS '{}'",
                        lhs, rhs
                    ))
                }
            }
            Expr::Le(lhs, rhs) if lhs == rhs => {
                state.goals.remove(0);
                Ok("Goal solved by reflexivity of ≤".to_string())
            }
            _ => Err(format!(
                "rfl failed: target '{}' is not an equality",
                active.target
            )),
        }
    }

    fn are_definitionally_equal(e1: &Expr, e2: &Expr) -> bool {
        // Evaluate simple numerals: e.g. 1 == succ 0, 2 == succ 1
        fn normalize_numeral(e: &Expr) -> Option<u64> {
            match e {
                Expr::Nat(n) => Some(*n),
                Expr::Succ(inner) => normalize_numeral(inner).map(|n| n + 1),
                _ => None,
            }
        }

        if let (Some(n1), Some(n2)) = (normalize_numeral(e1), normalize_numeral(e2)) {
            return n1 == n2;
        }

        false
    }

    fn apply_rw(
        steps: &[RwStep],
        at_hyp: Option<&str>,
        state: &mut ProofState,
        env: &Environment,
    ) -> Result<String, String> {
        for step in steps {
            let active = state
                .active_goal_mut()
                .ok_or_else(|| "No active goal".to_string())?;

            let rule = env
                .get_equality_rule(&step.lemma_name, &active.hypotheses)
                .ok_or_else(|| {
                    format!(
                        "Unknown theorem or hypothesis '{}' for rw",
                        step.lemma_name
                    )
                })?;

            let (lhs, rhs, pattern_vars) = rule;

            if let Some(target_hyp_name) = at_hyp {
                // Rewrite in hypothesis
                let hyp = active
                    .hypotheses
                    .iter_mut()
                    .find(|h| h.name == target_hyp_name)
                    .ok_or_else(|| format!("Hypothesis '{}' not found", target_hyp_name))?;

                let rewritten = hyp
                    .typ
                    .rewrite_once(&lhs, &rhs, &pattern_vars, step.reverse)
                    .ok_or_else(|| {
                        format!(
                            "rw [{}]: pattern not found in hypothesis '{} : {}'",
                            if step.reverse {
                                format!("← {}", step.lemma_name)
                            } else {
                                step.lemma_name.clone()
                            },
                            target_hyp_name,
                            hyp.typ
                        )
                    })?;

                hyp.typ = rewritten;
            } else {
                // Rewrite in target goal
                let new_target = active
                    .target
                    .rewrite_once(&lhs, &rhs, &pattern_vars, step.reverse)
                    .ok_or_else(|| {
                        format!(
                            "rw [{}]: did not find any subterm matching pattern in target '⊢ {}'",
                            if step.reverse {
                                format!("← {}", step.lemma_name)
                            } else {
                                step.lemma_name.clone()
                            },
                            active.target
                        )
                    })?;

                active.target = new_target;

                // In Lean 4 Natural Number Game, `rw` automatically attempts `rfl`
                if let Expr::Eq(l, r) = &active.target {
                    if l == r || Self::are_definitionally_equal(l, r) {
                        state.goals.remove(0);
                    }
                }
            }
        }

        Ok("Rewritten successfully".to_string())
    }

    fn apply_induction(
        var: &str,
        step_var: &str,
        step_hyp: &str,
        state: &mut ProofState,
    ) -> Result<String, String> {
        let active = state
            .active_goal()
            .ok_or_else(|| "No active goal".to_string())?;

        // Check that variable exists in context
        let var_exists = active.hypotheses.iter().any(|h| h.name == var && h.is_var);
        if !var_exists {
            return Err(format!(
                "Variable '{}' not found in hypotheses for induction",
                var
            ));
        }

        let original_target = active.target.clone();
        let original_hypotheses = active.hypotheses.clone();

        // 1. Base case: var = 0
        let base_hypotheses: Vec<Hypothesis> = original_hypotheses
            .iter()
            .filter(|h| h.name != var)
            .map(|h| Hypothesis {
                name: h.name.clone(),
                typ: h.typ.substitute(var, &Expr::zero()),
                is_var: h.is_var,
            })
            .collect();
        let base_target = original_target.substitute(var, &Expr::zero());
        let base_id = state.alloc_goal_id();
        let base_goal = Goal::new(base_id, base_hypotheses, base_target).with_tag("zero");

        // 2. Inductive step: var = succ(step_var), with induction hypothesis
        let mut step_hypotheses: Vec<Hypothesis> = original_hypotheses
            .iter()
            .filter(|h| h.name != var)
            .map(|h| Hypothesis {
                name: h.name.clone(),
                typ: h.typ.substitute(var, &Expr::var(step_var)),
                is_var: h.is_var,
            })
            .collect();

        // Add step variable `step_var : ℕ`
        step_hypotheses.push(Hypothesis::var(step_var, "ℕ"));

        // Add induction hypothesis: original target evaluated at `step_var`
        let ind_hyp_typ = original_target.substitute(var, &Expr::var(step_var));
        step_hypotheses.push(Hypothesis::prop(step_hyp, ind_hyp_typ));

        // Goal target: original target evaluated at `succ(step_var)`
        let step_target = original_target.substitute(var, &Expr::succ(Expr::var(step_var)));
        let step_id = state.alloc_goal_id();
        let step_goal =
            Goal::new(step_id, step_hypotheses, step_target).with_tag(format!("succ {}", step_var));

        // Replace active goal with the two induction subgoals
        state.goals.remove(0);
        state.goals.insert(0, step_goal);
        state.goals.insert(0, base_goal);

        Ok(format!(
            "Induction on '{}' generated 2 subgoals: base case (0) and inductive step (succ {})",
            var, step_var
        ))
    }

    fn apply_exact(term_str: &str, state: &mut ProofState, env: &Environment) -> Result<String, String> {
        let active = state
            .active_goal()
            .ok_or_else(|| "No active goal".to_string())?;

        // 1. Check if term is a hypothesis name
        if let Some(hyp) = active.hypotheses.iter().find(|h| h.name == term_str) {
            if hyp.typ == active.target || hyp.typ.is_equivalent(&active.target) {
                state.goals.remove(0);
                return Ok(format!("Goal closed by hypothesis '{}'", term_str));
            }
        }

        // 2. Check if term matches theorem in environment
        if let Some(thm) = env.get(term_str) {
            let (vars, stripped) = thm.statement.peel_forall();
            let var_set: HashSet<String> = vars.into_iter().map(|(v, _)| v).collect();
            let mut bindings = HashMap::new();
            if active.target.match_pattern(stripped, &var_set, &mut bindings) {
                state.goals.remove(0);
                return Ok(format!("Goal closed by theorem '{}'", term_str));
            }
        }

        // 3. Check parsed expression
        if let Ok(expr) = crate::engine::parser::Parser::parse_expr_str(term_str) {
            if expr == active.target || expr.is_equivalent(&active.target) {
                state.goals.remove(0);
                return Ok("Goal closed by exact term".to_string());
            }
        }

        Err(format!(
            "exact '{}' does not match current target '⊢ {}'",
            term_str, active.target
        ))
    }

    fn apply_intro(names: &[String], state: &mut ProofState) -> Result<String, String> {
        let active = state
            .active_goal_mut()
            .ok_or_else(|| "No active goal".to_string())?;

        for name in names {
            match active.target.clone() {
                Expr::Arrow(lhs, rhs) => {
                    active.hypotheses.push(Hypothesis::prop(name, *lhs));
                    active.target = *rhs;
                }
                Expr::Forall(mut vars, body) => {
                    if let Some((v, typ)) = vars.first().cloned() {
                        vars.remove(0);
                        active.hypotheses.push(Hypothesis::var(name, &typ));
                        let substituted_body = body.substitute(&v, &Expr::var(name));
                        if vars.is_empty() {
                            active.target = substituted_body;
                        } else {
                            active.target = Expr::Forall(vars, Box::new(substituted_body));
                        }
                    } else {
                        return Err("Empty forall quantifier".to_string());
                    }
                }
                _ => {
                    return Err(format!(
                        "Cannot apply intro: target '{}' is not an implication or universal quantifier",
                        active.target
                    ));
                }
            }
        }

        Ok(format!("Introduced {}", names.join(", ")))
    }

    fn apply_apply(name: &str, state: &mut ProofState, env: &Environment) -> Result<String, String> {
        let active = state
            .active_goal()
            .ok_or_else(|| "No active goal".to_string())?;

        let rule_expr = if let Some(h) = active.hypotheses.iter().find(|h| h.name == name) {
            h.typ.clone()
        } else if let Some(thm) = env.get(name) {
            thm.statement.clone()
        } else {
            return Err(format!("Unknown hypothesis or theorem '{}' for apply", name));
        };

        let (vars, stripped) = rule_expr.peel_forall();
        let var_set: HashSet<String> = vars.into_iter().map(|(v, _)| v).collect();

        let mut premises = Vec::new();
        let mut curr = stripped;
        while let Expr::Arrow(p, c) = curr {
            premises.push(p.as_ref());
            curr = c.as_ref();
        }
        let conclusion = curr;

        let mut bindings = HashMap::new();
        if active.target.match_pattern(conclusion, &var_set, &mut bindings) {
            let active_hyps = active.hypotheses.clone();
            state.goals.remove(0);

            for (idx, premise) in premises.iter().enumerate().rev() {
                let instantiated_premise = premise.substitute_map(&bindings);
                let goal_id = state.alloc_goal_id();
                let goal = Goal::new(goal_id, active_hyps.clone(), instantiated_premise)
                    .with_tag(format!("{}_{}", name, idx + 1));
                state.goals.insert(0, goal);
            }
            return Ok(format!(
                "Applied '{}', generated {} subgoals",
                name,
                premises.len()
            ));
        }

        Err(format!(
            "Cannot apply '{}': conclusion does not match target '⊢ {}'",
            name, active.target
        ))
    }

    fn apply_symm(state: &mut ProofState) -> Result<String, String> {
        let active = state
            .active_goal_mut()
            .ok_or_else(|| "No active goal".to_string())?;

        match &active.target {
            Expr::Eq(l, r) => {
                active.target = Expr::eq(*r.clone(), *l.clone());
                Ok("Symmetry applied to equality".to_string())
            }
            Expr::Ne(l, r) => {
                active.target = Expr::ne(*r.clone(), *l.clone());
                Ok("Symmetry applied to inequality".to_string())
            }
            _ => Err(format!(
                "symm failed: target '{}' is not an equality or inequality",
                active.target
            )),
        }
    }

    fn apply_have(name: &str, prop: &Expr, state: &mut ProofState) -> Result<String, String> {
        let active = state
            .active_goal()
            .ok_or_else(|| "No active goal".to_string())?;

        let active_hyps = active.hypotheses.clone();
        let main_target = active.target.clone();

        // 1. Goal to prove `prop`
        let have_id = state.alloc_goal_id();
        let have_goal = Goal::new(have_id, active_hyps.clone(), prop.clone())
            .with_tag(format!("have {}", name));

        // 2. Main goal with `name : prop` added to context
        let mut continuation_hyps = active_hyps;
        continuation_hyps.push(Hypothesis::prop(name, prop.clone()));
        let cont_id = state.alloc_goal_id();
        let cont_goal = Goal::new(cont_id, continuation_hyps, main_target);

        state.goals.remove(0);
        state.goals.insert(0, cont_goal);
        state.goals.insert(0, have_goal);

        Ok(format!("Asserted intermediate lemma '{} : {}'", name, prop))
    }

    fn apply_cases(var: &str, names: &[String], state: &mut ProofState) -> Result<String, String> {
        let active = state
            .active_goal()
            .ok_or_else(|| "No active goal".to_string())?;

        let var_exists = active.hypotheses.iter().any(|h| h.name == var && h.is_var);
        if !var_exists {
            return Err(format!("Variable '{}' not found for cases", var));
        }

        let d_name = names.first().map(|s| s.as_str()).unwrap_or("d");
        let orig_hyps = active.hypotheses.clone();
        let orig_target = active.target.clone();

        // Case 1: var = 0
        let c0_hyps: Vec<Hypothesis> = orig_hyps
            .iter()
            .filter(|h| h.name != var)
            .map(|h| Hypothesis {
                name: h.name.clone(),
                typ: h.typ.substitute(var, &Expr::zero()),
                is_var: h.is_var,
            })
            .collect();
        let c0_target = orig_target.substitute(var, &Expr::zero());
        let c0_id = state.alloc_goal_id();
        let c0_goal = Goal::new(c0_id, c0_hyps, c0_target).with_tag("zero");

        // Case 2: var = succ d
        let mut csucc_hyps: Vec<Hypothesis> = orig_hyps
            .iter()
            .filter(|h| h.name != var)
            .map(|h| Hypothesis {
                name: h.name.clone(),
                typ: h.typ.substitute(var, &Expr::succ(Expr::var(d_name))),
                is_var: h.is_var,
            })
            .collect();
        csucc_hyps.push(Hypothesis::var(d_name, "ℕ"));
        let csucc_target = orig_target.substitute(var, &Expr::succ(Expr::var(d_name)));
        let csucc_id = state.alloc_goal_id();
        let csucc_goal =
            Goal::new(csucc_id, csucc_hyps, csucc_target).with_tag(format!("succ {}", d_name));

        state.goals.remove(0);
        state.goals.insert(0, csucc_goal);
        state.goals.insert(0, c0_goal);

        Ok(format!("Split cases on '{}'", var))
    }

    fn apply_sorry(state: &mut ProofState) -> Result<String, String> {
        if state.goals.is_empty() {
            return Err("No goals to sorry".to_string());
        }
        state.goals.remove(0);
        Ok("Goal skipped with sorry (incomplete proof)".to_string())
    }

    fn apply_use(witness_str: &str, state: &mut ProofState) -> Result<String, String> {
        let active = state
            .active_goal_mut()
            .ok_or_else(|| "No active goal".to_string())?;
        let witness = crate::engine::parser::Parser::parse_expr_str(witness_str)?;

        match active.target.clone() {
            Expr::Exists(mut vars, body) => {
                if let Some((v, _typ)) = vars.first().cloned() {
                    vars.remove(0);
                    let substituted = body.substitute(&v, &witness);
                    if vars.is_empty() {
                        active.target = substituted;
                    } else {
                        active.target = Expr::Exists(vars, Box::new(substituted));
                    }
                    Ok(format!("Instantiated ∃ with witness '{}'", witness))
                } else {
                    Err("Empty existential quantifier".to_string())
                }
            }
            _ => Err(format!(
                "Cannot apply 'use': target '⊢ {}' is not an existential quantifier (∃)",
                active.target
            )),
        }
    }

    fn apply_constructor(state: &mut ProofState) -> Result<String, String> {
        let (p, q, hyps) = {
            let active = state
                .active_goal()
                .ok_or_else(|| "No active goal".to_string())?;

            match &active.target {
                Expr::And(p, q) => (*p.clone(), *q.clone(), active.hypotheses.clone()),
                _ => {
                    return Err(format!(
                        "Cannot apply 'constructor': target '⊢ {}' is not a conjunction (∧)",
                        active.target
                    ))
                }
            }
        };

        let g1_id = state.alloc_goal_id();
        let g2_id = state.alloc_goal_id();
        let g1 = Goal::new(g1_id, hyps.clone(), p).with_tag("left");
        let g2 = Goal::new(g2_id, hyps, q).with_tag("right");

        state.goals.remove(0);
        state.goals.insert(0, g2);
        state.goals.insert(0, g1);
        Ok("Split conjunction (∧) into 2 subgoals".to_string())
    }

    fn apply_rcases(
        hyp_name: &str,
        names: &[String],
        state: &mut ProofState,
    ) -> Result<String, String> {
        let active = state
            .active_goal_mut()
            .ok_or_else(|| "No active goal".to_string())?;
        let pos = active
            .hypotheses
            .iter()
            .position(|h| h.name == hyp_name)
            .ok_or_else(|| format!("Hypothesis '{}' not found for rcases", hyp_name))?;

        let hyp = active.hypotheses.remove(pos);
        match hyp.typ {
            Expr::Exists(vars, body) => {
                let var_name = names.first().map(|s| s.as_str()).unwrap_or("N");
                let prop_name = names.get(1).map(|s| s.as_str()).unwrap_or("hN");
                let (first_v, typ) = vars
                    .first()
                    .cloned()
                    .unwrap_or(("N".to_string(), "ℕ".to_string()));

                active.hypotheses.push(Hypothesis::var(var_name, &typ));
                let instantiated_body = body.substitute(&first_v, &Expr::var(var_name));
                active.hypotheses.push(Hypothesis::prop(prop_name, instantiated_body));
                Ok(format!(
                    "Unpacked ∃ into variable '{}' and hypothesis '{}'",
                    var_name, prop_name
                ))
            }
            Expr::And(p, q) => {
                let n1 = names.first().map(|s| s.as_str()).unwrap_or("h1");
                let n2 = names.get(1).map(|s| s.as_str()).unwrap_or("h2");
                active.hypotheses.push(Hypothesis::prop(n1, *p));
                active.hypotheses.push(Hypothesis::prop(n2, *q));
                Ok(format!("Unpacked ∧ into hypotheses '{}' and '{}'", n1, n2))
            }
            _ => Err(format!(
                "Hypothesis '{}' is neither an existential nor a conjunction",
                hyp_name
            )),
        }
    }

    fn apply_dsimp(state: &mut ProofState) -> Result<String, String> {
        let _ = state.active_goal().ok_or_else(|| "No active goal".to_string())?;
        Ok("Simplified definitions".to_string())
    }

    fn apply_linarith(state: &mut ProofState) -> Result<String, String> {
        let active = state
            .active_goal()
            .ok_or_else(|| "No active goal".to_string())?;

        match &active.target {
            Expr::Le(l, r) | Expr::Ge(l, r) | Expr::Eq(l, r) if l == r => {
                state.goals.remove(0);
                Ok("Goal solved by linear arithmetic".to_string())
            }
            _ => {
                for h in &active.hypotheses {
                    if h.typ == active.target || h.typ.is_equivalent(&active.target) {
                        state.goals.remove(0);
                        return Ok("Goal solved by linear arithmetic (hypothesis match)".to_string());
                    }
                }
                Err(format!("linarith failed to close goal '⊢ {}'", active.target))
            }
        }
    }
}
