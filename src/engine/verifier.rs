use crate::engine::environment::Environment;
use crate::engine::parser::Parser;
use crate::engine::proof_state::ProofState;
use crate::engine::tactics::TacticEngine;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LineDiagnostic {
    pub line_number: usize,
    pub raw_line: String,
    pub state_after: ProofState,
    pub info_message: Option<String>,
    pub error_message: Option<String>,
    pub is_sorry: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VerificationResult {
    pub is_complete: bool,
    pub used_sorry: bool,
    pub step_count: usize,
    pub diagnostics: Vec<LineDiagnostic>,
    pub final_state: ProofState,
    pub first_error: Option<String>,
}

pub struct Verifier;

impl Verifier {
    /// Verifies a full multi-line Lean 4 proof script against the initial proof state and environment.
    pub fn verify_script(
        initial_state: &ProofState,
        script: &str,
        env: &Environment,
    ) -> VerificationResult {
        let mut curr_state = initial_state.clone();
        let mut diagnostics = Vec::new();
        let mut used_sorry = false;
        let mut step_count = 0;
        let mut first_error = None;

        let lines: Vec<&str> = script.lines().collect();

        for (idx, line) in lines.iter().enumerate() {
            let line_number = idx + 1;
            let trimmed = line.trim();

            if trimmed.is_empty() || trimmed.starts_with("--") {
                diagnostics.push(LineDiagnostic {
                    line_number,
                    raw_line: line.to_string(),
                    state_after: curr_state.clone(),
                    info_message: None,
                    error_message: None,
                    is_sorry: false,
                });
                continue;
            }

            match Parser::parse_tactic_line(trimmed) {
                Ok(Some(tactic)) => {
                    let is_sorry = matches!(tactic, crate::engine::parser::Tactic::Sorry);
                    if is_sorry {
                        used_sorry = true;
                    }

                    match TacticEngine::apply_tactic(&tactic, &mut curr_state, env) {
                        Ok(msg) => {
                            step_count += 1;
                            diagnostics.push(LineDiagnostic {
                                line_number,
                                raw_line: line.to_string(),
                                state_after: curr_state.clone(),
                                info_message: Some(msg),
                                error_message: None,
                                is_sorry,
                            });
                        }
                        Err(err) => {
                            if first_error.is_none() {
                                first_error = Some(format!("Line {}: {}", line_number, err));
                            }
                            diagnostics.push(LineDiagnostic {
                                line_number,
                                raw_line: line.to_string(),
                                state_after: curr_state.clone(),
                                info_message: None,
                                error_message: Some(err),
                                is_sorry,
                            });
                            // Stop further execution on error
                            break;
                        }
                    }
                }
                Ok(None) => {
                    diagnostics.push(LineDiagnostic {
                        line_number,
                        raw_line: line.to_string(),
                        state_after: curr_state.clone(),
                        info_message: None,
                        error_message: None,
                        is_sorry: false,
                    });
                }
                Err(err) => {
                    let err_msg = format!("Parse error: {}", err);
                    if first_error.is_none() {
                        first_error = Some(format!("Line {}: {}", line_number, err_msg));
                    }
                    diagnostics.push(LineDiagnostic {
                        line_number,
                        raw_line: line.to_string(),
                        state_after: curr_state.clone(),
                        info_message: None,
                        error_message: Some(err_msg),
                        is_sorry: false,
                    });
                    break;
                }
            }
        }

        let is_complete = curr_state.is_solved() && !used_sorry && first_error.is_none();

        VerificationResult {
            is_complete,
            used_sorry,
            step_count,
            diagnostics,
            final_state: curr_state,
            first_error,
        }
    }

    /// Retrieve the proof state at a specific 1-based line number (for Infoview cursor following).
    pub fn state_at_line(
        result: &VerificationResult,
        line_number: usize,
        initial_state: &ProofState,
    ) -> ProofState {
        for diag in &result.diagnostics {
            if diag.line_number == line_number {
                return diag.state_after.clone();
            }
        }
        // If cursor is beyond evaluated lines or before, return closest
        if let Some(last) = result.diagnostics.last() {
            if line_number >= last.line_number {
                return last.state_after.clone();
            }
        }
        initial_state.clone()
    }
}
