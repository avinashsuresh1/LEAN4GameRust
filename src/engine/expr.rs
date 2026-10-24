use serde::{Deserialize, Serialize};
use std::collections::{HashMap, HashSet};
use std::fmt;

/// Mathematical expressions used in theorem proving across all mathematical domains
/// (Peano Arithmetic, Real Analysis, Set Theory, Logic).
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum Expr {
    /// Natural / Real number integer literal: e.g. 0, 1, 2
    Nat(u64),
    /// Variable identifier: e.g. "n", "m", "a", "b", "x", "y", "ε", "δ", "N"
    Var(String),
    /// Successor function: `succ e`
    Succ(Box<Expr>),
    /// Addition: `e1 + e2`
    Add(Box<Expr>, Box<Expr>),
    /// Subtraction: `e1 - e2`
    Sub(Box<Expr>, Box<Expr>),
    /// Multiplication: `e1 * e2`
    Mul(Box<Expr>, Box<Expr>),
    /// Division: `e1 / e2`
    Div(Box<Expr>, Box<Expr>),
    /// Power / Exponentiation: `e1 ^ e2`
    Pow(Box<Expr>, Box<Expr>),
    /// Absolute value: `|e|`
    Abs(Box<Expr>),
    /// Equality: `e1 = e2`
    Eq(Box<Expr>, Box<Expr>),
    /// Inequality: `e1 ≠ e2`
    Ne(Box<Expr>, Box<Expr>),
    /// Less than or equal to: `e1 ≤ e2`
    Le(Box<Expr>, Box<Expr>),
    /// Strictly less than: `e1 < e2`
    Lt(Box<Expr>, Box<Expr>),
    /// Greater than or equal to: `e1 ≥ e2`
    Ge(Box<Expr>, Box<Expr>),
    /// Strictly greater than: `e1 > e2`
    Gt(Box<Expr>, Box<Expr>),
    /// Logical conjunction: `P ∧ Q`
    And(Box<Expr>, Box<Expr>),
    /// Logical disjunction: `P ∨ Q`
    Or(Box<Expr>, Box<Expr>),
    /// Logical negation: `¬ P`
    Not(Box<Expr>),
    /// Implication: `P → Q`
    Arrow(Box<Expr>, Box<Expr>),
    /// Named function / predicate application: e.g. `P(n)`, `f(x)`, `seq_limit a L`
    App(String, Vec<Expr>),
    /// Universal quantifier: `∀ (x y : T), P(x, y)`
    Forall(Vec<(String, String)>, Box<Expr>),
    /// Existential quantifier: `∃ (x : T), P(x)`
    Exists(Vec<(String, String)>, Box<Expr>),
    /// Proposition variable: e.g. "P", "Q", "R"
    Prop(String),
}

impl Expr {
    /// Helper constructors
    pub fn zero() -> Self {
        Expr::Nat(0)
    }

    pub fn one() -> Self {
        Expr::Nat(1)
    }

    pub fn var(name: impl Into<String>) -> Self {
        Expr::Var(name.into())
    }

    pub fn succ(inner: Expr) -> Self {
        Expr::Succ(Box::new(inner))
    }

    pub fn add(lhs: Expr, rhs: Expr) -> Self {
        Expr::Add(Box::new(lhs), Box::new(rhs))
    }

    pub fn sub(lhs: Expr, rhs: Expr) -> Self {
        Expr::Sub(Box::new(lhs), Box::new(rhs))
    }

    pub fn mul(lhs: Expr, rhs: Expr) -> Self {
        Expr::Mul(Box::new(lhs), Box::new(rhs))
    }

    pub fn div(lhs: Expr, rhs: Expr) -> Self {
        Expr::Div(Box::new(lhs), Box::new(rhs))
    }

    pub fn pow(lhs: Expr, rhs: Expr) -> Self {
        Expr::Pow(Box::new(lhs), Box::new(rhs))
    }

    pub fn abs(inner: Expr) -> Self {
        Expr::Abs(Box::new(inner))
    }

