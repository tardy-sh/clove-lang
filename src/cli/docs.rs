//! Documentation content for clove CLI

use super::CliError;

/// Available documentation categories
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DocCategory {
    Syntax,
    Operators,
    ArrayMethods,
    StringMethods,
    ObjectMethods,
    Scopes,
    Types,
    Queries,
}

impl DocCategory {
    /// Parse category name from string
    pub fn from_str(s: &str) -> Option<Self> {
        match s.to_lowercase().replace('-', "_").as_str() {
            "syntax" => Some(Self::Syntax),
            "operators" | "ops" => Some(Self::Operators),
            "array_methods" | "array" | "arrays" => Some(Self::ArrayMethods),
            "string_methods" | "string" | "strings" => Some(Self::StringMethods),
            "object_methods" | "object" | "objects" => Some(Self::ObjectMethods),
            "scopes" | "scope" => Some(Self::Scopes),
            "types" | "type" => Some(Self::Types),
            "queries" | "query" | "pipes" => Some(Self::Queries),
            _ => None,
        }
    }
}

/// Get the docs overview (category listing)
pub fn get_docs_overview() -> &'static str {
    r#"CLOVE DOCUMENTATION

Clove is a JSON query language for filtering, transforming, and validating JSON
documents. Queries start with $ (the root document) and chain accessors, methods,
and operators to extract or transform data.

DOCUMENTATION CATEGORIES

  syntax            Root access, field access, array indexing, and basic notation
  operators         Comparison, logical, arithmetic, and existence operators
  array-methods     Filter, map, aggregate, and transform methods for arrays
  string-methods    Text manipulation and inspection methods for strings
  object-methods    Methods for working with object keys and values
  scopes            Reference scopes: $ (root), @ (current), and environment vars
  types             Type system, type checking, and coercion rules
  queries           Pipe syntax for document-level filter and transform operations

QUICK REFERENCE

  $                 Root document
  $[field]          Field access
  $[arr][0]         Array index
  @                 Current element (in filter/map)
  .method()         Method call
  | ?()  | ~()      Query operators

Run 'clove doc <category>' for detailed documentation.
Run 'clove onboard' for an interactive tutorial.
"#
}

/// Get documentation for a specific category
pub fn get_doc_category(name: &str) -> Result<&'static str, CliError> {
    match DocCategory::from_str(name) {
        Some(DocCategory::Syntax) => Ok(SYNTAX_DOC),
        Some(DocCategory::Operators) => Ok(OPERATORS_DOC),
        Some(DocCategory::ArrayMethods) => Ok(ARRAY_METHODS_DOC),
        Some(DocCategory::StringMethods) => Ok(STRING_METHODS_DOC),
        Some(DocCategory::ObjectMethods) => Ok(OBJECT_METHODS_DOC),
        Some(DocCategory::Scopes) => Ok(SCOPES_DOC),
        Some(DocCategory::Types) => Ok(TYPES_DOC),
        Some(DocCategory::Queries) => Ok(QUERIES_DOC),
        None => Err(CliError::UnknownCategory(name.to_string())),
    }
}

const SYNTAX_DOC: &str = r#"SYNTAX - Basic Access Notation

ROOT ACCESS
  $
    The entire JSON document. All queries start from root.

    Example:
      Input:  {"name": "Alice"}
      Query:  $
      Output: {"name": "Alice"}

FIELD ACCESS
  $[field]
    Access an object field by name. Field names are unquoted identifiers.

    Example:
      Input:  {"user": {"name": "Alice", "age": 30}}
      Query:  $[user][name]
      Output: "Alice"

    Constraints:
      - Field must exist, otherwise returns null
      - Field names are case-sensitive
      - Use identifier syntax (letters, digits, underscores)

ARRAY INDEX ACCESS
  $[array][index]
    Access array elements by zero-based index.

    Example:
      Input:  {"items": ["a", "b", "c"]}
      Query:  $[items][1]
      Output: "b"

    Negative indices (from end):
      $[items][-1]    Last element
      $[items][-2]    Second to last

    Constraints:
      - Index out of bounds returns null
      - Index must be an integer literal

CHAINED ACCESS
  $[field][nested][0][deep]
    Chain multiple accessors to traverse nested structures.

    Example:
      Input:  {"users": [{"profile": {"city": "NYC"}}]}
      Query:  $[users][0][profile][city]
      Output: "NYC"

