#[derive(Debug, Clone, PartialEq)]
pub enum Token {
    // Literals
    /// Integer or floating-point number
    ///
    /// # Examples
    /// ```text
    /// 42
    /// 3.14
    /// -1.0
    /// ```
    Float(f64),

    /// Integer
    ///
    /// # Examples
    /// ```text
    /// 42
    /// 314
    /// -10
    /// ```
    Integer(i64),
    
    /// String literal enclosed in double quotes
    ///
    /// # Examples
    /// ```text
    /// "hello"
    /// "item #1"
    /// ```
    String(String),
    
    /// Boolean values
    ///
    /// # Examples
    /// ```text
    /// true
    /// false
    /// ```
    Boolean(bool),
    
    /// Null value
    Null,

    /// Environment variable reference ($VARNAME)
    ///
    /// Follows bash convention - no brackets needed.
    ///
    /// # Examples
    /// ```text
    /// $HOME
    /// $API_KEY
    /// $threshold
    /// ```
    EnvVar(String),

    // Identifiers and References
    /// Field name or variable identifier
    ///
    /// Must start with letter or underscore, followed by letters, digits, or underscores.
    ///
    /// # Examples
    /// ```text
    /// user
    /// item_count
    /// _internal
    /// ```
    Identifier(String),
    
    /// Root document reference
    ///
    /// Always refers to the top-level document.
    ///
    /// # Examples
    /// ```text
    /// $
    /// $[field]
    /// ```
    Dollar,
    
    /// At-sign prefix for scope references, lambda params, or UDF args
    ///
    /// # Context-dependent meanings:
    /// - `@name` - Scope reference
    /// - `@` - Lambda parameter or current item
    /// - `@1` - UDF argument 1
    ///
    /// # Examples
    /// ```text
    /// @items := $[items]
    /// .any(@[price] > 100)
    /// &discount,2 := ~(@1 := @1 * (1 - @2))
    /// ```
    At,
    
    /// Ampersand prefix for user-defined functions
    ///
    /// # Examples
    /// ```text
    /// &expensive,1 := ?(@1[price] > 100)
    /// ?($[items].any(&expensive[@]))
    /// ```
    Ampersand,
    
    // Operators
    /// Filter operator
    ///
    /// Used to keep or discard records based on conditions.
    ///
    /// # Examples
    /// ```text
    /// ?($[status] == "active")
    /// ~($[items] := ?(@[price] > 100))
    /// ```
    Question,
    
    /// Transform operator
    ///
    /// Used to modify field values.
    ///
    /// # Examples
    /// ```text
    /// ~($[price] := $[price] * 1.1)
    /// ```
    Tilde,
    
    /// Output operator
    ///
    /// Specifies what to return from the query.
    ///
    /// # Examples
    /// ```text
    /// !($)
    /// !($[items])
    /// !({"total": $[total]})
    /// ```
    Exclamation,
    
    /// Assignment operator
    ///
    /// Used within transforms and scope definitions.
    ///
    /// # Examples
    /// ```text
    /// ~($[field] := value)
    /// @items := $[items]
    /// ```
    ColonEqual,
    
    /// Pipeline operator
    ///
    /// Chains operations together.
    ///
    /// # Examples
    /// ```text
    /// $ | ?(...) | ~(...) | !(...)
    /// ```
    Pipe,
    
    // Comparison
    /// Equality operator
    EqEq,
    
    /// Inequality operator
    NotEq,
    
    /// Less than
    Lt,
    
    /// Greater than
    Gt,
    
    /// Less than or equal
    LtEq,
    
    /// Greater than or equal
    GtEq,
    
    // Arithmetic
    /// Addition or string concatenation
    Plus,
    
    /// Subtraction
    Minus,
    
    /// Multiplication
    Star,
    
    /// Division
    Slash,
    
    /// Modulo
    Percent,
    
    // Logical
    /// Logical AND (word, not symbol)
    ///
    /// # Examples
    /// ```text
    /// $[age] > 18 and $[verified] == true
    /// ```
    And,
    
    /// Logical OR (word, not symbol)
    ///
    /// # Examples
    /// ```text
    /// $[role] == "admin" or $[role] == "mod"
    /// ```
    Or,
    
    // Delimiters
    /// Left bracket for accessors
    LBracket,
    
    /// Right bracket
    RBracket,
    
    /// Left parenthesis for grouping or function calls
    LParen,
    
    /// Right parenthesis
    RParen,
    
    /// Left brace for object literals
    LBrace,
    
    /// Right brace
    RBrace,
    
    /// Dot for method calls or field access
    Dot,
    
    /// Comma for separating arguments or array elements
    Comma,
    
    /// Colon for object literal key-value pairs
    Colon,
    
    /// End of file
    Eof,
}