    pub fn eq(lhs: Expr, rhs: Expr) -> Self {
        Expr::Eq(Box::new(lhs), Box::new(rhs))
    }

    pub fn ne(lhs: Expr, rhs: Expr) -> Self {
        Expr::Ne(Box::new(lhs), Box::new(rhs))
    }

    pub fn le(lhs: Expr, rhs: Expr) -> Self {
        Expr::Le(Box::new(lhs), Box::new(rhs))
    }

    pub fn lt(lhs: Expr, rhs: Expr) -> Self {
        Expr::Lt(Box::new(lhs), Box::new(rhs))
    }

    pub fn ge(lhs: Expr, rhs: Expr) -> Self {
        Expr::Ge(Box::new(lhs), Box::new(rhs))
    }

    pub fn gt(lhs: Expr, rhs: Expr) -> Self {
        Expr::Gt(Box::new(lhs), Box::new(rhs))
    }

    pub fn and(lhs: Expr, rhs: Expr) -> Self {
        Expr::And(Box::new(lhs), Box::new(rhs))
    }

    pub fn or(lhs: Expr, rhs: Expr) -> Self {
        Expr::Or(Box::new(lhs), Box::new(rhs))
    }

    pub fn not(inner: Expr) -> Self {
        Expr::Not(Box::new(inner))
    }

    pub fn arrow(lhs: Expr, rhs: Expr) -> Self {
        Expr::Arrow(Box::new(lhs), Box::new(rhs))
    }

    pub fn forall(vars: Vec<(&str, &str)>, body: Expr) -> Self {
        Expr::Forall(
            vars.into_iter()
                .map(|(v, t)| (v.to_string(), t.to_string()))
                .collect(),
            Box::new(body),
        )
    }

    pub fn exists(vars: Vec<(&str, &str)>, body: Expr) -> Self {
        Expr::Exists(
            vars.into_iter()
                .map(|(v, t)| (v.to_string(), t.to_string()))
                .collect(),
            Box::new(body),
        )
    }

    /// Strip outer universal quantifiers and collect bound variables.
    pub fn peel_forall(&self) -> (Vec<(String, String)>, &Expr) {
        let mut vars = Vec::new();
        let mut curr = self;
        while let Expr::Forall(v_list, body) = curr {
            vars.extend(v_list.clone());
            curr = body.as_ref();
        }
        (vars, curr)
    }

    /// Strip outer existential quantifiers and collect bound variables.
    pub fn peel_exists(&self) -> (Vec<(String, String)>, &Expr) {
        let mut vars = Vec::new();
        let mut curr = self;
        while let Expr::Exists(v_list, body) = curr {
            vars.extend(v_list.clone());
            curr = body.as_ref();
        }
        (vars, curr)
    }

    /// Check if expression is an equality.
    pub fn as_eq(&self) -> Option<(&Expr, &Expr)> {
        match self {
            Expr::Eq(lhs, rhs) => Some((lhs.as_ref(), rhs.as_ref())),
            _ => None,
        }
    }

    /// Checks if two expressions are mathematically equivalent up to relational symmetry
    /// (e.g. `a > b` is equivalent to `b < a`, `a ≥ b` is equivalent to `b ≤ a`).
    pub fn is_equivalent(&self, other: &Expr) -> bool {
        if self == other {
            return true;
        }
        match (self, other) {
            (Expr::Gt(a, b), Expr::Lt(c, d)) | (Expr::Lt(c, d), Expr::Gt(a, b)) => {
                a.is_equivalent(d) && b.is_equivalent(c)
            }
            (Expr::Ge(a, b), Expr::Le(c, d)) | (Expr::Le(c, d), Expr::Ge(a, b)) => {
                a.is_equivalent(d) && b.is_equivalent(c)
            }
            (Expr::Eq(a, b), Expr::Eq(c, d)) => {
                (a.is_equivalent(c) && b.is_equivalent(d)) || (a.is_equivalent(d) && b.is_equivalent(c))
            }
            _ => false,
        }
    }

