use std::{collections::HashMap, env};

use rust_decimal::{Decimal, prelude::FromPrimitive, prelude::ToPrimitive};

use crate::{
    ast::{BinOp, Expr, Query, Statement},
    transform::{PathSegment, TransformType, determine_transform_type, extract_path},
    value::Value,
};

/// Evaluation context holding both root and lambda contexts
#[derive(Debug, Clone)]
pub struct EvalContext {
    /// The root document (referred to by `$`)
    pub root: Value,
    /// The current lambda item (what @ refers to), if in lambda function
    pub lambda: Option<Value>,
}

impl EvalContext {
    pub fn new(root: Value) -> Self {
        EvalContext { root, lambda: None }
    }

    /// Create a new context with lambda item
    pub fn with_lambda(&self, lambda: Value) -> Self {
        EvalContext {
            root: self.root.clone(),
            lambda: Some(lambda),
        }
    }
}

/// The main query evaluator.
///
/// Executes parsed queries against JSON documents, maintaining scope references
/// and handling transformations.
#[derive(Default)]
pub struct Evaluator {
    /// Named scope references defined during query execution (@name := ...)
    scopes: HashMap<String, Value>,
}

/// Errors that can occur during query evaluation.
#[derive(Debug, Clone)]
pub enum EvalError {
    /// Type mismatch or invalid operation for the given type
    TypeError(String),

    /// Invalid field access or array index
    AccessError(String),

    /// Reference to undefined scope (@name not defined)
    UndefinedScope(String),

    /// Reference to undefined environment variable ($VARNAME)
    UndefinedEnvVar(String),

    /// Division by zero
    DivisionByZero,
}

impl std::fmt::Display for EvalError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            EvalError::TypeError(msg) => write!(f, "Type error: {}", msg),
            EvalError::AccessError(msg) => write!(f, "Access error: {}", msg),
            EvalError::UndefinedScope(name) => write!(f, "Undefined scope: @{} is not defined", name),
            EvalError::UndefinedEnvVar(name) => write!(f, "Undefined environment variable: ${}", name),
            EvalError::DivisionByZero => write!(f, "Division by zero"),
        }
    }
}

impl std::error::Error for EvalError {}

/// Returns a human-readable type name for a Value
fn type_name(v: &Value) -> &'static str {
    match v {
        Value::Null => "null",
        Value::Boolean(_) => "boolean",
        Value::Integer(_) => "integer",
        Value::Float(_) => "float",
        Value::String(_) => "string",
        Value::Array(_) => "array",
        Value::Object(_) => "object",
    }
}

impl Evaluator {
    /// Creates a new evaluator with empty scope references.
    pub fn new() -> Self {
        Self::default()
    }

    /// Evaluates a complete query against a JSON document.
    ///
    /// Executes the query pipeline statement by statement, threading the result
    /// through each transformation, and returns the final output.
    ///
    /// # Arguments
    ///
    /// * `query` - The parsed query to execute
    /// * `document` - The JSON document to query (becomes the root `$`)
    ///
    /// # Returns
    ///
    /// The result of the query's output expression, or the final transformed
    /// document if no explicit output is specified.
    ///
    /// # Examples
    ///
    /// ```
    /// use clove_lang::Evaluator;
    /// use clove_lang::parser::Parser;
    /// use clove_lang::lexer::Lexer;
    /// use clove_lang::Value;
    /// use std::collections::HashMap;
    ///
    /// let mut doc = HashMap::new();
    /// doc.insert("price".to_string(), Value::Integer(100));
    ///
    /// let query_str = "$ | ?($[price] > 50)";
    /// let lexer = Lexer::new(query_str);
    /// let mut parser = Parser::new(lexer).unwrap();
    /// let query = parser.parse_query().unwrap();
    ///
    /// let mut evaluator = Evaluator::new();
    /// let result = evaluator.eval_query(&query, Value::Object(doc)).unwrap();
    /// // Returns the document because price > 50
    /// ```
    pub fn eval_query(&mut self, query: &Query, document: Value) -> Result<Value, EvalError> {
        let mut current = document;

        for stmt in &query.statements {
            let ctx = EvalContext::new(current);
            current = self.eval_statement(stmt, &ctx)?;
        }

        match &query.output {
            Some(expr) => {
                let ctx = EvalContext::new(current);

                self.eval_expr(expr, &ctx)
            }
            None => Ok(current),
        }
    }

    /// Evaluates a single expression against a JSON document.
    ///
    /// This is a convenience method for evaluating standalone expressions
    /// without a full query pipeline.
    ///
    /// # Arguments
    ///
    /// * `expr` - The expression to evaluate
    /// * `document` - The JSON document (becomes the root `$`)
    ///
    /// # Examples
    ///
    /// ```
    /// use clove_lang::{Evaluator, Expr, Value};
    ///
    /// let mut evaluator = Evaluator::new();
    /// let expr = Expr::Root;
    /// let doc = Value::Integer(42);
    ///
    /// let result = evaluator.eval_expression(&expr, doc).unwrap();
    /// assert_eq!(result, Value::Integer(42));
    /// ```
    pub fn eval_expression(&mut self, expr: &Expr, document: Value) -> Result<Value, EvalError> {
        let context = EvalContext::new(document);
        self.eval_expr(expr, &context)
    }

    fn eval_statement(&mut self, stmt: &Statement, ctx: &EvalContext) -> Result<Value, EvalError> {
        match stmt {
            Statement::Filter(condition) => {
                let result = self.eval_expr(condition, ctx)?;
                if result.as_bool() {
                    Ok(ctx.root.clone())
                } else {
                    Ok(Value::Null)
                }
            }
            Statement::Transform { target, value } => self.apply_transform(ctx, target, value),
            Statement::ScopeDefinition { name, path } => {
                let value = self.eval_expr(path, ctx)?;
                self.scopes.insert(name.clone(), value);
                Ok(ctx.root.clone())
            }
            Statement::Delete(path_expr) => {
                let path = extract_path(path_expr)?;
                if path.is_empty() {
                    return Ok(ctx.root.clone());
                }
                let mut result = ctx.root.clone();
                self.delete_field(&mut result, &path);
                Ok(result)
            }
            Statement::Access(expr) => self.eval_expr(expr, ctx),
            Statement::ExistenceCheck(_expr) => unreachable!(),
        }
    }

