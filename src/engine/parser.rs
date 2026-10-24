use crate::engine::expr::Expr;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Tactic {
    /// `rfl`
    Rfl,
    /// `rw [lem1, ← lem2, ...]` with optional target hypothesis
    Rw {
        lemmas: Vec<RwStep>,
        at_hyp: Option<String>,
    },
    /// `induction var with d hd`
    Induction {
        var: String,
        base_name: Option<String>,
        step_var: String,
        step_hyp: String,
    },
    /// `exact term`
    Exact(String),
    /// `intro x` or `intro a b`
    Intro(Vec<String>),
    /// `apply h`
    Apply(String),
    /// `symm`
    Symm,
    /// `use <witness>` (instantiates existential quantifier `∃`)
    Use(String),
    /// `constructor` (splits conjunction `P ∧ Q` into subgoals `P` and `Q`)
    Constructor,
    /// `rcases h with ⟨a, b⟩`
    Rcases {
        hyp: String,
        names: Vec<String>,
    },
    /// `dsimp` (simplifies definitions)
    Dsimp,
    /// `linarith` (linear arithmetic)
    Linarith,
    /// `repeat <tactic>`
    Repeat(Box<Tactic>),
    /// `have h : <type>`
    Have {
        name: String,
        prop: Expr,
    },
    /// `cases var with ...`
    Cases {
        var: String,
        names: Vec<String>,
    },
    /// `sorry`
    Sorry,
    /// Subgoal bullet `·`
    Bullet,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RwStep {
    pub lemma_name: String,
    pub reverse: bool,
}

pub struct Parser<'a> {
    input: &'a str,
    pos: usize,
}

impl<'a> Parser<'a> {
    pub fn new(input: &'a str) -> Self {
        Self { input, pos: 0 }
    }

    fn peek_char(&self) -> Option<char> {
        self.input[self.pos..].chars().next()
    }

    fn next_char(&mut self) -> Option<char> {
        let ch = self.peek_char()?;
        self.pos += ch.len_utf8();
        Some(ch)
    }

    fn skip_whitespace(&mut self) {
        while let Some(ch) = self.peek_char() {
            if ch.is_whitespace() {
                self.next_char();
            } else if self.starts_with("--") {
                // Single line comment
                while let Some(c) = self.next_char() {
                    if c == '\n' {
                        break;
                    }
                }
            } else if self.starts_with("/*") || self.starts_with("/-") {
                // Block comment
                self.pos += 2;
                while self.pos < self.input.len() {
                    if self.starts_with("*/") || self.starts_with("-/") {
                        self.pos += 2;
                        break;
                    }
                    self.next_char();
                }
            } else {
                break;
            }
        }
    }

    fn starts_with(&self, s: &str) -> bool {
        self.input[self.pos..].starts_with(s)
    }

    fn consume_if(&mut self, s: &str) -> bool {
        if self.starts_with(s) {
            self.pos += s.len();
            true
        } else {
            false
        }
    }

    pub fn parse_ident(&mut self) -> Option<String> {
        self.skip_whitespace();
        let start = self.pos;
        let mut count = 0;
        while let Some(ch) = self.peek_char() {
            if ch.is_alphanumeric() || ch == '_' || ch == '\'' {
                self.next_char();
                count += 1;
            } else {
                break;
            }
        }
        if count > 0 {
            Some(self.input[start..self.pos].to_string())
        } else {
            None
        }
    }

    pub fn parse_expr_str(input: &str) -> Result<Expr, String> {
        let mut parser = Parser::new(input);
        let expr = parser.parse_expr()?;
        parser.skip_whitespace();
        if parser.pos < parser.input.len() {
            return Err(format!(
                "Unexpected trailing characters in expression: '{}'",
                &parser.input[parser.pos..]
            ));
        }
        Ok(expr)
    }

    /// Parse complete expression with operator precedence.
    pub fn parse_expr(&mut self) -> Result<Expr, String> {
        self.skip_whitespace();
        if self.starts_with("∀") || self.starts_with("\\all") || self.starts_with("forall") {
            return self.parse_forall();
        }
        if self.starts_with("∃") || self.starts_with("\\exists") || self.starts_with("exists") {
            return self.parse_exists();
        }
        self.parse_implication()
    }