    /// Substitute all occurrences of `var` with `replacement`.
    pub fn substitute(&self, var: &str, replacement: &Expr) -> Expr {
        let mut map = HashMap::new();
        map.insert(var.to_string(), replacement.clone());
        self.substitute_map(&map)
    }

    /// Substitute occurrences based on a map of variable names to expressions.
    pub fn substitute_map(&self, map: &HashMap<String, Expr>) -> Expr {
        match self {
            Expr::Nat(n) => Expr::Nat(*n),
            Expr::Var(v) => {
                if let Some(rep) = map.get(v) {
                    rep.clone()
                } else {
                    Expr::Var(v.clone())
                }
            }
            Expr::Succ(inner) => Expr::succ(inner.substitute_map(map)),
            Expr::Add(l, r) => Expr::add(l.substitute_map(map), r.substitute_map(map)),
            Expr::Sub(l, r) => Expr::sub(l.substitute_map(map), r.substitute_map(map)),
            Expr::Mul(l, r) => Expr::mul(l.substitute_map(map), r.substitute_map(map)),
            Expr::Div(l, r) => Expr::div(l.substitute_map(map), r.substitute_map(map)),
            Expr::Pow(l, r) => Expr::pow(l.substitute_map(map), r.substitute_map(map)),
            Expr::Abs(inner) => Expr::abs(inner.substitute_map(map)),
            Expr::Eq(l, r) => Expr::eq(l.substitute_map(map), r.substitute_map(map)),
            Expr::Ne(l, r) => Expr::ne(l.substitute_map(map), r.substitute_map(map)),
            Expr::Le(l, r) => Expr::le(l.substitute_map(map), r.substitute_map(map)),
            Expr::Lt(l, r) => Expr::lt(l.substitute_map(map), r.substitute_map(map)),
            Expr::Ge(l, r) => Expr::ge(l.substitute_map(map), r.substitute_map(map)),
            Expr::Gt(l, r) => Expr::gt(l.substitute_map(map), r.substitute_map(map)),
            Expr::And(l, r) => Expr::and(l.substitute_map(map), r.substitute_map(map)),
            Expr::Or(l, r) => Expr::or(l.substitute_map(map), r.substitute_map(map)),
            Expr::Not(inner) => Expr::not(inner.substitute_map(map)),
            Expr::Arrow(l, r) => Expr::arrow(l.substitute_map(map), r.substitute_map(map)),
            Expr::App(name, args) => {
                let new_args = args.iter().map(|a| a.substitute_map(map)).collect();
                Expr::App(name.clone(), new_args)
            }
            Expr::Forall(vars, body) => {
                let mut inner_map = map.clone();
                for (v, _) in vars {
                    inner_map.remove(v);
                }
                Expr::Forall(vars.clone(), Box::new(body.substitute_map(&inner_map)))
            }
            Expr::Exists(vars, body) => {
                let mut inner_map = map.clone();
                for (v, _) in vars {
                    inner_map.remove(v);
                }
                Expr::Exists(vars.clone(), Box::new(body.substitute_map(&inner_map)))
            }
            Expr::Prop(p) => Expr::Prop(p.clone()),
        }
    }

    /// Collect free variables.
    pub fn free_vars(&self) -> HashSet<String> {
        let mut set = HashSet::new();
        self.collect_free_vars(&mut set);
        set
    }