    fn eval_expr(&self, expr: &Expr, context: &EvalContext) -> Result<Value, EvalError> {
        match expr {
            Expr::Float(n) => Ok(Value::Float(*n)),
            Expr::Integer(n) => Ok(Value::Integer(*n)),
            Expr::String(s) => Ok(Value::String(s.clone())),
            Expr::Boolean(b) => Ok(Value::Boolean(*b)),
            Expr::Null => Ok(Value::Null),
            Expr::Root => Ok(context.root.clone()),
            Expr::EnvVar(name) => match env::var(name) {
                Ok(val) => Ok(Value::String(val)),
                Err(_) => Err(EvalError::UndefinedEnvVar(name.to_string())),
            },
            Expr::ScopeRef(name) => self
                .scopes
                .get(name)
                .cloned()
                .ok_or_else(|| EvalError::UndefinedScope(name.clone())),
            Expr::LambdaParam => {
                // In lambda context, `@` refers to current item.
                // This is passed as context already.
                match &context.lambda {
                    Some(lambda) => Ok(lambda.clone()),
                    None => Ok(context.root.clone()),
                }
            }
            Expr::Access { object, key } => {
                let obj_value = self.eval_expr(object, context)?;
                let key_value = self.eval_expr(key, context)?;
                self.apply_access(&obj_value, &key_value)
            }
            Expr::BinaryOp { op, left, right } => {
                if *op == BinOp::NullCoalesce {
                    let left_val = self.eval_expr(left, context)?;
                    if left_val == Value::Null {
                        self.eval_expr(right, context)
                    } else {
                        Ok(left_val)
                    }
                } else {
                    let left_val = self.eval_expr(left, context)?;
                    let right_val = self.eval_expr(right, context)?;
                    self.apply_binop(*op, &left_val, &right_val)
                }
            }
            Expr::Object(items) => {
                let mut map = HashMap::new();
                for (key, expr) in items {
                    let value = self.eval_expr(expr, context)?;
                    map.insert(key.clone(), value);
                }
                Ok(Value::Object(map))
            }
            Expr::Array(exprs) => {
                let mut arr = Vec::new();
                for expr in exprs {
                    arr.push(self.eval_expr(expr, context)?);
                }
                Ok(Value::Array(arr))
            }
            Expr::Filter(expr) => self.eval_expr(expr, context),
            Expr::MethodCall {
                object,
                method,
                args,
            } => {
                let obj_value = self.eval_expr(object, context)?;
                self.eval_method_call(&obj_value, method, args, context)
            }
            Expr::UDFCall { name: _, args: _ } => {
                // Next up
                todo!("UDF execution - needs UDF registry")
            }
            Expr::ArgRef(n) => Err(EvalError::TypeError(format!(
                "Argument reference @{} can only be used within UDF definitions",
                n
            ))),
            Expr::ExistenceCheck(expr) => {
                let value = self.eval_expr(expr, context)?;
                let exists = match value {
                    Value::Null => false,
                    Value::Array(ref arr) => !arr.is_empty(),
                    Value::Object(ref obj) => !obj.is_empty(),
                    Value::String(ref s) => !s.is_empty(),
                    _ => true,
                };
                Ok(Value::Boolean(exists))
            }
            Expr::Key(name) => Ok(Value::String(name.clone())),
        }
    }

    fn apply_access(&self, object: &Value, key: &Value) -> Result<Value, EvalError> {
        match (object, key) {
            (Value::Object(map), Value::Float(k)) => {
                Ok(map.get(&k.to_string()).cloned().unwrap_or(Value::Null))
            }
            (Value::Object(map), Value::Boolean(k)) => {
                Ok(map.get(&k.to_string()).cloned().unwrap_or(Value::Null))
            }
            (Value::Object(map), Value::Integer(k)) => {
                Ok(map.get(&k.to_string()).cloned().unwrap_or(Value::Null))
            }
            (Value::Object(map), Value::String(k)) => {
                Ok(map.get(k).cloned().unwrap_or(Value::Null))
            }
            (Value::Array(arr), Value::Integer(n)) => {
                let index = if *n < 0 {
                    // Negative index: count from end (-1 = last, -2 = second to last)
                    let abs_idx = (-*n) as usize;
                    if abs_idx > arr.len() {
                        return Ok(Value::Null);
                    }
                    arr.len() - abs_idx
                } else {
                    *n as usize
                };
                Ok(arr.get(index).cloned().unwrap_or(Value::Null))
            }
            (Value::Array(_), Value::String(k)) => Err(EvalError::TypeError(format!(
                "Cannot use string key '{}' on array; use integer index instead",
                k
            ))),
            (v, Value::Integer(_)) => Err(EvalError::TypeError(format!(
                "Cannot use integer index on {}; only arrays support integer indexing",
                type_name(v)
            ))),
            _ => Err(EvalError::TypeError(format!(
                "Cannot access {} with {} key",
                type_name(object),
                type_name(key)
            ))),
        }
    }