    fn parse_forall(&mut self) -> Result<Expr, String> {
        if !self.consume_if("∀") && !self.consume_if("\\all") && !self.consume_if("forall") {
            return Err("Expected quantifier".to_string());
        }
        self.skip_whitespace();
        let vars = self.parse_quantifier_vars()?;
        self.skip_whitespace();
        self.consume_if(",");
        let body = self.parse_expr()?;
        Ok(Expr::Forall(vars, Box::new(body)))
    }

    fn parse_exists(&mut self) -> Result<Expr, String> {
        if !self.consume_if("∃") && !self.consume_if("\\exists") && !self.consume_if("exists") {
            return Err("Expected existential quantifier".to_string());
        }
        self.skip_whitespace();
        let vars = self.parse_quantifier_vars()?;
        self.skip_whitespace();
        self.consume_if(",");
        let body = self.parse_expr()?;
        Ok(Expr::Exists(vars, Box::new(body)))
    }

    fn parse_quantifier_vars(&mut self) -> Result<Vec<(String, String)>, String> {
        let mut vars = Vec::new();

        while self.starts_with("(") {
            self.consume_if("(");
            let mut var_names = Vec::new();
            while let Some(id) = self.parse_ident() {
                var_names.push(id);
                self.skip_whitespace();
                if self.starts_with(":") {
                    break;
                }
            }
            if !self.consume_if(":") {
                return Err("Expected ':' in quantifier variable declaration".to_string());
            }
            self.skip_whitespace();
            let typ = self.parse_ident().unwrap_or_else(|| "ℝ".to_string());
            self.skip_whitespace();
            self.consume_if(")");
            self.skip_whitespace();

            for v in var_names {
                vars.push((v, typ.clone()));
            }
        }

        if vars.is_empty() {
            while let Some(id) = self.parse_ident() {
                vars.push((id, "ℝ".to_string()));
                self.skip_whitespace();
                if self.starts_with(",") {
                    break;
                }
            }
        }

        Ok(vars)
    }

    fn parse_implication(&mut self) -> Result<Expr, String> {
        let lhs = self.parse_or()?;
        self.skip_whitespace();
        if self.consume_if("→") || self.consume_if("->") {
            let rhs = self.parse_expr()?;
            Ok(Expr::arrow(lhs, rhs))
        } else {
            Ok(lhs)
        }
    }

    fn parse_or(&mut self) -> Result<Expr, String> {
        let mut expr = self.parse_and()?;
        loop {
            self.skip_whitespace();
            if self.consume_if("∨") || self.consume_if("\\/") {
                let rhs = self.parse_and()?;
                expr = Expr::or(expr, rhs);
            } else {
                break;
            }
        }
        Ok(expr)
    }

    fn parse_and(&mut self) -> Result<Expr, String> {
        let mut expr = self.parse_equality()?;
        loop {
            self.skip_whitespace();
            if self.consume_if("∧") || self.consume_if("/\\") {
                let rhs = self.parse_equality()?;
                expr = Expr::and(expr, rhs);
            } else {
                break;
            }
        }
        Ok(expr)
    }

    fn parse_equality(&mut self) -> Result<Expr, String> {
        let lhs = self.parse_additive()?;
        self.skip_whitespace();
        if self.consume_if("≠") || self.consume_if("!=") {
            let rhs = self.parse_additive()?;
            Ok(Expr::ne(lhs, rhs))
        } else if self.consume_if("≤") || self.consume_if("<=") {
            let rhs = self.parse_additive()?;
            Ok(Expr::le(lhs, rhs))
        } else if self.consume_if("≥") || self.consume_if(">=") {
            let rhs = self.parse_additive()?;
            Ok(Expr::ge(lhs, rhs))
        } else if self.consume_if("<") {
            let rhs = self.parse_additive()?;
            Ok(Expr::lt(lhs, rhs))
        } else if self.consume_if(">") {
            let rhs = self.parse_additive()?;
            Ok(Expr::gt(lhs, rhs))
        } else if self.consume_if("=") {
            let rhs = self.parse_additive()?;
            Ok(Expr::eq(lhs, rhs))
        } else {
            Ok(lhs)
        }
    }