    fn collect_free_vars(&self, set: &mut HashSet<String>) {
        match self {
            Expr::Nat(_) => {}
            Expr::Var(v) => {
                set.insert(v.clone());
            }
            Expr::Succ(inner) | Expr::Abs(inner) | Expr::Not(inner) => {
                inner.collect_free_vars(set);
            }
            Expr::Add(l, r)
            | Expr::Sub(l, r)
            | Expr::Mul(l, r)
            | Expr::Div(l, r)
            | Expr::Pow(l, r)
            | Expr::Eq(l, r)
            | Expr::Ne(l, r)
            | Expr::Le(l, r)
            | Expr::Lt(l, r)
            | Expr::Ge(l, r)
            | Expr::Gt(l, r)
            | Expr::And(l, r)
            | Expr::Or(l, r)
            | Expr::Arrow(l, r) => {
                l.collect_free_vars(set);
                r.collect_free_vars(set);
            }
            Expr::App(_, args) => {
                for a in args {
                    a.collect_free_vars(set);
                }
            }
            Expr::Forall(vars, body) | Expr::Exists(vars, body) => {
                let mut body_vars = HashSet::new();
                body.collect_free_vars(&mut body_vars);
                for (v, _) in vars {
                    body_vars.remove(v);
                }
                set.extend(body_vars);
            }
            Expr::Prop(p) => {
                set.insert(p.clone());
            }
        }
    }