EXISTENCE CHECK
  $[field]?
    Returns true if the field exists, false otherwise.

    Example:
      Input:  {"name": "Alice"}
      Query:  $[name]?
      Output: true

      Query:  $[email]?
      Output: false

    Constraints:
      - Only checks existence, not truthiness
      - null field that exists returns true
"#;

const OPERATORS_DOC: &str = r#"OPERATORS - Comparison, Logical, and Arithmetic

COMPARISON OPERATORS
  ==    Equal (strict type matching)
  !=    Not equal
  <     Less than
  >     Greater than
  <=    Less than or equal
  >=    Greater than or equal

  Examples:
    $[age] >= 18
    $[status] == "active"
    $[count] != 0

  Constraints:
    - Comparing different types returns false (except == null)
    - Strings compare lexicographically
    - Arrays and objects are compared by value

LOGICAL OPERATORS
  &&    Logical AND (short-circuit)
  ||    Logical OR (short-circuit)
  !     Logical NOT (prefix)

  Examples:
    $[age] >= 18 && $[verified]
    $[role] == "admin" || $[role] == "mod"
    !$[deleted]

  Constraints:
    - Operands are coerced to boolean
    - null, false, 0, "", [], {} are falsy
    - Everything else is truthy

ARITHMETIC OPERATORS
  +     Addition / String concatenation
  -     Subtraction
  *     Multiplication
  /     Division
  %     Modulo (remainder)

  Examples:
    $[price] * $[quantity]
    $[total] / $[count]
    $[value] % 2 == 0

  Type behavior:
    Integer + Integer = Integer
    Integer + Float   = Float
    Float + Float     = Float
    String + String   = String (concatenation)

  Division special case:
    100 / 4  => 25      (exact: returns integer)
    100 / 3  => 33.333  (inexact: returns float)

  Constraints:
    - Division by zero raises an error
    - Modulo by zero raises an error
    - Cannot mix strings with numbers in arithmetic

OPERATOR PRECEDENCE (highest to lowest)
  1. !           Unary NOT
  2. * / %       Multiplicative
  3. + -         Additive
  4. < > <= >=   Relational
  5. == !=       Equality
  6. &&          Logical AND
  7. ||          Logical OR

  Use parentheses to override: ($[a] || $[b]) && $[c]
"#;

const ARRAY_METHODS_DOC: &str = r#"ARRAY-METHODS - Filter, Transform, and Aggregate

ELEMENT ACCESS
  .first()
    Returns the first element, or null if empty.
    Example: $[items].first()

  .last()
    Returns the last element, or null if empty.
    Example: $[items].last()

  .length()
    Returns the number of elements.
    Example: $[items].length()  =>  3

SEARCHING
  .contains(value)
    Returns true if the array contains the value.
    Example: $[tags].contains("urgent")  =>  true/false

    Constraints:
      - Uses strict equality for comparison
      - Works with any value type

FILTERING
  .filter(condition)
    Returns elements where condition is true.
    Use @ to reference the current element.

    Examples:
      $[numbers].filter(@ > 5)
      $[users].filter(@[age] >= 18)
      $[items].filter(@[active] && @[price] < 100)

    Constraints:
      - Returns empty array if no matches
      - Condition must evaluate to boolean

TRANSFORMATION
  .map(expression)
    Transforms each element using the expression.
    Use @ for current element.

    Examples:
      $[numbers].map(@ * 2)
      $[users].map(@[name])
      $[items].map({"id": @[id], "label": @[title]})

    Constraints:
      - Returns array of same length
      - Can return any type per element

AGGREGATION
  .sum()
    Sum of all numeric elements.
    Example: $[prices].sum()  =>  150

  .min()
    Minimum value (works with numbers and strings).
    Example: $[scores].min()  =>  42

  .max()
    Maximum value.
    Example: $[scores].max()  =>  100

  .avg()
    Average of numeric elements (always returns float).
    Example: $[scores].avg()  =>  75.5

    Constraints:
      - Empty array returns null for min/max, 0 for sum, null for avg
      - Non-numeric elements are skipped in sum/avg

ORDERING
  .sort()
    Sort ascending.
    Example: $[numbers].sort()  =>  [1, 2, 3]

  .sort_desc()
    Sort descending.
    Example: $[numbers].sort_desc()  =>  [3, 2, 1]

  .reverse()
    Reverse element order.
    Example: $[items].reverse()

    Constraints:
      - Sorts by natural ordering (numbers, then strings)
      - Mixed types: numbers < strings

