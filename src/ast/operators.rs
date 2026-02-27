/// Binary operators.
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum BinOp {
    // Comparison
    /// Equal (`==`)
    Equal,
    /// Not equal (`!=`)
    NotEqual,
    /// Less than (`<`)
    LessThan,
    /// Greater than (`>`)
    GreaterThan,
    /// Less than or equal (`<=`)
    LessEqual,
    /// Greater than or equal (`>=`)
    GreaterEqual,
    
    // Arithmetic
    /// Addition or string concatenation (`+`)
    Add,
    /// Subtraction (`-`)
    Subtract,
    /// Multiplication (`*`)
    Multiply,
    /// Division (`/`)
    Divide,
    /// Modulo (`%`)
    Modulo,
    
    // Logical
    /// Logical AND (`and`)
    And,
    /// Logical OR (`or`)
    Or,

    // Null-coalescing
    /// Null-coalescing (`??`)
    NullCoalesce,
}
