use crate::ast::{Expr, Statement, UDF};

/// Complete query pipeline.
///
/// Represents a full query from UDF definitions to final output.
#[derive(Debug, Clone)]
pub struct Query {
    /// User-defined functions
    pub udfs: Vec<UDF>,
    
    /// Pipeline statements
    pub statements: Vec<Statement>,
    
    /// Optional output expression (defaults to root if None)
    pub output: Option<Expr>,
}
