//! CLI support for clove-lang
//!
//! Provides programmatic access to clove CLI functionality for embedding
//! in other tools (like checkmate).

mod check;
mod convert;
mod docs;
mod onboard;

pub use check::{execute_check, CheckOptions, CheckResult};
pub use convert::{clove_to_json, json_to_clove};
pub use docs::{get_doc_category, get_docs_overview, DocCategory};
pub use onboard::get_onboarding_content;

use std::io;

/// Errors that can occur during CLI operations
#[derive(Debug)]
pub enum CliError {
    /// Parser error
    Parse(crate::ParseError),
    /// Evaluation error
    Eval(crate::EvalError),
    /// JSON parsing error
    Json(serde_json::Error),
    /// IO error
    Io(io::Error),
    /// No input provided
    NoInput,
    /// Unknown documentation category
    UnknownCategory(String),
}

impl std::fmt::Display for CliError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            CliError::Parse(e) => write!(f, "Parse error: {}", e),
            CliError::Eval(e) => write!(f, "Evaluation error: {}", e),
            CliError::Json(e) => write!(f, "Invalid JSON: {}", e),
            CliError::Io(e) => write!(f, "IO error: {}", e),
            CliError::NoInput => write!(f, "No input provided. Use --input or pipe JSON to stdin."),
            CliError::UnknownCategory(c) => {
                write!(f, "Unknown category: '{}'\nRun 'clove docs' to see available categories.", c)
            }
        }
    }
}

impl std::error::Error for CliError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            CliError::Parse(e) => Some(e),
            CliError::Eval(e) => Some(e),
            CliError::Json(e) => Some(e),
            CliError::Io(e) => Some(e),
            _ => None,
        }
    }
}

impl From<crate::ParseError> for CliError {
    fn from(e: crate::ParseError) -> Self {
        CliError::Parse(e)
    }
}

impl From<crate::EvalError> for CliError {
    fn from(e: crate::EvalError) -> Self {
        CliError::Eval(e)
    }
}

impl From<serde_json::Error> for CliError {
    fn from(e: serde_json::Error) -> Self {
        CliError::Json(e)
    }
}

impl From<io::Error> for CliError {
    fn from(e: io::Error) -> Self {
        CliError::Io(e)
    }
}