    /// Matches `pattern` against `self`. If matching succeeds, populates `bindings`.
    pub fn match_pattern(
        &self,
        pattern: &Expr,
        pattern_vars: &HashSet<String>,
        bindings: &mut HashMap<String, Expr>,
    ) -> bool {
        match pattern {
            Expr::Var(name) if pattern_vars.contains(name) => {
                if let Some(existing) = bindings.get(name) {
                    existing == self
                } else {
                    bindings.insert(name.clone(), self.clone());
                    true
                }
            }
            Expr::Nat(n1) => match self {
                Expr::Nat(n2) => n1 == n2,
                _ => false,
            },
            Expr::Var(v1) => match self {
                Expr::Var(v2) => v1 == v2,
                _ => false,
            },
            Expr::Succ(p_inner) => match self {
                Expr::Succ(s_inner) => s_inner.match_pattern(p_inner, pattern_vars, bindings),
                _ => false,
            },
            Expr::Abs(p_inner) => match self {
                Expr::Abs(s_inner) => s_inner.match_pattern(p_inner, pattern_vars, bindings),
                _ => false,
            },
            Expr::Not(p_inner) => match self {
                Expr::Not(s_inner) => s_inner.match_pattern(p_inner, pattern_vars, bindings),
                _ => false,
            },
            Expr::Add(pl, pr) => match self {
                Expr::Add(sl, sr) => {
                    sl.match_pattern(pl, pattern_vars, bindings)
                        && sr.match_pattern(pr, pattern_vars, bindings)
                }
                _ => false,
            },
            Expr::Sub(pl, pr) => match self {
                Expr::Sub(sl, sr) => {
                    sl.match_pattern(pl, pattern_vars, bindings)
                        && sr.match_pattern(pr, pattern_vars, bindings)
                }
                _ => false,
            },
            Expr::Mul(pl, pr) => match self {
                Expr::Mul(sl, sr) => {
                    sl.match_pattern(pl, pattern_vars, bindings)
                        && sr.match_pattern(pr, pattern_vars, bindings)
                }
                _ => false,
            },
            Expr::Div(pl, pr) => match self {
                Expr::Div(sl, sr) => {
                    sl.match_pattern(pl, pattern_vars, bindings)
                        && sr.match_pattern(pr, pattern_vars, bindings)
                }
                _ => false,
            },
            Expr::Pow(pl, pr) => match self {
                Expr::Pow(sl, sr) => {
                    sl.match_pattern(pl, pattern_vars, bindings)
                        && sr.match_pattern(pr, pattern_vars, bindings)
                }
                _ => false,
            },
            Expr::Eq(pl, pr) => match self {
                Expr::Eq(sl, sr) => {
                    sl.match_pattern(pl, pattern_vars, bindings)
                        && sr.match_pattern(pr, pattern_vars, bindings)
                }
                _ => false,
            },
            Expr::Ne(pl, pr) => match self {
                Expr::Ne(sl, sr) => {
                    sl.match_pattern(pl, pattern_vars, bindings)
                        && sr.match_pattern(pr, pattern_vars, bindings)
                }
                _ => false,
            },
            Expr::Le(pl, pr) => match self {
                Expr::Le(sl, sr) => {
                    sl.match_pattern(pl, pattern_vars, bindings)
                        && sr.match_pattern(pr, pattern_vars, bindings)
                }
                Expr::Ge(sr, sl) => {
                    sl.match_pattern(pl, pattern_vars, bindings)
                        && sr.match_pattern(pr, pattern_vars, bindings)
                }
                _ => false,
            },
            Expr::Lt(pl, pr) => match self {
                Expr::Lt(sl, sr) => {
                    sl.match_pattern(pl, pattern_vars, bindings)
                        && sr.match_pattern(pr, pattern_vars, bindings)
                }
                Expr::Gt(sr, sl) => {
                    sl.match_pattern(pl, pattern_vars, bindings)
                        && sr.match_pattern(pr, pattern_vars, bindings)
                }
                _ => false,
            },
            Expr::Ge(pl, pr) => match self {
                Expr::Ge(sl, sr) => {
                    sl.match_pattern(pl, pattern_vars, bindings)
                        && sr.match_pattern(pr, pattern_vars, bindings)
                }
                Expr::Le(sr, sl) => {
                    sl.match_pattern(pl, pattern_vars, bindings)
                        && sr.match_pattern(pr, pattern_vars, bindings)
                }
                _ => false,
            },
            Expr::Gt(pl, pr) => match self {
                Expr::Gt(sl, sr) => {
                    sl.match_pattern(pl, pattern_vars, bindings)
                        && sr.match_pattern(pr, pattern_vars, bindings)
                }
                Expr::Lt(sr, sl) => {
                    sl.match_pattern(pl, pattern_vars, bindings)
                        && sr.match_pattern(pr, pattern_vars, bindings)
                }
                _ => false,
            },
            Expr::And(pl, pr) => match self {
                Expr::And(sl, sr) => {
                    sl.match_pattern(pl, pattern_vars, bindings)
                        && sr.match_pattern(pr, pattern_vars, bindings)
                }
                _ => false,
            },
            Expr::Or(pl, pr) => match self {
                Expr::Or(sl, sr) => {
                    sl.match_pattern(pl, pattern_vars, bindings)
                        && sr.match_pattern(pr, pattern_vars, bindings)
                }
                _ => false,
            },
            Expr::Arrow(pl, pr) => match self {
                Expr::Arrow(sl, sr) => {
                    sl.match_pattern(pl, pattern_vars, bindings)
                        && sr.match_pattern(pr, pattern_vars, bindings)
                }
                _ => false,
            },
            Expr::App(p_name, p_args) => match self {
                Expr::App(s_name, s_args) if p_name == s_name && p_args.len() == s_args.len() => {
                    for (s_arg, p_arg) in s_args.iter().zip(p_args.iter()) {
                        if !s_arg.match_pattern(p_arg, pattern_vars, bindings) {
                            return false;
                        }
                    }
                    true
                }
                _ => false,
            },
            Expr::Prop(p1) => match self {
                Expr::Prop(p2) => p1 == p2,
                _ => false,
            },
            Expr::Forall(p_vars, p_body) => match self {
                Expr::Forall(s_vars, s_body) if p_vars == s_vars => {
                    s_body.match_pattern(p_body, pattern_vars, bindings)
                }
                _ => false,
            },
            Expr::Exists(p_vars, p_body) => match self {
                Expr::Exists(s_vars, s_body) if p_vars == s_vars => {
                    s_body.match_pattern(p_body, pattern_vars, bindings)
                }
                _ => false,
            },
        }
    }