    fn apply_binop(&self, op: BinOp, left: &Value, right: &Value) -> Result<Value, EvalError> {
        match op {
            BinOp::Add => match (left, right) {
                (Value::Float(a), Value::Float(b)) => Ok(Value::Float(a + b)),
                (Value::Integer(a), Value::Integer(b)) => Ok(Value::Integer(a + b)),
                (Value::Integer(a), Value::Float(b)) => {
                    if let Some(ad) = Decimal::from_i64(*a)
                        && let Some(bd) = Decimal::from_f64(*b)
                    {
                        let rd = ad + bd;
                        if rd.is_integer() &&
                            let Some(r) = rd.to_i64() {
                            return Ok(Value::Integer(r));
                        } else if let Some(r) = rd.to_f64() {
                            return Ok(Value::Float(r));
                        }
                    }
                    let res = *a as f64 + b;
                    Ok(Value::Float(res))
                }
                (Value::Float(a), Value::Integer(b)) => {
                    if let Some(ad) = Decimal::from_f64(*a)
                        && let Some(bd) = Decimal::from_i64(*b)
                    {
                        let rd = ad + bd;
                        if rd.is_integer() &&
                            let Some(r) = rd.to_i64() {
                            return Ok(Value::Integer(r));
                        } else if let Some(r) = rd.to_f64() {
                            return Ok(Value::Float(r));
                        }
                    }
                    let res = *a + *b as f64;
                    Ok(Value::Float(res))
                }
                (Value::String(a), Value::String(b)) => Ok(Value::String(format!("{}{}", a, b))),
                (a, b) => Err(EvalError::TypeError(format!(
                    "Cannot add {} and {}",
                    type_name(a), type_name(b)
                ))),
            },
            BinOp::Subtract => match (left, right) {
                (Value::Float(a), Value::Float(b)) => Ok(Value::Float(a - b)),
                (Value::Integer(a), Value::Integer(b)) => Ok(Value::Integer(a - b)),
                (Value::Integer(a), Value::Float(b)) => {
                    if let Some(ad) = Decimal::from_i64(*a)
                        && let Some(bd) = Decimal::from_f64(*b)
                    {
                        let rd = ad - bd;
                        if rd.is_integer() &&
                            let Some(r) = rd.to_i64() {
                            return Ok(Value::Integer(r));
                        } else if let Some(r) = rd.to_f64() {
                            return Ok(Value::Float(r));
                        }
                    }
                    let res = *a as f64 - b;
                    Ok(Value::Float(res))
                }
                (Value::Float(a), Value::Integer(b)) => {
                    if let Some(ad) = Decimal::from_f64(*a)
                        && let Some(bd) = Decimal::from_i64(*b)
                    {
                        let rd = ad - bd;
                        if rd.is_integer() &&
                            let Some(r) = rd.to_i64() {
                            return Ok(Value::Integer(r));
                        } else if let Some(r) = rd.to_f64() {
                            return Ok(Value::Float(r));
                        }
                    }
                    let res = *a - *b as f64;
                    Ok(Value::Float(res))
                }
                (a, b) => Err(EvalError::TypeError(format!(
                    "Cannot subtract {} from {}",
                    type_name(b), type_name(a)
                ))),
            },

            BinOp::Multiply => match (left, right) {
                (Value::Float(a), Value::Float(b)) => Ok(Value::Float(a * b)),
                (Value::Integer(a), Value::Integer(b)) => Ok(Value::Integer(a * b)),
                (Value::Integer(a), Value::Float(b)) => {
                    if let Some(ad) = Decimal::from_i64(*a)
                        && let Some(bd) = Decimal::from_f64(*b)
                    {
                        let rd = ad * bd;
                        if rd.is_integer() &&
                            let Some(r) = rd.to_i64() {
                            return Ok(Value::Integer(r));
                        } else if let Some(r) = rd.to_f64() {
                            return Ok(Value::Float(r));
                        }
                    }
                    let res = *a as f64 * b;
                    Ok(Value::Float(res))
                }
                (Value::Float(a), Value::Integer(b)) => {
                    if let Some(ad) = Decimal::from_f64(*a)
                        && let Some(bd) = Decimal::from_i64(*b)
                    {
                        let rd = ad * bd;
                        if rd.is_integer() &&
                            let Some(r) = rd.to_i64() {
                            return Ok(Value::Integer(r));
                        } else if let Some(r) = rd.to_f64() {
                            return Ok(Value::Float(r));
                        }
                    }
                    let res = *a * *b as f64;
                    Ok(Value::Float(res))
                }
                (a, b) => Err(EvalError::TypeError(format!(
                    "Cannot multiply {} by {}",
                    type_name(a), type_name(b)
                ))),
            },
            BinOp::Divide => match (left, right) {
                (Value::Float(a), Value::Float(b)) => Ok(Value::Float(a / b)),
                (Value::Integer(a), Value::Integer(b)) => {
                    // Check if division is exact; if not, return Float
                    if *a % *b == 0 {
                        Ok(Value::Integer(a / b))
                    } else {
                        Ok(Value::Float(*a as f64 / *b as f64))
                    }
                }
                (Value::Integer(a), Value::Float(b)) => {
                    if let Some(ad) = Decimal::from_i64(*a)
                        && let Some(bd) = Decimal::from_f64(*b)
                    {
                        let rd = ad / bd;
                        if rd.is_integer() &&
                            let Some(r) = rd.to_i64() {
                            return Ok(Value::Integer(r));
                        } else if let Some(r) = rd.to_f64() {
                            return Ok(Value::Float(r));
                        }
                    }
                    let res = *a as f64 / b;
                    Ok(Value::Float(res))
                }
                (Value::Float(a), Value::Integer(b)) => {
                    if let Some(ad) = Decimal::from_f64(*a)
                        && let Some(bd) = Decimal::from_i64(*b)
                    {
                        let rd = ad / bd;
                        if rd.is_integer() &&
                            let Some(r) = rd.to_i64() {
                            return Ok(Value::Integer(r));
                        } else if let Some(r) = rd.to_f64() {
                            return Ok(Value::Float(r));
                        }
                    }
                    let res = *a / *b as f64;
                    Ok(Value::Float(res))
                }
                (a, b) => Err(EvalError::TypeError(format!(
                    "Cannot divide {} by {}",
                    type_name(a), type_name(b)
                ))),
            },
            BinOp::Modulo => match (left, right) {
                (Value::Float(a), Value::Float(b)) => Ok(Value::Float(a % b)),
                (Value::Integer(a), Value::Integer(b)) => Ok(Value::Integer(a % b)),
                (Value::Integer(a), Value::Float(b)) => {
                    if let Some(ad) = Decimal::from_i64(*a)
                        && let Some(bd) = Decimal::from_f64(*b)
                    {
                        let rd = ad % bd;
                        if rd.is_integer() &&
                            let Some(r) = rd.to_i64() {
                            return Ok(Value::Integer(r));
                        } else if let Some(r) = rd.to_f64() {
                            return Ok(Value::Float(r));
                        }
                    }
                    let res = *a as f64 % b;
                    Ok(Value::Float(res))
                }
                (Value::Float(a), Value::Integer(b)) => {
                    if let Some(ad) = Decimal::from_f64(*a)
                        && let Some(bd) = Decimal::from_i64(*b)
                    {
                        let rd = ad % bd;
                        if rd.is_integer() &&
                            let Some(r) = rd.to_i64() {
                            return Ok(Value::Integer(r));
                        } else if let Some(r) = rd.to_f64() {
                            return Ok(Value::Float(r));
                        }
                    }
                    let res = *a % *b as f64;
                    Ok(Value::Float(res))
                }
                (a, b) => Err(EvalError::TypeError(format!(
                    "Cannot compute modulo of {} by {}",
                    type_name(a), type_name(b)
                ))),
            },
            BinOp::Equal => Ok(Value::Boolean(left == right)),
            BinOp::NotEqual => Ok(Value::Boolean(left != right)),
            BinOp::LessThan => match (left, right) {
                (Value::Float(a), Value::Float(b)) => Ok(Value::Boolean(a < b)),
                (Value::Integer(a), Value::Integer(b)) => Ok(Value::Boolean(a < b)),
                (Value::Integer(a), Value::Float(b)) => Ok(Value::Boolean((*a as f64) < *b)),
                (Value::Float(a), Value::Integer(b)) => Ok(Value::Boolean(*a < (*b as f64))),
                (a, b) => Err(EvalError::TypeError(format!(
                    "Cannot compare {} < {} (comparison requires numeric types)",
                    type_name(a), type_name(b)
                ))),
            },
            BinOp::GreaterThan => match (left, right) {
                (Value::Float(a), Value::Float(b)) => Ok(Value::Boolean(a > b)),
                (Value::Integer(a), Value::Integer(b)) => Ok(Value::Boolean(a > b)),
                (Value::Integer(a), Value::Float(b)) => Ok(Value::Boolean(*a as f64 > *b)),
                (Value::Float(a), Value::Integer(b)) => Ok(Value::Boolean(*a > *b as f64)),
                (a, b) => Err(EvalError::TypeError(format!(
                    "Cannot compare {} > {} (comparison requires numeric types)",
                    type_name(a), type_name(b)
                ))),
            },
            BinOp::LessEqual => match (left, right) {
                (Value::Float(a), Value::Float(b)) => Ok(Value::Boolean(a <= b)),
                (Value::Integer(a), Value::Integer(b)) => Ok(Value::Boolean(a <= b)),
                (Value::Integer(a), Value::Float(b)) => Ok(Value::Boolean(*a as f64 <= *b)),
                (Value::Float(a), Value::Integer(b)) => Ok(Value::Boolean(*a <= *b as f64)),
                (a, b) => Err(EvalError::TypeError(format!(
                    "Cannot compare {} <= {} (comparison requires numeric types)",
                    type_name(a), type_name(b)
                ))),
            },
            BinOp::GreaterEqual => match (left, right) {
                (Value::Float(a), Value::Float(b)) => Ok(Value::Boolean(a >= b)),
                (Value::Integer(a), Value::Integer(b)) => Ok(Value::Boolean(a >= b)),
                (Value::Integer(a), Value::Float(b)) => Ok(Value::Boolean(*a as f64 >= *b)),
                (Value::Float(a), Value::Integer(b)) => Ok(Value::Boolean(*a >= *b as f64)),
                (a, b) => Err(EvalError::TypeError(format!(
                    "Cannot compare {} >= {} (comparison requires numeric types)",
                    type_name(a), type_name(b)
                ))),
            },
            BinOp::And => Ok(Value::Boolean(left.as_bool() && right.as_bool())),
            BinOp::Or => Ok(Value::Boolean(left.as_bool() || right.as_bool())),
            BinOp::NullCoalesce => unreachable!("NullCoalesce handled in eval_expr"),
        }
    }
    /// Remove a field at the given path. Silent no-op if path doesn't exist.
    fn delete_field(&self, current: &mut Value, path: &[PathSegment]) {
        if path.is_empty() {
            return;
        }

        if path.len() == 1 {
            // Remove the target key from parent
            match (&mut *current, &path[0]) {
                (Value::Object(map), PathSegment::Field(key)) => {
                    map.remove(key);
                }
                (Value::Array(arr), PathSegment::Index(idx)) => {
                    let len = arr.len();
                    let index = if *idx < 0 {
                        let abs = idx.unsigned_abs() as usize;
                        if abs > len { return; }
                        len - abs
                    } else {
                        *idx as usize
                    };
                    if index < len {
                        arr.remove(index);
                    }
                }
                _ => {} // no-op
            }
            return;
        }

        // Navigate to parent, then recurse
        match (&mut *current, &path[0]) {
            (Value::Object(map), PathSegment::Field(key)) => {
                if let Some(child) = map.get_mut(key) {
                    self.delete_field(child, &path[1..]);
                }
                // missing intermediate → no-op
            }
            (Value::Array(arr), PathSegment::Index(idx)) => {
                let len = arr.len();
                let index = if *idx < 0 {
                    let abs = idx.unsigned_abs() as usize;
                    if abs > len { return; }
                    len - abs
                } else {
                    *idx as usize
                };
                if let Some(child) = arr.get_mut(index) {
                    self.delete_field(child, &path[1..]);
                }
            }
            _ => {} // no-op
        }
    }

