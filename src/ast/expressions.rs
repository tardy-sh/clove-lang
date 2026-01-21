use crate::ast::BinOp;

/// Abstract Syntax Tree node representing a parsed expression.
///
/// The AST is the internal representation of a query after parsing.
/// It captures the structure and meaning of the query for evaluation.
#[derive(Debug, Clone, PartialEq)]
pub enum Expr {
    // Literals
    /// Literal floating point number
    ///
    /// # Example
    /// ```text
    /// 42.0
    /// ```
    Float(f64),

    /// Literal integer
    ///
    /// # Example
    /// ```text
    /// 42
    /// ```
    Integer(i64),
    
    /// String literal
    ///
    /// # Example
    /// ```text
    /// "hello"
    /// ```
    String(String),
    
    /// Boolean literal
    Boolean(bool),
    
    /// Null literal
    Null,
    
    // References
    /// Root document reference (`$`)
    Root,
    
    /// Scope reference (`@name`)
    ///
    /// A user-defined shorthand for a path.
    ScopeRef(String),
    
    /// Lambda parameter (`@`)
    ///
    /// Refers to the current item in a lambda or transform context.
    LambdaParam,
    
    /// UDF argument reference (`@1`, `@2`, etc.)
    ArgRef(usize),


    /// Environment variable reference
    ///
    /// # Examples
    /// ```text
    /// $HOME          // EnvVar("HOME")
    /// $API_KEY       // EnvVar("API_KEY")
    /// $min_price     // EnvVar("min_price")
    /// ```
    EnvVar(String),

    // Keys - field/property names in access expressions
    /// Field or property name used in access expressions.
    /// Transformed from Token::Identifier during parsing.
    /// Only appears as the `key` in `Expr::Access`.
    Key(String),
    
    // Access
    /// Field or index access
    ///
    /// # Examples
    /// ```text
    /// $[field]
    /// $[0]
    /// $[items][name]
    /// ```
    Access {
        object: Box<Expr>,
        key: Box<Expr>,
    },
    
    /// Existence check (`[?]`)
    ///
    /// Returns true if the value exists and is non-empty.
    ExistenceCheck(Box<Expr>),

    /// Filter expression 
    /// e.g.: `?(condition)`
    ///
    /// Used in transforms: ~($[items] := ?(@[price] > 100))
    Filter(Box<Expr>),

    // Operations
    /// Binary operation (arithmetic, comparison, logical)
    BinaryOp {
        op: BinOp,
        left: Box<Expr>,
        right: Box<Expr>,
    },
    
    /// Method call
    ///
    /// # Examples
    /// ```text
    /// $[items].any(@[price] > 100)
    /// $[numbers].sum()
    /// ```
    MethodCall {
        object: Box<Expr>,
        method: String,
        args: Vec<Expr>,
    },
    
    /// User-defined function call
    ///
    /// # Example
    /// ```text
    /// &expensive[@]
    /// ```
    UDFCall {
        name: String,
        args: Vec<Expr>,
    },
    
    // Object and Array Literals
    /// Object literal
    ///
    /// # Example
    /// ```text
    /// {"name": $[name], "total": $[total]}
    /// ```
    Object(Vec<(String, Expr)>),
    
    /// Array literal
    ///
    /// # Example
    /// ```text
    /// [$[item1], $[item2]]
    /// ```
    Array(Vec<Expr>),
}