SET OPERATIONS
  .unique()
    Remove duplicate values (preserves first occurrence).
    Example: [1, 2, 1, 3].unique()  =>  [1, 2, 3]

  .flatten()
    Flatten nested arrays one level deep.
    Example: [[1, 2], [3, 4]].flatten()  =>  [1, 2, 3, 4]

    Constraints:
      - Non-array elements are kept as-is
      - Only flattens one level
"#;

const STRING_METHODS_DOC: &str = r#"STRING-METHODS - Text Manipulation and Inspection

LENGTH
  .length()
    Returns the number of characters.
    Example: "hello".length()  =>  5

    Constraints:
      - Counts Unicode scalar values
      - Empty string returns 0

SEARCHING
  .contains(substring)
    Returns true if string contains the substring.
    Example: $[name].contains("Alice")  =>  true/false

    Constraints:
      - Case-sensitive
      - Empty substring always returns true

  .startswith(prefix)
    Returns true if string starts with prefix.
    Example: $[path].startswith("/api/")  =>  true/false

  .endswith(suffix)
    Returns true if string ends with suffix.
    Example: $[file].endswith(".json")  =>  true/false

    Constraints:
      - Case-sensitive
      - Empty prefix/suffix always returns true

CASE CONVERSION
  .upper()
    Convert to uppercase.
    Example: "hello".upper()  =>  "HELLO"

  .lower()
    Convert to lowercase.
    Example: "HELLO".lower()  =>  "hello"

    Constraints:
      - Uses Unicode case mapping
      - Non-alphabetic characters unchanged

WHITESPACE
  .trim()
    Remove leading and trailing whitespace.
    Example: "  hello  ".trim()  =>  "hello"

    Constraints:
      - Removes spaces, tabs, newlines
      - Interior whitespace preserved

SPLITTING
  .split(delimiter)
    Split string into array by delimiter.
    Example: "a,b,c".split(",")  =>  ["a", "b", "c"]

    Constraints:
      - Empty string returns [""]
      - Delimiter not found returns [original]
      - Empty delimiter splits into characters

CONCATENATION
  Use the + operator to concatenate strings.
    Example: $[first] + " " + $[last]  =>  "John Doe"

    Constraints:
      - Both operands must be strings
      - Use .to_string() to convert other types (not yet implemented)
"#;

const OBJECT_METHODS_DOC: &str = r#"OBJECT-METHODS - Working with Keys and Values

KEYS
  .keys()
    Returns an array of the object's keys.

    Example:
      Input:  {"name": "Alice", "age": 30}
      Query:  $.keys()
      Output: ["name", "age"]

    Constraints:
      - Order is not guaranteed (depends on implementation)
      - Returns empty array for empty object
      - Only works on objects, not arrays

VALUES
  .values()
    Returns an array of the object's values.

    Example:
      Input:  {"name": "Alice", "age": 30}
      Query:  $.values()
      Output: ["Alice", 30]

    Constraints:
      - Order matches .keys() order
      - Returns empty array for empty object
      - Only works on objects, not arrays

TYPE CHECK
  .type()
    Returns the type name as a string.

    Example:
      {}.type()        =>  "object"
      [].type()          =>  "array"
      "text".type()      =>  "string"

    See 'clove doc types' for all type names.

COMMON PATTERNS

  Check if object has a key:
    $[field]?

  Get all keys matching a pattern:
    $.keys().filter(@.startswith("user_"))

  Transform object to array of entries:
    $.keys().map({"key": @, "value": $[@]})

  Count keys:
    $.keys().length()
"#;

const SCOPES_DOC: &str = r#"SCOPES - Reference Contexts

$ (ROOT SCOPE)
  References the original input document.
  All queries start from root.

  Example:
    Input: {"config": {"debug": true}, "items": [1, 2, 3]}
    Query: $[config][debug]
    Output: true

  Use inside methods to reference root from nested context:
    $[items].filter(@ > $[config][threshold])

@ (CURRENT SCOPE)
  References the current element in filter/map operations.
  Only valid inside .filter() and .map() method arguments.

  Examples:
    $[numbers].filter(@ > 10)
    $[users].map(@[name])
    $[items].filter(@[price] < 100 && @[stock] > 0)

  Nested access:
    @[field]              Field of current element
    @[nested][path]       Nested access on current

  Combining with root:
    $[products].filter(@[category] == $[default_category])

    This compares each product's category against the root
    document's default_category field.

ENVIRONMENT VARIABLES
  $VARIABLE_NAME
    Access shell environment variables.

  Example:
    $[env] == $NODE_ENV
    $[config][api_key] == $API_KEY

  Constraints:
    - Variable name must be uppercase by convention
    - Undefined variables return null
    - Values are always strings

