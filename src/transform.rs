use crate::{ast::Expr, evaluator::EvalError};

/// A segment in a navigable path used for transformations.
///
/// Paths are extracted from access expressions and used to locate
/// and modify values in the JSON document during transform operations.
#[derive(Debug, Clone, PartialEq)]
pub enum PathSegment {
    /// Object field access by name
    ///
    /// # Examples
    /// - `$[name]` → `Field("name")`
    /// - `$[user][email]` → `[Field("user"), Field("email")]`
    /// - `$[config][1.5]` → `Field("1.5")` (float converted to string)
    Field(String),

    /// Array element access by index
    ///
    /// # Examples
    /// - `$[items][0]` → `Index(0)`
    /// - `$[items][-1]` → `Index(-1)` (negative indices supported)
    ///
    /// # Note
    /// Only integer literals create Index segments. Float literals always
    /// create Field segments (converted to strings).
    Index(i64),
}

/// A sequence of path segments representing a navigation path through a JSON document.
///
/// Used by the transform system to locate and modify nested values.
///
/// # Examples
///
/// For the access expression `$[user][profile][name]`, the path would be:
/// - `PathSegment::Field("user")`
/// - `PathSegment::Field("profile")`
/// - `PathSegment::Field("name")`
///
/// For the access expression `$[items][0][price]`, the path would be:
/// - `PathSegment::Field("items")`
/// - `PathSegment::Index(0)`
/// - `PathSegment::Field("price")`
pub type Path = Vec<PathSegment>;

/// Extract a navigable path from an access expression
///
/// # Examples
/// ```
/// // $[field] → [Field("field")]
/// // $[items][0] → [Field("items"), Index(0)]
/// // $[user][profile][name] → [Field("user"), Field("profile"), Field("name")]
/// ```
pub fn extract_path(expr: &Expr) -> Result<Path, EvalError> {
    let mut segments = Vec::new();
    extract_path_recursive(expr, &mut segments)?;
    Ok(segments)
}

fn extract_path_recursive(expr: &Expr, segments: &mut Path) -> Result<(), EvalError> {
    match expr {
        Expr::Root => {
            // Root ($) is the starting point, adds no segment
            Ok(())
        }

        Expr::Access { object, key } => {
            // First, extract path from the object (left to right traversal)
            extract_path_recursive(object, segments)?;

            // Then add this key as a segment
            match key.as_ref() {
                Expr::Key(name) => {
                    segments.push(PathSegment::Field(name.clone()));
                    Ok(())
                }

                Expr::Float(n) => {


                    segments.push(PathSegment::Field(n.to_string()));
                    Ok(())
                }

                Expr::Integer(n) => {


                    segments.push(PathSegment::Index(*n));
                    Ok(())
                }

                Expr::String(s) => {
                    // String literal used as field name (quoted key)
                    segments.push(PathSegment::Field(s.clone()));
                    Ok(())
                }

                // Any other expression in key position is invalid for transforms
                _ => Err(EvalError::TypeError(
                    "Transform target cannot contain computed keys. Use literal field names or indices only.".to_string(),
                )),
            }
        }

        Expr::ScopeRef(name) => {
            // Scope references evaluate to values, not paths
            // We can't transform through them
            Err(EvalError::TypeError(format!(
                "Cannot use scope reference @{} as transform target. Use the original path instead (e.g., $[items] not @items)",
                name
            )))
        }

        // Any other expression type is invalid as a transform target
        _ => Err(EvalError::TypeError(
            "Invalid transform target. Target must be an access path like $[field] or $[items][0][name]".to_string(),
        )),
    }
}

/// Type of transform operation to perform on the target field.
///
/// The transform type is automatically detected based on the value expression.
#[derive(Debug, Clone, PartialEq)]
pub enum TransformType {
    /// Simple value replacement
    ///
    /// Replaces the entire field value with the result of evaluating the expression.
    ///
    /// # Examples
    /// ```text
    /// ~($[price] := 100)              // Replace with literal
    /// ~($[total] := $[price] * 1.1)   // Replace with computed value
    /// ~($[count] := $[a] + $[b])      // Replace with sum of other fields
    /// ```
    Replace(Expr),