    fn apply_transform(
        &mut self,
        ctx: &EvalContext,
        target: &Expr,
        value_expr: &Expr,
    ) -> Result<Value, EvalError> {
        let path = extract_path(target)?;

        // if path.is_empty() {
        //     return Err(EvalError::TypeError(
        //         "Cannot transform root directly. Use a specific field path or use literal object for output".to_string(),
        //     ));
        // }

        let transform_type = determine_transform_type(value_expr);

        let mut result = ctx.root.clone();

        self.apply_transform_at_path(&mut result, &path, transform_type, ctx)?;

        Ok(result)
    }

    fn apply_transform_at_path(
        &mut self,
        current: &mut Value,
        path: &[PathSegment],
        transform: TransformType,
        ctx: &EvalContext,
    ) -> Result<(), EvalError> {
        if path.is_empty() {
            return Err(EvalError::TypeError(
                "Internal error: empty path in apply_transform_at_path".into(),
            ));
        }

        if path.len() == 1 {
            return self.apply_transform_to_parent(current, &path[0], transform, ctx);
        }

        let segment = &path[0];

        let rest = &path[1..];

        match (current, segment) {
            (Value::Object(map), PathSegment::Field(key)) => {
                let child = map
                    .get_mut(key)
                    .ok_or_else(|| EvalError::AccessError(format!("Field '{}' not found", key)))?;

                self.apply_transform_at_path(child, rest, transform, ctx)?;
                Ok(())
            }
            (Value::Array(arr), PathSegment::Index(idx)) => {
                let len = arr.len();
                let index = if *idx >= 0 {
                    *idx as usize
                } else if idx.unsigned_abs() < (len as u64) {
                    len - idx.unsigned_abs() as usize
                } else {
                    return Err(EvalError::AccessError(format!("Cannot access array element at {} for array with length {}", idx, len)))
                };
                let child = arr.get_mut(index).ok_or_else(|| {
                    EvalError::AccessError(format!(
                        "Array index {} out of bounds (length: {})",
                        idx, len,
                    ))
                })?;
                self.apply_transform_at_path(child, rest, transform, ctx)?;
                Ok(())
            }

            (Value::Array(_), PathSegment::Field(_)) => Err(EvalError::TypeError(
                "Cannot use field name on array".into(),
            )),

            (v, p) => Err(EvalError::TypeError(format!(
                "Cannot navigate through {} with path segment {:?}",
                type_name(v), p
            ))),
        }
    }