    /// Rewrites the first matching subexpression using equality `lhs = rhs`.
    pub fn rewrite_once(
        &self,
        lhs: &Expr,
        rhs: &Expr,
        pattern_vars: &HashSet<String>,
        reverse: bool,
    ) -> Option<Expr> {
        let (from_pat, to_pat) = if reverse { (rhs, lhs) } else { (lhs, rhs) };

        // 1. Try matching the entire expression first
        let mut bindings = HashMap::new();
        if self.match_pattern(from_pat, pattern_vars, &mut bindings) {
            return Some(to_pat.substitute_map(&bindings));
        }

        // 2. Otherwise recursively match subterms
        match self {
            Expr::Succ(inner) => inner
                .rewrite_once(lhs, rhs, pattern_vars, reverse)
                .map(Expr::succ),
            Expr::Abs(inner) => inner
                .rewrite_once(lhs, rhs, pattern_vars, reverse)
                .map(Expr::abs),
            Expr::Not(inner) => inner
                .rewrite_once(lhs, rhs, pattern_vars, reverse)
                .map(Expr::not),
            Expr::Add(l, r) => {
                if let Some(new_l) = l.rewrite_once(lhs, rhs, pattern_vars, reverse) {
                    Some(Expr::add(new_l, *r.clone()))
                } else if let Some(new_r) = r.rewrite_once(lhs, rhs, pattern_vars, reverse) {
                    Some(Expr::add(*l.clone(), new_r))
                } else {
                    None
                }
            }
            Expr::Sub(l, r) => {
                if let Some(new_l) = l.rewrite_once(lhs, rhs, pattern_vars, reverse) {
                    Some(Expr::sub(new_l, *r.clone()))
                } else if let Some(new_r) = r.rewrite_once(lhs, rhs, pattern_vars, reverse) {
                    Some(Expr::sub(*l.clone(), new_r))
                } else {
                    None
                }
            }
            Expr::Mul(l, r) => {
                if let Some(new_l) = l.rewrite_once(lhs, rhs, pattern_vars, reverse) {
                    Some(Expr::mul(new_l, *r.clone()))
                } else if let Some(new_r) = r.rewrite_once(lhs, rhs, pattern_vars, reverse) {
                    Some(Expr::mul(*l.clone(), new_r))
                } else {
                    None
                }
            }
            Expr::Div(l, r) => {
                if let Some(new_l) = l.rewrite_once(lhs, rhs, pattern_vars, reverse) {
                    Some(Expr::div(new_l, *r.clone()))
                } else if let Some(new_r) = r.rewrite_once(lhs, rhs, pattern_vars, reverse) {
                    Some(Expr::div(*l.clone(), new_r))
                } else {
                    None
                }
            }
            Expr::Pow(l, r) => {
                if let Some(new_l) = l.rewrite_once(lhs, rhs, pattern_vars, reverse) {
                    Some(Expr::pow(new_l, *r.clone()))
                } else if let Some(new_r) = r.rewrite_once(lhs, rhs, pattern_vars, reverse) {
                    Some(Expr::pow(*l.clone(), new_r))
                } else {
                    None
                }
            }
            Expr::Eq(l, r) => {
                if let Some(new_l) = l.rewrite_once(lhs, rhs, pattern_vars, reverse) {
                    Some(Expr::eq(new_l, *r.clone()))
                } else if let Some(new_r) = r.rewrite_once(lhs, rhs, pattern_vars, reverse) {
                    Some(Expr::eq(*l.clone(), new_r))
                } else {
                    None
                }
            }
            Expr::Ne(l, r) => {
                if let Some(new_l) = l.rewrite_once(lhs, rhs, pattern_vars, reverse) {
                    Some(Expr::ne(new_l, *r.clone()))
                } else if let Some(new_r) = r.rewrite_once(lhs, rhs, pattern_vars, reverse) {
                    Some(Expr::ne(*l.clone(), new_r))
                } else {
                    None
                }
            }
            Expr::Le(l, r) => {
                if let Some(new_l) = l.rewrite_once(lhs, rhs, pattern_vars, reverse) {
                    Some(Expr::le(new_l, *r.clone()))
                } else if let Some(new_r) = r.rewrite_once(lhs, rhs, pattern_vars, reverse) {
                    Some(Expr::le(*l.clone(), new_r))
                } else {
                    None
                }
            }
            Expr::Lt(l, r) => {
                if let Some(new_l) = l.rewrite_once(lhs, rhs, pattern_vars, reverse) {
                    Some(Expr::lt(new_l, *r.clone()))
                } else if let Some(new_r) = r.rewrite_once(lhs, rhs, pattern_vars, reverse) {
                    Some(Expr::lt(*l.clone(), new_r))
                } else {
                    None
                }
            }
            Expr::Ge(l, r) => {
                if let Some(new_l) = l.rewrite_once(lhs, rhs, pattern_vars, reverse) {
                    Some(Expr::ge(new_l, *r.clone()))
                } else if let Some(new_r) = r.rewrite_once(lhs, rhs, pattern_vars, reverse) {
                    Some(Expr::ge(*l.clone(), new_r))
                } else {
                    None
                }
            }
            Expr::Gt(l, r) => {
                if let Some(new_l) = l.rewrite_once(lhs, rhs, pattern_vars, reverse) {
                    Some(Expr::gt(new_l, *r.clone()))
                } else if let Some(new_r) = r.rewrite_once(lhs, rhs, pattern_vars, reverse) {
                    Some(Expr::gt(*l.clone(), new_r))
                } else {
                    None
                }
            }
            Expr::And(l, r) => {
                if let Some(new_l) = l.rewrite_once(lhs, rhs, pattern_vars, reverse) {
                    Some(Expr::and(new_l, *r.clone()))
                } else if let Some(new_r) = r.rewrite_once(lhs, rhs, pattern_vars, reverse) {
                    Some(Expr::and(*l.clone(), new_r))
                } else {
                    None
                }
            }
            Expr::Or(l, r) => {
                if let Some(new_l) = l.rewrite_once(lhs, rhs, pattern_vars, reverse) {
                    Some(Expr::or(new_l, *r.clone()))
                } else if let Some(new_r) = r.rewrite_once(lhs, rhs, pattern_vars, reverse) {
                    Some(Expr::or(*l.clone(), new_r))
                } else {
                    None
                }
            }
            Expr::Arrow(l, r) => {
                if let Some(new_l) = l.rewrite_once(lhs, rhs, pattern_vars, reverse) {
                    Some(Expr::arrow(new_l, *r.clone()))
                } else if let Some(new_r) = r.rewrite_once(lhs, rhs, pattern_vars, reverse) {
                    Some(Expr::arrow(*l.clone(), new_r))
                } else {
                    None
                }
            }
            Expr::App(name, args) => {
                for i in 0..args.len() {
                    if let Some(new_arg) = args[i].rewrite_once(lhs, rhs, pattern_vars, reverse) {
                        let mut new_args = args.clone();
                        new_args[i] = new_arg;
                        return Some(Expr::App(name.clone(), new_args));
                    }
                }
                None
            }
            _ => None,
        }
    }
}

