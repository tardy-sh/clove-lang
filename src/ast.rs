//! # Clove Query Language - Abstract Syntax Tree
//!
//! This module defines the Abstract Syntax Tree (AST) for the Clove query language,
//! a powerful, explicit query language for JSON documents that emphasizes clarity
//! and composability.
//!
//! ## Architecture Overview
//!
//! The AST module is organized into focused submodules:
//!
//! - **[tokens]** - Lexical tokens produced by the lexer
//! - **[expressions]** - Expression nodes (values, access, operations, literals)
//! - **[operators]** - Binary operators (comparison, arithmetic, logical)
//! - **[statements]** - Pipeline statements (filter, transform, scope definition)
//! - **[query]** - Complete query structure with UDFs and output
//! - **[udf]** - User-defined function definitions
//!
//! ## Quick Start
//!
//! ```text
//! $ | ?($[status] == "active") | !($[items])
//! ```
//!
//! This query filters for active records and returns their items.
//!
//! ## Core Concepts
//!
//! ### Pipeline Structure
//!
//! Every query is a pipeline starting with `$` (root) and chaining operations:
//!
//! ```text
//! $ | operation | operation | ... | !(output)
//! ```
//!
//! ### The Three Operations
//!
//! - **Filter** `?()` - Keep or discard records based on conditions
//! - **Transform** `~()` - Modify field values with assignments
//! - **Output** `!()` - Specify return value (defaults to root)
//!
//! ### Numeric Key Behavior
//!
//! The language intelligently handles numeric keys based on context:
//!
//! - **Integer keys** on arrays → Array index access (supports negative indices)
//! - **Integer keys** on objects → String key access (e.g., `[0]` → `"0"`)
//! - **Float keys** on objects → String key access (e.g., `[1.5]` → `"1.5"`)
//!
//! ### Type System
//!
//! Values support all JSON types (null, boolean, integer, float, string, array, object)
//! with intelligent arithmetic that preserves integer types when results are whole numbers.
//!
//! ## Examples
//!
//! ### Simple Filter
//!
//! ```text
//! $ | ?($[price] > 100) | !($)
//! ```
//!
//! ### Array Filtering with Transform
//!
//! ```text
//! $ | ~($[items] := ?(@[status] == "ok")) | !($)
//! ```
//!
//! ### Array Mapping with Lambda
//!
//! ```text
//! $ | ~($[prices] := @[price] * 1.1) | !($)
//! ```
//!
//! ### Scope References
//!
//! ```text
//! $ | @user := $[user] | ?(@user[age] >= 18) | !(@user)
//! ```
//!
//! ### Negative Array Indices
//!
//! ```text
//! $ | !($[items][-1])  // Last element
//! ```
pub mod tokens;
pub mod expressions;
pub mod operators;
pub mod statements;
pub mod query;
pub mod udf;

pub use tokens::Token;
pub use expressions::Expr;
pub use operators::{BinOp};
pub use statements::Statement;
pub use query::Query;
pub use udf::UDF;

// #[derive(Debug, Clone)]
// pub enum Expr2 {
//     Variable(String),
//     Access { object: Box<Expr>, key: String },
// }