    fn apply_transform_to_parent(
        &mut self,
        parent: &mut Value,
        segment: &PathSegment,
        transform: TransformType,
        ctx: &EvalContext,
    ) -> Result<(), EvalError> {
        match (parent, segment) {
            (Value::Object(map), PathSegment::Field(key)) => match transform {
                TransformType::Replace(expr) => {
                    let new_value = self.eval_expr(&expr, ctx)?;
                    map.insert(key.clone(), new_value);
                    Ok(())
                }
                TransformType::FilterArray(cond) => {
                    let arr = map.get(key).ok_or_else(|| {
                        EvalError::AccessError(format!("Field '{}' not found", key))
                    })?;

                    match arr {
                        Value::Array(items) => {
                            let filtered = self.filter_array(items, &cond, ctx)?;
                            map.insert(key.clone(), Value::Array(filtered));
                            Ok(())
                        }
                        _ => Err(EvalError::TypeError(format!(
                            "Filter transform requires array, but '{}' is {}",
                            key, type_name(arr)
                        ))),
                    }
                }
                TransformType::MapArray(expr) => {
                    let arr = map.get(key).ok_or_else(|| {
                        EvalError::AccessError(format!("Field '{}' not found", key))
                    })?;

                    match arr {
                        Value::Array(items) => {
                            let mapped = self.map_array(items, &expr, ctx)?;
                            map.insert(key.clone(), Value::Array(mapped));
                            Ok(())
                        }
                        _ => Err(EvalError::TypeError(format!(
                            "Map transform requires array, but '{}' is {}",
                            key, type_name(arr)
                        ))),
                    }
                }
            },
            (Value::Array(arr), PathSegment::Index(idx)) => match transform {
                TransformType::Replace(expr) => {
                    let len = arr.len();
                    let index = if *idx >= 0 {
                        *idx as usize
                    } else {
                        len - idx.unsigned_abs() as usize
                    };

                    let new_val = self.eval_expr(&expr, ctx)?;
                    arr[index] = new_val;
                    Ok(())
                }
                TransformType::FilterArray(_) | TransformType::MapArray(_) => {
                    Err(EvalError::TypeError("Cannot filter/map on array index. Use a field instead (e.g., $[items] not $[items][0])".into()))
                }
            },

            (Value::Object(_), PathSegment::Index(_)) => {
                Err(EvalError::TypeError(
                        "Cannot use array index on object".into()
                ))
            }

            (Value::Array(_), PathSegment::Field(_)) => {
                Err(EvalError::TypeError(
                        "Cannot use array index on object".into()
                ))
            }

            _ => {
                Err(EvalError::TypeError(
                        "Invalid parent type for transform.".to_string()
                ))
            }
        }
    }

    fn filter_array(
        &self,
        items: &[Value],
        condition: &Expr,
        ctx: &EvalContext,
    ) -> Result<Vec<Value>, EvalError> {
        let mut result = Vec::new();

        for item in items {
            let lambda_ctx = ctx.with_lambda(item.clone());

            let keep = self.eval_expr(condition, &lambda_ctx)?;

            if keep.as_bool() {
                result.push(item.clone());
            }
        }

        Ok(result)
    }

    fn map_array(
        &self,
        items: &[Value],
        expr: &Expr,
        ctx: &EvalContext,
    ) -> Result<Vec<Value>, EvalError> {
        let mut result = Vec::new();

        for item in items {
            let lambda_ctx = ctx.with_lambda(item.clone());

            let new_value = self.eval_expr(expr, &lambda_ctx)?;

            result.push(new_value);
        }

        Ok(result)
    }

    /// Dispatch method calls to their implementations
    fn eval_method_call(
        &self,
        object: &Value,
        method: &str,
        args: &[Expr],
        ctx: &EvalContext,
    ) -> Result<Value, EvalError> {
        match method {
            // Array methods
            "any" => self.method_any(object, args, ctx),
            "all" => self.method_all(object, args, ctx),
            "filter" => self.method_filter(object, args, ctx),
            "map" => self.method_map(object, args, ctx),
            "count" => self.method_count(object),
            "length" => self.method_length(object),
            "sum" => self.method_sum(object, args, ctx),
            "min" => self.method_min(object),
            "max" => self.method_max(object),
            "avg" => self.method_avg(object),
            "first" => self.method_first(object),
            "last" => self.method_last(object),
            "exists" => self.method_exists(object),
            "unique" => self.method_unique(object),
            "sort" => self.method_sort(object, args, ctx),
            "sort_desc" => self.method_sort_desc(object),
            "reverse" => self.method_reverse(object),
            "flatten" => self.method_flatten(object),
            // String methods
            "upper" => self.method_upper(object),
            "lower" => self.method_lower(object),
            "trim" => self.method_trim(object),
            "split" => self.method_split(object, args, ctx),
            "contains" => self.method_contains(object, args, ctx),
            "startswith" => self.method_startswith(object, args, ctx),
            "endswith" => self.method_endswith(object, args, ctx),
            "matches" => self.method_matches(object, args, ctx),
            // Object methods
            "keys" => self.method_keys(object),
            "values" => self.method_values(object),
            // Type method (works on any value)
            "type" => self.method_type(object),
            _ => Err(EvalError::TypeError(format!(
                "Unknown method: {}",
                method
            ))),
        }
    }

    /// .any(lambda) - returns true if any element matches
    fn method_any(
        &self,
        object: &Value,
        args: &[Expr],
        ctx: &EvalContext,
    ) -> Result<Value, EvalError> {
        let arr = match object {
            Value::Array(arr) => arr,
            _ => {
                return Err(EvalError::TypeError(format!(
                    ".any() requires array, got {}",
                    type_name(object)
                )))
            }
        };

        if args.is_empty() {
            return Err(EvalError::TypeError(
                ".any() requires a predicate argument".to_string(),
            ));
        }

        let predicate = &args[0];

        for item in arr {
            let lambda_ctx = ctx.with_lambda(item.clone());
            let result = self.eval_expr(predicate, &lambda_ctx)?;
            if result.as_bool() {
                return Ok(Value::Boolean(true));
            }
        }

        Ok(Value::Boolean(false))
    }

    /// .all(lambda) - returns true if all elements match
    fn method_all(
        &self,
        object: &Value,
        args: &[Expr],
        ctx: &EvalContext,
    ) -> Result<Value, EvalError> {
        let arr = match object {
            Value::Array(arr) => arr,
            _ => {
                return Err(EvalError::TypeError(format!(
                    ".all() requires array, got {}",
                    type_name(object)
                )))
            }
        };

        if args.is_empty() {
            return Err(EvalError::TypeError(
                ".all() requires a predicate argument".to_string(),
            ));
        }

        let predicate = &args[0];

        for item in arr {
            let lambda_ctx = ctx.with_lambda(item.clone());
            let result = self.eval_expr(predicate, &lambda_ctx)?;
            if !result.as_bool() {
                return Ok(Value::Boolean(false));
            }
        }

        Ok(Value::Boolean(true))
    }