    fn parse_additive(&mut self) -> Result<Expr, String> {
        let mut expr = self.parse_multiplicative()?;
        loop {
            self.skip_whitespace();
            if self.consume_if("+") {
                let rhs = self.parse_multiplicative()?;
                expr = Expr::add(expr, rhs);
            } else if self.consume_if("-") {
                let rhs = self.parse_multiplicative()?;
                expr = Expr::sub(expr, rhs);
            } else {
                break;
            }
        }
        Ok(expr)
    }

    fn parse_multiplicative(&mut self) -> Result<Expr, String> {
        let mut expr = self.parse_exponential()?;
        loop {
            self.skip_whitespace();
            if self.consume_if("*") {
                let rhs = self.parse_exponential()?;
                expr = Expr::mul(expr, rhs);
            } else if self.consume_if("/") {
                let rhs = self.parse_exponential()?;
                expr = Expr::div(expr, rhs);
            } else {
                break;
            }
        }
        Ok(expr)
    }

    fn parse_exponential(&mut self) -> Result<Expr, String> {
        let lhs = self.parse_application()?;
        self.skip_whitespace();
        if self.consume_if("^") {
            let rhs = self.parse_exponential()?;
            Ok(Expr::pow(lhs, rhs))
        } else {
            Ok(lhs)
        }
    }

    fn parse_application(&mut self) -> Result<Expr, String> {
        let mut expr = self.parse_primary()?;
        loop {
            self.skip_whitespace();
            if self.pos >= self.input.len() {
                break;
            }
            if self.starts_with("+")
                || self.starts_with("-")
                || self.starts_with("*")
                || self.starts_with("/")
                || self.starts_with("^")
                || self.starts_with("=")
                || self.starts_with("≠")
                || self.starts_with("!=")
                || self.starts_with("≤")
                || self.starts_with("<=")
                || self.starts_with("≥")
                || self.starts_with(">=")
                || self.starts_with("<")
                || self.starts_with(">")
                || self.starts_with("→")
                || self.starts_with("->")
                || self.starts_with("∧")
                || self.starts_with("/\\")
                || self.starts_with("∨")
                || self.starts_with("\\/")
                || self.starts_with(")")
                || self.starts_with(",")
                || self.starts_with(":")
                || self.starts_with("|")
                || self.starts_with("·")
                || self.starts_with("⟨")
                || self.starts_with("⟩")
            {
                break;
            }

            match &expr {
                Expr::Var(_) | Expr::App(..) => {
                    let arg = self.parse_primary()?;
                    match expr {
                        Expr::Var(f) => {
                            expr = Expr::App(f, vec![arg]);
                        }
                        Expr::App(f, mut args) => {
                            args.push(arg);
                            expr = Expr::App(f, args);
                        }
                        _ => unreachable!(),
                    }
                }
                _ => break,
            }
        }
        Ok(expr)
    }