    /// Filter array elements
    ///
    /// Filters the array at the target path, keeping only elements that match the condition.
    /// Uses explicit filter syntax `?()`.
    ///
    /// # Examples
    /// ```text
    /// ~($[items] := ?(@[price] > 100))        // Keep expensive items
    /// ~($[users] := ?(@[active] == true))     // Keep active users
    /// ~($[tags] := ?(@ != "deprecated"))      // Remove deprecated tags
    /// ```
    FilterArray(Expr),

    /// Map over array elements
    ///
    /// Applies a transformation to each element in the array, using `@` to reference
    /// the current element.
    ///
    /// # Examples
    /// ```text
    /// ~($[prices] := @[price])              // Extract price field from each item
    /// ~($[prices] := @[price] * 1.1)        // Increase all prices by 10%
    /// ~($[names] := @[first] + " " + @[last]) // Combine first and last names
    /// ```
    MapArray(Expr),
}

/// Determine what type of transform to perform based on the value expression
///
/// # Transform Type Detection Rules
///
/// 1. **FilterArray**: Value is wrapped in `Expr::Filter`
///    - Example: `?(@[price] > 100)`
///
/// 2. **MapArray**: Value uses lambda parameter `@`
///    - Example: `@[price]`, `@[price] * 1.1`, `@[name] + " " + @[surname]`
///
/// 3. **Replace**: Everything else (simple values or expressions not using @)
///    - Example: `100`, `$[other_field]`, `$[a] + $[b]`
pub fn determine_transform_type(value_expr: &Expr) -> TransformType {
    match value_expr {
        // Explicit filter syntax
        Expr::Filter(condition) => TransformType::FilterArray(condition.as_ref().clone()),

        // Check if expression uses lambda parameter (@)
        expr if uses_lambda_param(expr) => TransformType::MapArray(expr.clone()),

        // Default: simple replacement
        expr => TransformType::Replace(expr.clone()),
    }
}

/// Check if an expression uses the lambda parameter (@)
///
/// Recursively walks the expression tree looking for `Expr::LambdaParam`
pub fn uses_lambda_param(expr: &Expr) -> bool {
    match expr {
        // Direct lambda parameter
        Expr::LambdaParam => true,

        // Access might contain @ in object or key
        Expr::Access { object, key } => uses_lambda_param(object) || uses_lambda_param(key),

        // Binary operations check both sides
        Expr::BinaryOp { left, right, .. } => uses_lambda_param(left) || uses_lambda_param(right),

        // Object literals check all values
        Expr::Object(pairs) => pairs
            .iter()
            .any(|(_, value_expr)| uses_lambda_param(value_expr)),

        // Array literals check all elements
        Expr::Array(elements) => elements.iter().any(|elem| uses_lambda_param(elem)),

        // Filter expression - check the condition
        Expr::Filter(condition) => uses_lambda_param(condition),

        // Existence check - check inner expression
        Expr::ExistenceCheck(inner) => uses_lambda_param(inner),

        // Method calls check object and all arguments
        Expr::MethodCall { object, args, .. } => {
            uses_lambda_param(object) || args.iter().any(|arg| uses_lambda_param(arg))
        }

        // UDF calls check all arguments
        Expr::UDFCall { args, .. } => args.iter().any(|arg| uses_lambda_param(arg)),

        // These never contain lambda params
        Expr::Null
        | Expr::Boolean(_)
        | Expr::Float(_)
        | Expr::String(_)
        | Expr::Root
        | Expr::EnvVar(_)
        | Expr::ScopeRef(_)
        | Expr::ArgRef(_)
        | Expr::Integer(_)
        | Expr::Key(_) => false,
    }
}