    /// .filter(lambda) - returns array of matching elements
    fn method_filter(
        &self,
        object: &Value,
        args: &[Expr],
        ctx: &EvalContext,
    ) -> Result<Value, EvalError> {
        let arr = match object {
            Value::Array(arr) => arr,
            _ => {
                return Err(EvalError::TypeError(format!(
                    ".filter() requires array, got {}",
                    type_name(object)
                )))
            }
        };

        if args.is_empty() {
            return Err(EvalError::TypeError(
                ".filter() requires a predicate argument".to_string(),
            ));
        }

        let predicate = &args[0];
        let filtered = self.filter_array(arr, predicate, ctx)?;

        Ok(Value::Array(filtered))
    }

    /// .map(lambda) - transforms each element
    fn method_map(
        &self,
        object: &Value,
        args: &[Expr],
        ctx: &EvalContext,
    ) -> Result<Value, EvalError> {
        let arr = match object {
            Value::Array(arr) => arr,
            _ => {
                return Err(EvalError::TypeError(format!(
                    ".map() requires array, got {}",
                    type_name(object)
                )))
            }
        };

        if args.is_empty() {
            return Err(EvalError::TypeError(
                ".map() requires a transform expression argument".to_string(),
            ));
        }

        let transform = &args[0];
        let mapped = self.map_array(arr, transform, ctx)?;

        Ok(Value::Array(mapped))
    }

    /// .count() - returns number of elements
    fn method_count(&self, object: &Value) -> Result<Value, EvalError> {
        match object {
            Value::Array(arr) => Ok(Value::Integer(arr.len() as i64)),
            _ => Err(EvalError::TypeError(format!(
                ".count() requires array, got {}",
                type_name(object)
            ))),
        }
    }

    /// .sum(lambda?) - sums numeric values, optionally extracting with lambda
    fn method_sum(
        &self,
        object: &Value,
        args: &[Expr],
        ctx: &EvalContext,
    ) -> Result<Value, EvalError> {
        let arr = match object {
            Value::Array(arr) => arr,
            _ => {
                return Err(EvalError::TypeError(format!(
                    ".sum() requires array, got {}",
                    type_name(object)
                )))
            }
        };

        let mut sum_int: i64 = 0;
        let mut sum_float: f64 = 0.0;
        let mut has_float = false;

        for item in arr {
            let value = if args.is_empty() {
                item.clone()
            } else {
                let lambda_ctx = ctx.with_lambda(item.clone());
                self.eval_expr(&args[0], &lambda_ctx)?
            };

            match value {
                Value::Integer(n) => {
                    if has_float {
                        sum_float += n as f64;
                    } else {
                        sum_int += n;
                    }
                }
                Value::Float(n) => {
                    if !has_float {
                        sum_float = sum_int as f64;
                        has_float = true;
                    }
                    sum_float += n;
                }
                _ => {
                    return Err(EvalError::TypeError(format!(
                        ".sum() requires numeric values, got {}",
                        type_name(&value)
                    )))
                }
            }
        }

        if has_float {
            Ok(Value::Float(sum_float))
        } else {
            Ok(Value::Integer(sum_int))
        }
    }

    /// .first() - returns first element or null
    fn method_first(&self, object: &Value) -> Result<Value, EvalError> {
        match object {
            Value::Array(arr) => Ok(arr.first().cloned().unwrap_or(Value::Null)),
            _ => Err(EvalError::TypeError(format!(
                ".first() requires array, got {}",
                type_name(object)
            ))),
        }
    }

    /// .last() - returns last element or null
    fn method_last(&self, object: &Value) -> Result<Value, EvalError> {
        match object {
            Value::Array(arr) => Ok(arr.last().cloned().unwrap_or(Value::Null)),
            _ => Err(EvalError::TypeError(format!(
                ".last() requires array, got {}",
                type_name(object)
            ))),
        }
    }

    /// .exists() - returns true if array exists and is non-empty
    fn method_exists(&self, object: &Value) -> Result<Value, EvalError> {
        match object {
            Value::Array(arr) => Ok(Value::Boolean(!arr.is_empty())),
            Value::Null => Ok(Value::Boolean(false)),
            _ => Err(EvalError::TypeError(format!(
                ".exists() requires array, got {}",
                type_name(object)
            ))),
        }
    }

    /// .unique() - returns array with duplicates removed
    fn method_unique(&self, object: &Value) -> Result<Value, EvalError> {
        let arr = match object {
            Value::Array(arr) => arr,
            _ => {
                return Err(EvalError::TypeError(format!(
                    ".unique() requires array, got {}",
                    type_name(object)
                )))
            }
        };

        let mut result = Vec::new();
        for item in arr {
            if !result.contains(item) {
                result.push(item.clone());
            }
        }

        Ok(Value::Array(result))
    }

    /// .sort(lambda?) - returns sorted array
    fn method_sort(
        &self,
        object: &Value,
        args: &[Expr],
        ctx: &EvalContext,
    ) -> Result<Value, EvalError> {
        let arr = match object {
            Value::Array(arr) => arr.clone(),
            _ => {
                return Err(EvalError::TypeError(format!(
                    ".sort() requires array, got {}",
                    type_name(object)
                )))
            }
        };

        if arr.is_empty() {
            return Ok(Value::Array(arr));
        }

        // Extract sort keys if lambda provided
        let mut items_with_keys: Vec<(Value, Value)> = if args.is_empty() {
            arr.iter().map(|v| (v.clone(), v.clone())).collect()
        } else {
            let mut result = Vec::new();
            for item in &arr {
                let lambda_ctx = ctx.with_lambda(item.clone());
                let key = self.eval_expr(&args[0], &lambda_ctx)?;
                result.push((item.clone(), key));
            }
            result
        };

        // Sort by keys
        items_with_keys.sort_by(|(_, a), (_, b)| self.compare_values(a, b));

        let sorted: Vec<Value> = items_with_keys.into_iter().map(|(v, _)| v).collect();
        Ok(Value::Array(sorted))
    }

    /// Compare two values for sorting
    fn compare_values(&self, a: &Value, b: &Value) -> std::cmp::Ordering {
        match (a, b) {
            (Value::Integer(a), Value::Integer(b)) => a.cmp(b),
            (Value::Float(a), Value::Float(b)) => a.partial_cmp(b).unwrap_or(std::cmp::Ordering::Equal),
            (Value::Integer(a), Value::Float(b)) => (*a as f64).partial_cmp(b).unwrap_or(std::cmp::Ordering::Equal),
            (Value::Float(a), Value::Integer(b)) => a.partial_cmp(&(*b as f64)).unwrap_or(std::cmp::Ordering::Equal),
            (Value::String(a), Value::String(b)) => a.cmp(b),
            (Value::Boolean(a), Value::Boolean(b)) => a.cmp(b),
            _ => std::cmp::Ordering::Equal,
        }
    }

