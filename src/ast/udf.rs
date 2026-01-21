use crate::ast::Statement;

/// User-defined function.
///
/// Functions can encapsulate filters, transforms, or computed values.
#[derive(Debug, Clone)]
pub struct UDF {
    /// Function name
    pub name: String,
    
    /// Number of arguments
    pub arity: usize,
    
    /// Function body (pre-parsed AST)
    pub body: Statement,
}