SCOPE RESOLUTION ORDER
  1. @ - Current element (only in filter/map)
  2. $ - Root document
  3. $VAR - Environment variable

  Explicit scoping avoids ambiguity:
    $[items].filter(@[x] > $[x])
    Here @[x] is item's x, $[x] is root's x.
"#;

const TYPES_DOC: &str = r#"TYPES - Type System and Coercion

PRIMITIVE TYPES

  null
    Absence of value.
    Literal: null
    .type() returns: "null"

  boolean
    True or false.
    Literals: true, false
    .type() returns: "boolean"

  integer
    Whole numbers (64-bit signed).
    Examples: 0, 42, -17, 9007199254740991
    .type() returns: "integer"

  float
    Floating-point numbers (64-bit IEEE 754).
    Examples: 3.14, -0.5, 1.0, 1e10
    .type() returns: "float"

  string
    UTF-8 text in double quotes.
    Examples: "hello", "world", ""
    .type() returns: "string"

COMPOSITE TYPES

  array
    Ordered collection of values.
    Examples: [], [1, 2, 3], ["a", null, true]
    .type() returns: "array"

  object
    Unordered key-value mapping.
    Examples: {}, {"name": "Alice", "age": 30}
    .type() returns: "object"

TYPE COERCION

  Arithmetic:
    integer + integer  =>  integer
    integer + float    =>  float
    float + float      =>  float

  Division special case:
    10 / 2   =>  5       (exact result: integer)
    10 / 3   =>  3.333   (inexact result: float)

  String concatenation:
    "a" + "b"  =>  "ab"
    Mixing string + number raises error

  Boolean coercion (for logical operators):
    Falsy: null, false, 0, 0.0, "", [], {}
    Truthy: everything else

  Comparison:
    Different types are never equal (except null == null)
    Comparing incompatible types returns false, not error

TYPE CHECKING

  .type()
    Returns type as lowercase string.

    null.type()           =>  "null"
    true.type()           =>  "boolean"
    42.type()             =>  "integer"
    3.14.type()           =>  "float"
    "hello".type()        =>  "string"
    [1, 2].type()         =>  "array"
    {"a": 1}.type()       =>  "object"

  Type-based filtering:
    $[items].filter(@.type() == "string")
"#;

const QUERIES_DOC: &str = r#"QUERIES - Pipe Syntax for Document Operations

Query operations use pipe syntax to perform document-level filtering
and transformation. Unlike methods (which operate on values), query
operators transform or validate the entire document flow.

FILTER OPERATOR
  $ | ?(<condition>)

  Passes the document through if condition is true, returns null if false.
  Used for validation and conditional processing.

  Examples:
    $ | ?($[status] == "active")
    $ | ?($[price] > 0 && $[quantity] > 0)
    $ | ?($[email]?)

  Behavior:
    - Condition true: returns entire document unchanged
    - Condition false: returns null

  Use case: Validate documents before processing
    $ | ?($[version] >= 2) | ~($[data] := @[value])

TRANSFORM OPERATOR
  $ | ~(<path> := <expression>)

  Modifies a field in the document.

  Examples:
    $ | ~($[price] := $[price] * 1.1)
    $ | ~($[updated] := "2024-01-01")
    $ | ~($[total] := $[price] * $[quantity])

  Array field transforms:
    Filter array: $ | ~($[items] := ?(@[active]))
    Map array:    $ | ~($[names] := @[name])

  Constraints:
    - Path must be a valid field reference
    - Creates field if it doesn't exist
    - Expression can reference $, @, or literals

OUTPUT OPERATORS
  $ | !json
    Format output as compact JSON.

  $ | !json_pretty
    Format output as indented JSON.

  Note: The CLI always outputs JSON, so these are mainly useful
  for explicit formatting control in query chains.

CHAINING OPERATIONS
  Operations can be chained left-to-right:

    $ | ?($[active]) | ~($[price] := $[price] * 0.9) | !json_pretty

  This:
    1. Filters to active documents only
    2. Applies 10% discount to price
    3. Outputs as pretty JSON

QUERY VS METHOD

  Methods operate on values:
    $[items].filter(@[active])     Filter the items array

  Query operators operate on documents:
    $ | ?($[items].length() > 0)   Filter entire document

  Combining both:
    $ | ?($[items].length() > 0) | ~($[items] := ?(@[active]))
"#;
