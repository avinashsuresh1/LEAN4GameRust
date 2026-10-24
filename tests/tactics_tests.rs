use lean4game::engine::environment::Environment;
use lean4game::engine::expr::Expr;
use lean4game::engine::parser::Parser;
use lean4game::engine::proof_state::{Hypothesis, ProofState};
use lean4game::engine::tactics::TacticEngine;

#[test]
fn test_tactic_rfl() {
    let env = Environment::standard_peano();
    let hyps = vec![Hypothesis::var("x", "ℕ")];
    let target = Expr::eq(Expr::var("x"), Expr::var("x"));
    let mut state = ProofState::new(hyps, target);

    let tactic = Parser::parse_tactic_line("rfl").unwrap().unwrap();
    let res = TacticEngine::apply_tactic(&tactic, &mut state, &env);
    assert!(res.is_ok());
    assert!(state.is_solved());
}

#[test]
fn test_tactic_rfl_numerals() {
    let env = Environment::standard_peano();
    let target = Expr::eq(Expr::Nat(2), Expr::succ(Expr::Nat(1)));
    let mut state = ProofState::new(vec![], target);

    let tactic = Parser::parse_tactic_line("rfl").unwrap().unwrap();
    let res = TacticEngine::apply_tactic(&tactic, &mut state, &env);
    assert!(res.is_ok());
    assert!(state.is_solved());
}

#[test]
fn test_tactic_rw_and_rfl_automatic_close() {
    let env = Environment::standard_peano();
    let hyps = vec![Hypothesis::var("x", "ℕ")];
    // Target: x + 0 = x
    let target = Expr::eq(Expr::add(Expr::var("x"), Expr::zero()), Expr::var("x"));
    let mut state = ProofState::new(hyps, target);

    let tactic = Parser::parse_tactic_line("rw [add_zero]").unwrap().unwrap();
    let res = TacticEngine::apply_tactic(&tactic, &mut state, &env);
    assert!(res.is_ok());
    // In Lean 4 NNG, rw automatically closes if goal reduces to X = X
    assert!(state.is_solved());
}

#[test]
fn test_tactic_rw_reverse() {
    let env = Environment::standard_peano();
    let hyps = vec![Hypothesis::var("a", "ℕ"), Hypothesis::var("b", "ℕ")];
    // Target: succ (a + b) = a + succ b
    let target = Expr::eq(
        Expr::succ(Expr::add(Expr::var("a"), Expr::var("b"))),
        Expr::add(Expr::var("a"), Expr::succ(Expr::var("b"))),
    );
    let mut state = ProofState::new(hyps, target);

    let tactic = Parser::parse_tactic_line("rw [← add_succ]").unwrap().unwrap();
    let res = TacticEngine::apply_tactic(&tactic, &mut state, &env);
    assert!(res.is_ok());
    assert!(state.is_solved());
}

#[test]
fn test_tactic_induction() {
    let env = Environment::standard_peano();
    let hyps = vec![Hypothesis::var("n", "ℕ")];
    // Target: 0 + n = n
    let target = Expr::eq(Expr::add(Expr::zero(), Expr::var("n")), Expr::var("n"));
    let mut state = ProofState::new(hyps, target);

    let tactic = Parser::parse_tactic_line("induction n with d hd").unwrap().unwrap();
    let res = TacticEngine::apply_tactic(&tactic, &mut state, &env);
    assert!(res.is_ok());
    assert_eq!(state.goals.len(), 2);

    // Goal 1: Base case (0)
    let g1 = &state.goals[0];
    assert_eq!(g1.tag.as_deref(), Some("zero"));
    assert_eq!(g1.target, Expr::eq(Expr::add(Expr::zero(), Expr::zero()), Expr::zero()));

    // Goal 2: Inductive step (succ d)
    let g2 = &state.goals[1];
    assert_eq!(g2.tag.as_deref(), Some("succ d"));
    assert!(g2.hypotheses.iter().any(|h| h.name == "d" && h.is_var));
    assert!(g2.hypotheses.iter().any(|h| h.name == "hd"
        && h.typ == Expr::eq(Expr::add(Expr::zero(), Expr::var("d")), Expr::var("d"))));
    assert_eq!(
        g2.target,
        Expr::eq(Expr::add(Expr::zero(), Expr::succ(Expr::var("d"))), Expr::succ(Expr::var("d")))
    );
}

#[test]
fn test_tactic_intro_and_exact() {
    let env = Environment::standard_peano();
    let target = Expr::arrow(Expr::Prop("P".to_string()), Expr::Prop("P".to_string()));
    let mut state = ProofState::new(vec![], target);

    let intro_tac = Parser::parse_tactic_line("intro h").unwrap().unwrap();
    let res_intro = TacticEngine::apply_tactic(&intro_tac, &mut state, &env);
    assert!(res_intro.is_ok());
    assert_eq!(state.goals.len(), 1);
    assert_eq!(state.goals[0].hypotheses.len(), 1);
    assert_eq!(state.goals[0].hypotheses[0].name, "h");

    let exact_tac = Parser::parse_tactic_line("exact h").unwrap().unwrap();
    let res_exact = TacticEngine::apply_tactic(&exact_tac, &mut state, &env);
    assert!(res_exact.is_ok());
    assert!(state.is_solved());
}

#[test]
fn test_tactic_apply() {
    let env = Environment::standard_peano();
    let target = Parser::parse_expr_str("(P → Q) → P → Q").unwrap();
    let mut state = ProofState::new(vec![], target);

    let intro = Parser::parse_tactic_line("intro f p").unwrap().unwrap();
    TacticEngine::apply_tactic(&intro, &mut state, &env).unwrap();

    let apply = Parser::parse_tactic_line("apply f").unwrap().unwrap();
    TacticEngine::apply_tactic(&apply, &mut state, &env).unwrap();

    assert_eq!(state.goals[0].target, Expr::var("P"));

    let exact = Parser::parse_tactic_line("exact p").unwrap().unwrap();
    TacticEngine::apply_tactic(&exact, &mut state, &env).unwrap();

    assert!(state.is_solved());
}