impl fmt::Display for Expr {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Expr::Nat(n) => write!(f, "{}", n),
            Expr::Var(v) => write!(f, "{}", v),
            Expr::Succ(inner) => {
                if matches!(inner.as_ref(), Expr::Nat(_) | Expr::Var(_)) {
                    write!(f, "succ {}", inner)
                } else {
                    write!(f, "succ ({})", inner)
                }
            }
            Expr::Abs(inner) => write!(f, "|{}|", inner),
            Expr::Add(l, r) => {
                let l_str = match l.as_ref() {
                    Expr::Eq(..) | Expr::Arrow(..) | Expr::Forall(..) | Expr::Exists(..) => format!("({})", l),
                    _ => format!("{}", l),
                };
                let r_str = match r.as_ref() {
                    Expr::Add(..) | Expr::Sub(..) | Expr::Eq(..) | Expr::Arrow(..) | Expr::Forall(..) | Expr::Exists(..) => {
                        format!("({})", r)
                    }
                    _ => format!("{}", r),
                };
                write!(f, "{} + {}", l_str, r_str)
            }
            Expr::Sub(l, r) => {
                let l_str = match l.as_ref() {
                    Expr::Eq(..) | Expr::Arrow(..) | Expr::Forall(..) | Expr::Exists(..) => format!("({})", l),
                    _ => format!("{}", l),
                };
                let r_str = match r.as_ref() {
                    Expr::Add(..) | Expr::Sub(..) | Expr::Eq(..) | Expr::Arrow(..) => format!("({})", r),
                    _ => format!("{}", r),
                };
                write!(f, "{} - {}", l_str, r_str)
            }
            Expr::Mul(l, r) => {
                let l_str = match l.as_ref() {
                    Expr::Add(..) | Expr::Sub(..) | Expr::Eq(..) | Expr::Arrow(..) => format!("({})", l),
                    _ => format!("{}", l),
                };
                let r_str = match r.as_ref() {
                    Expr::Add(..) | Expr::Sub(..) | Expr::Mul(..) | Expr::Div(..) | Expr::Eq(..) | Expr::Arrow(..) => {
                        format!("({})", r)
                    }
                    _ => format!("{}", r),
                };
                write!(f, "{} * {}", l_str, r_str)
            }
            Expr::Div(l, r) => {
                let l_str = match l.as_ref() {
                    Expr::Add(..) | Expr::Sub(..) | Expr::Eq(..) | Expr::Arrow(..) => format!("({})", l),
                    _ => format!("{}", l),
                };
                let r_str = match r.as_ref() {
                    Expr::Add(..) | Expr::Sub(..) | Expr::Mul(..) | Expr::Div(..) | Expr::Eq(..) | Expr::Arrow(..) => {
                        format!("({})", r)
                    }
                    _ => format!("{}", r),
                };
                write!(f, "{} / {}", l_str, r_str)
            }
            Expr::Pow(l, r) => {
                let l_str = match l.as_ref() {
                    Expr::Add(..) | Expr::Sub(..) | Expr::Mul(..) | Expr::Div(..) | Expr::Pow(..) | Expr::Succ(..) => {
                        format!("({})", l)
                    }
                    _ => format!("{}", l),
                };
                let r_str = match r.as_ref() {
                    Expr::Add(..) | Expr::Sub(..) | Expr::Mul(..) | Expr::Div(..) => format!("({})", r),
                    _ => format!("{}", r),
                };
                write!(f, "{} ^ {}", l_str, r_str)
            }
            Expr::Eq(l, r) => write!(f, "{} = {}", l, r),
            Expr::Ne(l, r) => write!(f, "{} ≠ {}", l, r),
            Expr::Le(l, r) => write!(f, "{} ≤ {}", l, r),
            Expr::Lt(l, r) => write!(f, "{} < {}", l, r),
            Expr::Ge(l, r) => write!(f, "{} ≥ {}", l, r),
            Expr::Gt(l, r) => write!(f, "{} > {}", l, r),
            Expr::And(l, r) => write!(f, "{} ∧ {}", l, r),
            Expr::Or(l, r) => write!(f, "{} ∨ {}", l, r),
            Expr::Not(inner) => write!(f, "¬{}", inner),
            Expr::Arrow(l, r) => {
                if matches!(l.as_ref(), Expr::Arrow(..)) {
                    write!(f, "({}) → {}", l, r)
                } else {
                    write!(f, "{} → {}", l, r)
                }
            }
            Expr::App(name, args) => {
                write!(f, "{}", name)?;
                for arg in args {
                    if matches!(arg, Expr::Nat(_) | Expr::Var(_)) {
                        write!(f, " {}", arg)?;
                    } else {
                        write!(f, " ({})", arg)?;
                    }
                }
                Ok(())
            }
            Expr::Forall(vars, body) => {
                write!(f, "∀")?;
                for (v, t) in vars {
                    write!(f, " ({} : {})", v, t)?;
                }
                write!(f, ", {}", body)
            }
            Expr::Exists(vars, body) => {
                write!(f, "∃")?;
                for (v, t) in vars {
                    write!(f, " ({} : {})", v, t)?;
                }
                write!(f, ", {}", body)
            }
            Expr::Prop(p) => write!(f, "{}", p),
        }
    }
}