    // ========================================
    // String Methods
    // ========================================

    /// .upper() - converts string to uppercase
    fn method_upper(&self, object: &Value) -> Result<Value, EvalError> {
        match object {
            Value::String(s) => Ok(Value::String(s.to_uppercase())),
            _ => Err(EvalError::TypeError(format!(
                ".upper() requires string, got {}",
                type_name(object)
            ))),
        }
    }

    /// .lower() - converts string to lowercase
    fn method_lower(&self, object: &Value) -> Result<Value, EvalError> {
        match object {
            Value::String(s) => Ok(Value::String(s.to_lowercase())),
            _ => Err(EvalError::TypeError(format!(
                ".lower() requires string, got {}",
                type_name(object)
            ))),
        }
    }

    /// .contains(substring) - returns true if string contains substring
    fn method_contains(
        &self,
        object: &Value,
        args: &[Expr],
        ctx: &EvalContext,
    ) -> Result<Value, EvalError> {
        let s = match object {
            Value::String(s) => s,
            _ => {
                return Err(EvalError::TypeError(format!(
                    ".contains() requires string, got {}",
                    type_name(object)
                )))
            }
        };

        if args.is_empty() {
            return Err(EvalError::TypeError(
                ".contains() requires a substring argument".to_string(),
            ));
        }

        let substr = self.eval_expr(&args[0], ctx)?;
        match substr {
            Value::String(sub) => Ok(Value::Boolean(s.contains(&sub))),
            _ => Err(EvalError::TypeError(format!(
                ".contains() argument must be string, got {}",
                type_name(&substr)
            ))),
        }
    }

    /// .startswith(prefix) - returns true if string starts with prefix
    fn method_startswith(
        &self,
        object: &Value,
        args: &[Expr],
        ctx: &EvalContext,
    ) -> Result<Value, EvalError> {
        let s = match object {
            Value::String(s) => s,
            _ => {
                return Err(EvalError::TypeError(format!(
                    ".startswith() requires string, got {}",
                    type_name(object)
                )))
            }
        };

        if args.is_empty() {
            return Err(EvalError::TypeError(
                ".startswith() requires a prefix argument".to_string(),
            ));
        }

        let prefix = self.eval_expr(&args[0], ctx)?;
        match prefix {
            Value::String(p) => Ok(Value::Boolean(s.starts_with(&p))),
            _ => Err(EvalError::TypeError(format!(
                ".startswith() argument must be string, got {}",
                type_name(&prefix)
            ))),
        }
    }

    /// .endswith(suffix) - returns true if string ends with suffix
    fn method_endswith(
        &self,
        object: &Value,
        args: &[Expr],
        ctx: &EvalContext,
    ) -> Result<Value, EvalError> {
        let s = match object {
            Value::String(s) => s,
            _ => {
                return Err(EvalError::TypeError(format!(
                    ".endswith() requires string, got {}",
                    type_name(object)
                )))
            }
        };

        if args.is_empty() {
            return Err(EvalError::TypeError(
                ".endswith() requires a suffix argument".to_string(),
            ));
        }

        let suffix = self.eval_expr(&args[0], ctx)?;
        match suffix {
            Value::String(suf) => Ok(Value::Boolean(s.ends_with(&suf))),
            _ => Err(EvalError::TypeError(format!(
                ".endswith() argument must be string, got {}",
                type_name(&suffix)
            ))),
        }
    }

    /// .matches(pattern) - returns true if string matches regex pattern
    fn method_matches(
        &self,
        object: &Value,
        args: &[Expr],
        ctx: &EvalContext,
    ) -> Result<Value, EvalError> {
        if args.len() != 1 {
            return Err(EvalError::TypeError(
                ".matches() requires exactly one argument".to_string(),
            ));
        }
        let pattern_val = self.eval_expr(&args[0], ctx)?;
        let pattern_str = match &pattern_val {
            Value::String(s) => s.as_str(),
            _ => {
                return Err(EvalError::TypeError(format!(
                    ".matches() argument must be string, got {}",
                    type_name(&pattern_val)
                )))
            }
        };
        let re = regex::Regex::new(pattern_str)
            .map_err(|e| EvalError::TypeError(format!("invalid regex: {e}")))?;
        match object {
            Value::String(s) => Ok(Value::Boolean(re.is_match(s))),
            _ => Ok(Value::Boolean(false)),
        }
    }

    // ========================================
    // Type Method
    // ========================================

    /// .type() - returns the type name as a string
    fn method_type(&self, object: &Value) -> Result<Value, EvalError> {
        let type_name = match object {
            Value::Null => "null",
            Value::Boolean(_) => "boolean",
            Value::Integer(_) => "number",
            Value::Float(_) => "number",
            Value::String(_) => "string",
            Value::Array(_) => "array",
            Value::Object(_) => "object",
        };
        Ok(Value::String(type_name.to_string()))
    }

    // ========================================
    // Additional Array Methods
    // ========================================

    /// .length() - returns length of array or string
    fn method_length(&self, object: &Value) -> Result<Value, EvalError> {
        match object {
            Value::Array(arr) => Ok(Value::Integer(arr.len() as i64)),
            Value::String(s) => Ok(Value::Integer(s.chars().count() as i64)),
            _ => Err(EvalError::TypeError(format!(
                ".length() requires array or string, got {}",
                type_name(object)
            ))),
        }
    }

    /// .min() - returns minimum value in array
    fn method_min(&self, object: &Value) -> Result<Value, EvalError> {
        let arr = match object {
            Value::Array(arr) => arr,
            _ => {
                return Err(EvalError::TypeError(format!(
                    ".min() requires array, got {}",
                    type_name(object)
                )))
            }
        };

        if arr.is_empty() {
            return Ok(Value::Null);
        }

        let mut min: Option<&Value> = None;
        for item in arr {
            match (min, item) {
                (None, v) => min = Some(v),
                (Some(Value::Integer(a)), Value::Integer(b)) if b < a => min = Some(item),
                (Some(Value::Float(a)), Value::Float(b)) if b < a => min = Some(item),
                (Some(Value::Integer(a)), Value::Float(b)) if *b < (*a as f64) => min = Some(item),
                (Some(Value::Float(a)), Value::Integer(b)) if (*b as f64) < *a => min = Some(item),
                (Some(Value::String(a)), Value::String(b)) if b < a => min = Some(item),
                _ => {}
            }
        }

        Ok(min.cloned().unwrap_or(Value::Null))
    }

