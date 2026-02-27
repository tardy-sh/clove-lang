use crate::ast::Expr;

/// Pipeline statement.
///
/// A pipeline consists of a sequence of statements that process data.
#[derive(Debug, Clone)]
pub enum Statement {
    /// Scope definition
    ///
    /// Creates a shorthand reference for a path.
    ///
    /// # Example
    /// ```text
    /// @items := $[items]
    /// ```
    ScopeDefinition {
        name: String,
        path: Expr,
    },
    
    /// Existence check
    ///
    /// # Example
    /// ```text
    /// $[items][?]
    /// ```
    ExistenceCheck(Expr),
    
    /// Filter operation
    ///
    /// Keeps or discards records based on a condition.
    ///
    /// # Example
    /// ```text
    /// ?($[status] == "active")
    /// ```
    Filter(Expr),
    
    /// Transform operation
    ///
    /// Modifies field values.
    ///
    /// # Example
    /// ```text
    /// ~($[price] := $[price] * 1.1)
    /// ```
    Transform {
        target: Expr,
        value: Expr,
    },
    
    /// Field deletion
    ///
    /// Removes the specified field from the document. Silent no-op if nonexistent.
    ///
    /// # Example
    /// ```text
    /// $ | -($[password])
    /// $ | -($[user][token])
    /// ```
    Delete(Expr),

    /// Plain access (passes through the value)
    ///
    /// # Example
    /// ```text
    /// $[items]
    /// ```
    Access(Expr),
}
