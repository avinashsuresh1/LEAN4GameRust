use lean4game::engine::expr::Expr;
use lean4game::engine::parser::{Parser, Tactic};
use std::collections::{HashMap, HashSet};

#[test]
fn test_expr_construction_and_display() {
    let e = Expr::add(Expr::var("a"), Expr::succ(Expr::zero()));
    assert_eq!(format!("{}", e), "a + succ 0");

    let e_mul = Expr::mul(Expr::var("x"), Expr::var("y"));
    let e_add = Expr::add(e_mul, Expr::var("z"));
    assert_eq!(format!("{}", e_add), "x * y + z");

    let e_pow = Expr::pow(Expr::var("a"), Expr::var("b"));
    assert_eq!(format!("{}", e_pow), "a ^ b");

    let e_arrow = Expr::arrow(Expr::var("P"), Expr::var("Q"));
    assert_eq!(format!("{}", e_arrow), "P → Q");

    let e_forall = Expr::forall(vec![("n", "ℕ")], Expr::eq(Expr::add(Expr::var("n"), Expr::zero()), Expr::var("n")));
    assert_eq!(format!("{}", e_forall), "∀ (n : ℕ), n + 0 = n");
}

#[test]
fn test_expr_free_vars() {
    let e = Expr::add(Expr::var("a"), Expr::succ(Expr::var("b")));
    let free = e.free_vars();
    assert!(free.contains("a"));
    assert!(free.contains("b"));
    assert_eq!(free.len(), 2);

    let forall_e = Expr::forall(vec![("a", "ℕ")], e);
    let free2 = forall_e.free_vars();
    assert!(!free2.contains("a"));
    assert!(free2.contains("b"));
}

#[test]
fn test_expr_substitution() {
    let e = Expr::add(Expr::var("x"), Expr::succ(Expr::var("x")));
    let substituted = e.substitute("x", &Expr::Nat(5));
    assert_eq!(
        substituted,
        Expr::add(Expr::Nat(5), Expr::succ(Expr::Nat(5)))
    );
}

#[test]
fn test_expr_pattern_matching() {
    // Pattern: ?n + 0
    let pat = Expr::add(Expr::var("n"), Expr::zero());
    let mut pat_vars = HashSet::new();
    pat_vars.insert("n".to_string());

    // Target: (a + b) + 0
    let target = Expr::add(Expr::add(Expr::var("a"), Expr::var("b")), Expr::zero());

    let mut bindings = HashMap::new();
    let matched = target.match_pattern(&pat, &pat_vars, &mut bindings);
    assert!(matched);
    assert_eq!(
        bindings.get("n"),
        Some(&Expr::add(Expr::var("a"), Expr::var("b")))
    );
}

#[test]
fn test_expr_rewriting() {
    // Rule: n + 0 = n
    let lhs = Expr::add(Expr::var("n"), Expr::zero());
    let rhs = Expr::var("n");
    let mut pat_vars = HashSet::new();
    pat_vars.insert("n".to_string());

    // Target: succ (x + 0)
    let target = Expr::succ(Expr::add(Expr::var("x"), Expr::zero()));
    let rewritten = target.rewrite_once(&lhs, &rhs, &pat_vars, false);
    assert_eq!(rewritten, Some(Expr::succ(Expr::var("x"))));

    // Reverse rewrite: from n to n + 0
    let rev_target = Expr::var("x");
    let rev_rewritten = rev_target.rewrite_once(&lhs, &rhs, &pat_vars, true);
    assert_eq!(
        rev_rewritten,
        Some(Expr::add(Expr::var("x"), Expr::zero()))
    );
}

#[test]
fn test_parser_expressions() {
    let expr1 = Parser::parse_expr_str("a + b * c").unwrap();
    // Multiplication binds tighter than addition
    assert_eq!(
        expr1,
        Expr::add(Expr::var("a"), Expr::mul(Expr::var("b"), Expr::var("c")))
    );

    let expr2 = Parser::parse_expr_str("(a + b) * c").unwrap();
    assert_eq!(
        expr2,
        Expr::mul(Expr::add(Expr::var("a"), Expr::var("b")), Expr::var("c"))
    );

    let expr3 = Parser::parse_expr_str("a ^ b ^ c").unwrap();
    // Exponentiation is right-associative
    assert_eq!(
        expr3,
        Expr::pow(Expr::var("a"), Expr::pow(Expr::var("b"), Expr::var("c")))
    );

    let eq_expr = Parser::parse_expr_str("a + 0 = a").unwrap();
    assert_eq!(
        eq_expr,
        Expr::eq(Expr::add(Expr::var("a"), Expr::zero()), Expr::var("a"))
    );
}

#[test]
fn test_parser_tactics() {
    assert_eq!(Parser::parse_tactic_line("rfl").unwrap(), Some(Tactic::Rfl));
    assert_eq!(Parser::parse_tactic_line("sorry").unwrap(), Some(Tactic::Sorry));
    assert_eq!(Parser::parse_tactic_line("symm").unwrap(), Some(Tactic::Symm));

    let rw = Parser::parse_tactic_line("rw [add_zero, ← add_succ]").unwrap();
    match rw {
        Some(Tactic::Rw { lemmas, at_hyp }) => {
            assert_eq!(lemmas.len(), 2);
            assert_eq!(lemmas[0].lemma_name, "add_zero");
            assert!(!lemmas[0].reverse);
            assert_eq!(lemmas[1].lemma_name, "add_succ");
            assert!(lemmas[1].reverse);
            assert_eq!(at_hyp, None);
        }
        _ => panic!("Expected Rw tactic"),
    }

    let rw_at = Parser::parse_tactic_line("rw [add_comm] at h1").unwrap();
    match rw_at {
        Some(Tactic::Rw { lemmas, at_hyp }) => {
            assert_eq!(lemmas[0].lemma_name, "add_comm");
            assert_eq!(at_hyp, Some("h1".to_string()));
        }
        _ => panic!("Expected Rw at tactic"),
    }

    let ind = Parser::parse_tactic_line("induction n with d hd").unwrap();
    match ind {
        Some(Tactic::Induction { var, step_var, step_hyp, .. }) => {
            assert_eq!(var, "n");
            assert_eq!(step_var, "d");
            assert_eq!(step_hyp, "hd");
        }
        _ => panic!("Expected Induction tactic"),
    }
}