    /// .max() - returns maximum value in array
    fn method_max(&self, object: &Value) -> Result<Value, EvalError> {
        let arr = match object {
            Value::Array(arr) => arr,
            _ => {
                return Err(EvalError::TypeError(format!(
                    ".max() requires array, got {}",
                    type_name(object)
                )))
            }
        };

        if arr.is_empty() {
            return Ok(Value::Null);
        }

        let mut max: Option<&Value> = None;
        for item in arr {
            match (max, item) {
                (None, v) => max = Some(v),
                (Some(Value::Integer(a)), Value::Integer(b)) if b > a => max = Some(item),
                (Some(Value::Float(a)), Value::Float(b)) if b > a => max = Some(item),
                (Some(Value::Integer(a)), Value::Float(b)) if *b > (*a as f64) => max = Some(item),
                (Some(Value::Float(a)), Value::Integer(b)) if (*b as f64) > *a => max = Some(item),
                (Some(Value::String(a)), Value::String(b)) if b > a => max = Some(item),
                _ => {}
            }
        }

        Ok(max.cloned().unwrap_or(Value::Null))
    }

    /// .avg() - returns average of numeric values in array
    fn method_avg(&self, object: &Value) -> Result<Value, EvalError> {
        let arr = match object {
            Value::Array(arr) => arr,
            _ => {
                return Err(EvalError::TypeError(format!(
                    ".avg() requires array, got {}",
                    type_name(object)
                )))
            }
        };

        if arr.is_empty() {
            return Ok(Value::Null);
        }

        let mut sum: f64 = 0.0;
        let mut count: usize = 0;

        for item in arr {
            match item {
                Value::Integer(n) => {
                    sum += *n as f64;
                    count += 1;
                }
                Value::Float(n) => {
                    sum += n;
                    count += 1;
                }
                _ => {}
            }
        }

        if count == 0 {
            return Ok(Value::Null);
        }

        Ok(Value::Float(sum / count as f64))
    }

    /// .sort_desc() - sorts array in descending order
    fn method_sort_desc(&self, object: &Value) -> Result<Value, EvalError> {
        let arr = match object {
            Value::Array(arr) => arr,
            _ => {
                return Err(EvalError::TypeError(format!(
                    ".sort_desc() requires array, got {}",
                    type_name(object)
                )))
            }
        };

        let mut sorted = arr.clone();
        sorted.sort_by(|a, b| {
            match (a, b) {
                (Value::Integer(x), Value::Integer(y)) => y.cmp(x),
                (Value::Float(x), Value::Float(y)) => y.partial_cmp(x).unwrap_or(std::cmp::Ordering::Equal),
                (Value::Integer(x), Value::Float(y)) => y.partial_cmp(&(*x as f64)).unwrap_or(std::cmp::Ordering::Equal),
                (Value::Float(x), Value::Integer(y)) => (*y as f64).partial_cmp(x).unwrap_or(std::cmp::Ordering::Equal),
                (Value::String(x), Value::String(y)) => y.cmp(x),
                _ => std::cmp::Ordering::Equal
            }
        });

        Ok(Value::Array(sorted))
    }

    /// .reverse() - reverses array order
    fn method_reverse(&self, object: &Value) -> Result<Value, EvalError> {
        let arr = match object {
            Value::Array(arr) => arr,
            _ => {
                return Err(EvalError::TypeError(format!(
                    ".reverse() requires array, got {}",
                    type_name(object)
                )))
            }
        };

        let mut reversed = arr.clone();
        reversed.reverse();
        Ok(Value::Array(reversed))
    }

    /// .flatten() - flattens nested arrays one level
    fn method_flatten(&self, object: &Value) -> Result<Value, EvalError> {
        let arr = match object {
            Value::Array(arr) => arr,
            _ => {
                return Err(EvalError::TypeError(format!(
                    ".flatten() requires array, got {}",
                    type_name(object)
                )))
            }
        };

        let mut result = Vec::new();
        for item in arr {
            match item {
                Value::Array(inner) => result.extend(inner.clone()),
                other => result.push(other.clone()),
            }
        }

        Ok(Value::Array(result))
    }

    // ========================================
    // Additional String Methods
    // ========================================

    /// .trim() - removes leading and trailing whitespace
    fn method_trim(&self, object: &Value) -> Result<Value, EvalError> {
        match object {
            Value::String(s) => Ok(Value::String(s.trim().to_string())),
            _ => Err(EvalError::TypeError(format!(
                ".trim() requires string, got {}",
                type_name(object)
            ))),
        }
    }

    /// .split(delimiter) - splits string into array
    fn method_split(
        &self,
        object: &Value,
        args: &[Expr],
        ctx: &EvalContext,
    ) -> Result<Value, EvalError> {
        let s = match object {
            Value::String(s) => s,
            _ => {
                return Err(EvalError::TypeError(format!(
                    ".split() requires string, got {}",
                    type_name(object)
                )))
            }
        };

        if args.is_empty() {
            return Err(EvalError::TypeError(
                ".split() requires a delimiter argument".to_string(),
            ));
        }

        let delim = self.eval_expr(&args[0], ctx)?;
        match delim {
            Value::String(d) => {
                let parts: Vec<Value> = if d.is_empty() {
                    s.chars().map(|c| Value::String(c.to_string())).collect()
                } else {
                    s.split(&d).map(|p| Value::String(p.to_string())).collect()
                };
                Ok(Value::Array(parts))
            }
            _ => Err(EvalError::TypeError(format!(
                ".split() delimiter must be string, got {}",
                type_name(&delim)
            ))),
        }
    }

    // ========================================
    // Object Methods
    // ========================================

    /// .keys() - returns array of object keys
    fn method_keys(&self, object: &Value) -> Result<Value, EvalError> {
        match object {
            Value::Object(obj) => {
                let keys: Vec<Value> = obj.keys().map(|k| Value::String(k.clone())).collect();
                Ok(Value::Array(keys))
            }
            _ => Err(EvalError::TypeError(format!(
                ".keys() requires object, got {}",
                type_name(object)
            ))),
        }
    }

    /// .values() - returns array of object values
    fn method_values(&self, object: &Value) -> Result<Value, EvalError> {
        match object {
            Value::Object(obj) => {
                let values: Vec<Value> = obj.values().cloned().collect();
                Ok(Value::Array(values))
            }
            _ => Err(EvalError::TypeError(format!(
                ".values() requires object, got {}",
                type_name(object)
            ))),
        }
    }
}
