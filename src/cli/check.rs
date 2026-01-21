//! Execute clove queries against JSON input

use crate::{Evaluator, Lexer, Parser};
use super::{CliError, json_to_clove, clove_to_json};

/// Options for the check command
#[derive(Debug, Clone, Default)]
pub struct CheckOptions {
    /// The Clove query to execute
    pub query: String,
    /// JSON input string
    pub input: Option<String>,
    /// Pretty-print the output
    pub pretty: bool,
    /// Only validate syntax, don't execute
    pub syntax_only: bool,
}

/// Result of a check operation
#[derive(Debug)]
pub enum CheckResult {
    /// Syntax validation passed
    SyntaxValid,
    /// Query executed successfully with JSON output
    Success(serde_json::Value),
}

/// Detect whether a query string is a pipeline query or simple expression
fn is_pipeline_query(query: &str) -> bool {
    // A query uses single | for piping, but not || for logical OR
    query.contains(" | ") || (query.contains('|') && !query.contains("||"))
}

/// Execute a clove check operation
pub fn execute_check(options: &CheckOptions) -> Result<CheckResult, CliError> {
    let query = &options.query;
    let is_query = is_pipeline_query(query);

    let lexer = Lexer::new(query);
    let mut parser = Parser::new(lexer).map_err(CliError::Parse)?;

    if options.syntax_only {
        let result = if is_query {
            parser.parse_query().map(|_| ())
        } else {
            parser.parse().map(|_| ())
        };

        return match result {
            Ok(()) => Ok(CheckResult::SyntaxValid),
            Err(e) => Err(CliError::Parse(e)),
        };
    }

    let json_str = options.input.as_ref().ok_or(CliError::NoInput)?;

    let json_value: serde_json::Value =
        serde_json::from_str(json_str).map_err(CliError::Json)?;

    let input_value = json_to_clove(json_value);

    let mut evaluator = Evaluator::new();
    let result = if is_query {
        let q = parser.parse_query().map_err(CliError::Parse)?;
        evaluator.eval_query(&q, input_value)
    } else {
        let expr = parser.parse().map_err(CliError::Parse)?;
        evaluator.eval_expression(&expr, input_value)
    }
    .map_err(CliError::Eval)?;

    let output = clove_to_json(result);
    Ok(CheckResult::Success(output))
}