    fn parse_primary(&mut self) -> Result<Expr, String> {
        self.skip_whitespace();

        // Quantifiers inside primary expressions (e.g. within parentheses or conjunctions)
        if self.starts_with("∀") || self.starts_with("\\all") || self.starts_with("forall") {
            return self.parse_forall();
        }
        if self.starts_with("∃") || self.starts_with("\\exists") || self.starts_with("exists") {
            return self.parse_exists();
        }

        // Negation: ¬ e
        if self.consume_if("¬") || self.consume_if("~") {
            let inner = self.parse_primary()?;
            return Ok(Expr::not(inner));
        }

        // Absolute value: |e|
        if self.consume_if("|") {
            let inner = self.parse_expr()?;
            self.skip_whitespace();
            if !self.consume_if("|") {
                return Err("Expected closing '|'".to_string());
            }
            return Ok(Expr::abs(inner));
        }

        // Parentheses: (e)
        if self.consume_if("(") {
            let inner = self.parse_expr()?;
            self.skip_whitespace();
            if !self.consume_if(")") {
                return Err("Expected closing ')'".to_string());
            }
            return Ok(inner);
        }

        // Numeric literal
        if let Some(ch) = self.peek_char() {
            if ch.is_ascii_digit() {
                let mut num_str = String::new();
                while let Some(c) = self.peek_char() {
                    if c.is_ascii_digit() {
                        num_str.push(c);
                        self.next_char();
                    } else {
                        break;
                    }
                }
                let n: u64 = num_str.parse().map_err(|e| format!("{}", e))?;
                return Ok(Expr::Nat(n));
            }
        }

        // Identifiers or keywords
        if let Some(id) = self.parse_ident() {
            if id == "succ" {
                self.skip_whitespace();
                let arg = self.parse_primary()?;
                return Ok(Expr::succ(arg));
            }
            return Ok(Expr::Var(id));
        }

        Err(format!(
            "Unexpected token at: '{}'",
            &self.input[self.pos..self.input.len().min(self.pos + 15)]
        ))
    }

    /// Parse a single line tactic from Lean 4 script.
    pub fn parse_tactic_line(line: &str) -> Result<Option<Tactic>, String> {
        let mut trimmed = line.trim();
        if trimmed.is_empty() || trimmed.starts_with("--") {
            return Ok(None);
        }

        // Bullet point: · tactic or just ·
        if trimmed.starts_with('·') || trimmed.starts_with('*') || trimmed.starts_with('-') {
            let after = trimmed
                .trim_start_matches(|c| c == '·' || c == '*' || c == '-')
                .trim();
            if after.is_empty() {
                return Ok(Some(Tactic::Bullet));
            }
            trimmed = after;
        }

        let mut parser = Parser::new(trimmed);
        parser.skip_whitespace();

        let keyword = parser
            .parse_ident()
            .ok_or_else(|| "Expected tactic keyword".to_string())?;

        match keyword.as_str() {
            "rfl" => Ok(Some(Tactic::Rfl)),
            "sorry" => Ok(Some(Tactic::Sorry)),
            "symm" => Ok(Some(Tactic::Symm)),
            "constructor" => Ok(Some(Tactic::Constructor)),
            "dsimp" => Ok(Some(Tactic::Dsimp)),
            "linarith" => Ok(Some(Tactic::Linarith)),
            "use" => {
                parser.skip_whitespace();
                let witness = parser.input[parser.pos..].trim().to_string();
                if witness.is_empty() {
                    return Err("Expected witness term after use".to_string());
                }
                Ok(Some(Tactic::Use(witness)))
            }
            "rcases" => {
                parser.skip_whitespace();
                let hyp = parser
                    .parse_ident()
                    .ok_or_else(|| "Expected hypothesis name after rcases".to_string())?;
                parser.skip_whitespace();
                let mut names = Vec::new();
                if parser.consume_if("with") {
                    parser.skip_whitespace();
                    let rest = parser.input[parser.pos..]
                        .trim()
                        .trim_matches(|c| c == '⟨' || c == '⟩' || c == '<' || c == '>');
                    for part in rest.split(',') {
                        let trimmed_part = part.trim();
                        if !trimmed_part.is_empty() {
                            names.push(trimmed_part.to_string());
                        }
                    }
                }
                Ok(Some(Tactic::Rcases { hyp, names }))
            }
            "rw" | "rewrite" => {
                parser.skip_whitespace();
                if !parser.consume_if("[") {
                    return Err("Expected '[' after rw".to_string());
                }

                let mut steps = Vec::new();
                loop {
                    parser.skip_whitespace();
                    if parser.consume_if("]") {
                        break;
                    }

                    // Check for reverse arrow: `←`, `<-`, or `\l`
                    let reverse = parser.consume_if("←")
                        || parser.consume_if("<-")
                        || parser.consume_if("\\l");
                    parser.skip_whitespace();

                    let lemma_name = parser
                        .parse_ident()
                        .ok_or_else(|| "Expected lemma name in rw".to_string())?;

                    steps.push(RwStep {
                        lemma_name,
                        reverse,
                    });

                    parser.skip_whitespace();
                    parser.consume_if(",");
                }

                // Check for `at <hyp>`
                parser.skip_whitespace();
                let at_hyp = if parser.starts_with("at") {
                    parser.consume_if("at");
                    parser.skip_whitespace();
                    parser.parse_ident()
                } else {
                    None
                };

                Ok(Some(Tactic::Rw {
                    lemmas: steps,
                    at_hyp,
                }))
            }
            "induction" | "induction'" => {
                parser.skip_whitespace();
                let var = parser
                    .parse_ident()
                    .ok_or_else(|| "Expected variable after induction".to_string())?;

                parser.skip_whitespace();
                let base_name = None;
                let mut step_var = format!("{}_d", var);
                let mut step_hyp = format!("{}_hd", var);

                if parser.consume_if("with") {
                    parser.skip_whitespace();
                    if parser.starts_with("|") {
                        let rest = &parser.input[parser.pos..];
                        if let Some(pos) = rest.find("succ") {
                            let mut inner_parser = Parser::new(&rest[pos + 4..]);
                            if let Some(v) = inner_parser.parse_ident() {
                                step_var = v;
                            }
                            if let Some(h) = inner_parser.parse_ident() {
                                step_hyp = h;
                            }
                        }
                    } else {
                        let id1 = parser.parse_ident();
                        let id2 = parser.parse_ident();
                        match (id1, id2) {
                            (Some(v), Some(h)) => {
                                step_var = v;
                                step_hyp = h;
                            }
                            (Some(v), None) => {
                                step_var = v.clone();
                                step_hyp = format!("h{}", v);
                            }
                            _ => {}
                        }
                    }
                }

                Ok(Some(Tactic::Induction {
                    var,
                    base_name,
                    step_var,
                    step_hyp,
                }))
            }
            "exact" => {
                parser.skip_whitespace();
                let rest = parser.input[parser.pos..].trim().to_string();
                if rest.is_empty() {
                    return Err("Expected term after exact".to_string());
                }
                Ok(Some(Tactic::Exact(rest)))
            }
            "intro" => {
                let mut vars = Vec::new();
                while let Some(id) = parser.parse_ident() {
                    vars.push(id);
                }
                if vars.is_empty() {
                    vars.push("h".to_string());
                }
                Ok(Some(Tactic::Intro(vars)))
            }
            "apply" => {
                parser.skip_whitespace();
                let term = parser
                    .parse_ident()
                    .ok_or_else(|| "Expected term after apply".to_string())?;
                Ok(Some(Tactic::Apply(term)))
            }
            "repeat" => {
                parser.skip_whitespace();
                let rest = &parser.input[parser.pos..];
                if let Some(inner) = Self::parse_tactic_line(rest)? {
                    Ok(Some(Tactic::Repeat(Box::new(inner))))
                } else {
                    Err("Expected tactic after repeat".to_string())
                }
            }
            "have" => {
                parser.skip_whitespace();
                let name = parser
                    .parse_ident()
                    .ok_or_else(|| "Expected identifier after have".to_string())?;
                parser.skip_whitespace();
                if !parser.consume_if(":") {
                    return Err("Expected ':' after have identifier".to_string());
                }
                let prop_str = &parser.input[parser.pos..];
                let prop = Parser::parse_expr_str(prop_str)?;
                Ok(Some(Tactic::Have { name, prop }))
            }
            "cases" => {
                parser.skip_whitespace();
                let var = parser
                    .parse_ident()
                    .ok_or_else(|| "Expected variable after cases".to_string())?;
                let mut names = Vec::new();
                parser.skip_whitespace();
                if parser.consume_if("with") {
                    while let Some(id) = parser.parse_ident() {
                        names.push(id);
                    }
                }
                Ok(Some(Tactic::Cases { var, names }))
            }
            other => Err(format!("Unknown tactic: '{}'", other)),
        }
    }
}
