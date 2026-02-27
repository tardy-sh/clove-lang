# JSON Query Language Specification

Version: 0.1.0  
Last Updated: 2025-01-04

## Table of Contents

1. [Overview](#overview)
2. [Basic Concepts](#basic-concepts)
3. [Syntax Reference](#syntax-reference)
4. [Operators](#operators)
5. [Built-in Functions](#built-in-functions)
6. [User-Defined Functions](#user-defined-functions)
7. [Examples](#examples)

---

## Overview

This query language provides a powerful, type-safe way to query and transform JSON documents. It emphasizes clarity through explicit operators and avoids ambiguous syntax.

### Design Principles

- **Explicit over implicit**: Operations are clearly marked with operators
- **No magic**: What you see is what happen
- **Scannable**: You can understand a query at a glance
- **Composable**: Build complex queries from simple parts

---

## Basic Concepts

### Pipeline Structure

Every query is a pipeline that starts with `$` (the root document) and chains operations with `|`:
```
$ | operation | operation | ... | !(output)
```

### The Three Core Operations

1. **Filter** `?()` - Keep or discard records based on conditions
2. **Transform** `~()` - Modify field values
3. **Output** `!()` - Specify what to return (optional, defaults to `$`)

### Context References

- `$` - Always refers to the root document
- `@name` - Scope reference (user-defined shorthand for a path)
- `@` - Current item in lambda/transform context
- `@N` - Argument N in a user-defined function

---

## Syntax Reference

### Root Access
```
$                    # The entire document
$[field]            # Access a field
$[field][nested]    # Nested access
$[array][0]         # Array index (integer keys on arrays)
$[array][?]         # Check if array exists and is non-empty
```

**Numeric Key Behavior:**

Numeric keys behave differently depending on the target type:

- **Integer keys** (`[0]`, `[42]`, `[-1]`):
  - On **arrays**: Array index access (supports negative indices)
  - On **objects**: Converted to string keys (`"0"`, `"42"`, `"-1"`)

- **Float keys** (`[1.5]`, `[3.14]`):
  - On **objects**: Converted to string keys (`"1.5"`, `"3.14"`)
  - On **arrays**: Type error (floats cannot index arrays)

Examples:
```
// Array indexing
$[items][0]         # First element
$[items][-1]        # Last element (negative index)

// Object access with numeric keys
$[metrics]["0"]     # Access field "0" (explicit string)
$[metrics][0]       # Access field "0" (integer → string)
$[config][1.5]      # Access field "1.5" (float → string)
```

### Scope References

Define shortcuts for frequently used paths:
```
@items := $[order][items]
| ?(@items.any(@[price] > 100))
```

Once defined, `@items` can be used anywhere in subsequent operations.

### Accessors

#### Bracket Notation
```
[fieldname]         # Access object key
[0]                 # Integer: array index OR object key "0" (context-dependent)
[1.5]               # Float: object key "1.5" (converted to string)
[?]                 # Existence check
[$[other_field]]    # Computed key (evaluation contexts only, not in transforms)
```

#### Dot Notation
```
.fieldname          # Same as [fieldname]
.method(args)       # Method call
```

### Existence Check

Two syntax forms are supported:
```
$[field][?]         # Bracket form
$[field]?           # Postfix form (equivalent)
```

Returns true if the value exists and is non-empty:
- For objects: true if exists and not null
- For arrays: true if exists and has length > 0
- For strings: true if exists and not empty

Can be used in method arguments:
```
$[items].filter(@[field]?)  # Keep items where field exists
```

### Negative Array Indices

Array access supports negative indices to access elements from the end:

```
$[items][-1]        # Last element
$[items][-2]        # Second-to-last element
```

Negative indices are calculated as: `array.length - abs(index)`  

### Environment Variables

Environment variables use bash-style syntax: `$VARNAME`
```
?($[price] > $PRICE_THRESHOLD)
~($[url] := $BASE_URL + "/api")
```

**Note**: `$VARNAME` (no brackets) is an environment variable, while `$[VARNAME]` accesses a field called "VARNAME" in the root document.

**Convention**: UPPERCASE is conventional for env vars but not enforced.

---

## Operators

### Filter Operator: `?()`

Filters records based on a condition. Records that don't match are excluded.
```
?($[status] == "active")
?($[price] > 100 and $[quantity] > 0)
```

### Transform Operator: `~()`

Modifies fields. Uses `:=` for assignment.
```
~($[price] := $[price] * 1.1)           # Increase price by 10%
~($[items] := ?(@[status] == "ok"))     # Filter items array
~($[categories] := @[category])          # Map to categories
```

**Transform semantics:**

- If RHS is `?()` → Filter the array
- If RHS uses `@` → Map over array elements
- Otherwise → Replace entire field value

**Transform target restrictions:**

- Must be a literal path (e.g., `$[items][0][price]`)
- Cannot use computed keys (e.g., `$[items][$[index]]` is invalid)
- Cannot use scope references as targets (e.g., `~(@items := ...)` is invalid)
- Integer keys create array index paths; float keys create object field paths

### Delete Operator: `-()`

Removes the specified field from the document. Silent no-op if the field does not exist. Can be chained.
```
$ | -($[password])                        # removes top-level "password" field
$ | -($[user][api_key])                   # removes nested "api_key" from "user" object
$ | -($[password]) | -($[secret])         # removes multiple fields
$ | -($[_internal]) | ~($[processed] := true)  # delete then transform
```

### Output Operator: `!()`

Specifies what to return. Optional; defaults to `!($)`.
```
!($)                                    # Return entire document
!($[items])                            # Return just items
!({"total": $[total], "count": $[count]})  # Return custom object
```

### Assignment Operator: `:=`

Used within transforms to assign values:
```
~($[field] := value)
```

### Pipe Operator: `|`

Chains operations in the pipeline:
```
$ | operation1 | operation2 | operation3
```

---

## Operators Reference

### Comparison Operators

| Operator | Meaning                  | Example              |
|----------|--------------------------|----------------------|
| `==`     | Equal                    | `$[age] == 25`      |
| `!=`     | Not equal                | `$[status] != "ok"` |
| `<`      | Less than                | `$[price] < 100`    |
| `>`      | Greater than             | `$[count] > 10`     |
| `<=`     | Less than or equal       | `$[age] <= 65`      |
| `>=`     | Greater than or equal    | `$[score] >= 90`    |

### Logical Operators

| Operator | Meaning     | Example                                  |
|----------|-------------|------------------------------------------|
| `and`    | Logical AND | `$[age] > 18 and $[verified] == true`   |
| `or`     | Logical OR  | `$[role] == "admin" or $[role] == "mod"`|
| `&&`     | Logical AND | `$[age] > 18 && $[verified] == true`    |
| `\|\|`   | Logical OR  | `$[role] == "admin" \|\| $[role] == "mod"`|

Both keyword (`and`/`or`) and symbol (`&&`/`||`) forms are supported.

### Null-Coalescing Operator

| Operator | Meaning          | Example                               |
|----------|------------------|---------------------------------------|
| `??`     | Null-coalescing  | `$[severity] ?? $[level] ?? "unknown"`|

Returns the left operand if it is non-null, otherwise evaluates and returns the right operand. Short-circuits: the right side is not evaluated if the left is non-null. Chains left-to-right: `a ?? b ?? c`.

```
$[severity] ?? $[level] ?? "unknown"          # first non-null value
($[bytes] ?? 0) / 1024                        # null-safe arithmetic
$[user][name] ?? $[user][login] ?? "anonymous" # nested field fallback
```

### Arithmetic Operators

| Operator | Meaning        | Example              |
|----------|----------------|----------------------|
| `+`      | Addition       | `$[price] + 10`     |
| `-`      | Subtraction    | `$[total] - $[tax]` |
| `*`      | Multiplication | `$[price] * 1.1`    |
| `/`      | Division       | `$[total] / 2`      |
| `%`      | Modulo         | `$[count] % 10`     |

**Arithmetic Type Behavior:**

The language uses high-precision decimal arithmetic and intelligently preserves integer types:

- Operations between integers return integers when the result is whole
- Operations involving floats return floats
- Mixed integer/float operations preserve integers when mathematically valid

Examples:
```
100 + 10        # → 110 (Integer)
100.0 + 10.0    # → 110.0 (Float)
100.0 + 10      # → 110 (Integer, result is whole)
100.5 + 10      # → 110.5 (Float, result has decimal)
100 / 10        # → 10 (Integer, exact division)
100 / 3         # → 33.333... (Float, inexact division)
```

### String Operators

| Operator | Meaning        | Example                     |
|----------|----------------|-----------------------------|
| `+`      | Concatenation  | `$[first] + " " + $[last]` |

---

## Built-in Functions

All functions are called as methods with dot notation:
```
$[array].method(args)
```

### Array Functions

#### `any(lambda)`

Returns true if any element matches the condition.
```
$[items].any(@[price] > 100)
$[tags].any(@ == "urgent")
```

#### `all(lambda)`

Returns true if all elements match the condition.
```
$[items].all(@[status] == "shipped")
$[scores].all(@ >= 60)
```

#### `filter(lambda)`

Returns a new array with only matching elements.
```
$[items].filter(@[category] == "electronics")
$[numbers].filter(@ > 0)
```

#### `map(lambda)`

Transforms each element and returns new array.
```
$[items].map(@[name])
$[prices].map(@ * 1.1)
```

#### `sum(lambda?)`

Sums numeric values. Optional lambda to extract values.
```
$[numbers].sum()
$[items].sum(@[price])
```

#### `count()`

Returns number of elements.
```
$[items].count()
```

#### `first()`

Returns first element or null if empty.
```
$[items].first()
```

#### `last()`

Returns last element or null if empty.
```
$[items].last()
```

#### `exists()`

Returns true if array exists and is non-empty.
```
$[items].exists()
```

#### `unique()`

Returns array with duplicate values removed.
```
$[tags].unique()
```

#### `sort()`

Returns sorted array (ascending).
```
$[numbers].sort()
$[items].sort(@[price])  # Sort by field
```

#### `sort_desc()`

Returns sorted array (descending).
```
$[numbers].sort_desc()
```

#### `length()`

Returns the number of elements in an array.
```
$[items].length()
```

#### `min()`

Returns the minimum numeric value in an array.
```
$[prices].min()
```

#### `max()`

Returns the maximum numeric value in an array.
```
$[prices].max()
```

#### `avg()`

Returns the average of numeric values in an array.
```
$[scores].avg()
```

#### `reverse()`

Returns array with elements in reverse order.
```
$[items].reverse()
```

#### `flatten()`

Flattens nested arrays one level deep.
```
$[nested].flatten()  # [[1,2],[3,4]] → [1,2,3,4]
```

### Object Functions

#### `keys()`

Returns array of object keys.
```
$[config].keys()
```

#### `values()`

Returns array of object values.
```
$[config].values()
```

### Type Functions

#### `type()`

Returns type as string: "object", "array", "string", "number", "boolean", "null".
```
$[field].type()
```

### String Functions

#### `upper()`

Converts string to uppercase.
```
$[name].upper()
```

#### `lower()`

Converts string to lowercase.
```
$[email].lower()
```

#### `contains(substring)`

Returns true if string contains substring.
```
$[text].contains("error")
```

#### `startswith(prefix)`

Returns true if string starts with prefix.
```
$[url].startswith("https://")
```

#### `endswith(suffix)`

Returns true if string ends with suffix.
```
$[filename].endswith(".json")
```

#### `trim()`

Removes leading and trailing whitespace.
```
$[input].trim()
```

#### `split(delimiter)`

Splits string into array by delimiter.
```
$[csv].split(",")  # "a,b,c" → ["a","b","c"]
```

#### `length()` (strings)

Returns the number of characters in a string.
```
$[name].length()
```

#### `matches(pattern)`

Returns true if the string matches the regex pattern (Rust regex syntax). Returns false for non-string receivers (not an error).
```
$[message].matches("Failed .* from \\d+\\.\\d+\\.\\d+\\.\\d+")
$[path].matches("^/api/v[12]/")
$[status].to_string().matches("^[45]")
```

---

## User-Defined Functions (UDFs)

Define reusable operations at the start of your query.

### Syntax
```
&function_name,arity := operation
```

- `&` prefix marks UDF definition
- `arity` is the number of arguments (0-9)
- Inside the function body, use `@1`, `@2`, etc. to reference arguments

### Examples

#### Define a Filter
```
&expensive,1 := ?(@1[price] > 100)
```

Use it:
```
?($[items].any(&expensive[@]))
```

#### Define a Transform
```
&discount,2 := ~(@1 := @1 * (1 - @2))
```

Use it:
```
~($[price] := &discount[$[price], 0.1])  # 10% discount
```

#### Define a Computed Value
```
&fullname,1 := @1[first] + " " + @1[last]
```

Use it:
```
~($[name] := &fullname[$[person]])
```

### UDF Configuration File

Store UDFs in `~/.query-lang-udfs.toml`:
```toml
[filters]
expensive = "?(@1[price] > 100)"
active = "?(@1[status] == 'active')"

[transforms]
discount = "~(@1 := @1 * (1 - @2))"
uppercase = "~(@1 := @1.upper())"

[computed]
fullname = "@1[first] + ' ' + @1[last]"
```

Load automatically when CLI starts.

---

## Examples

### Basic Queries

#### Simple Field Access
```
$ | !($[user][name])
```

#### Filter by Condition
```
$
| ?($[status] == "active")
| !($)
```

#### Transform Field
```
$
| ~($[price] := $[price] * 1.1)
| !($)
```

### Intermediate Queries

#### Filter Array Elements
```
$
| ~($[items] := ?(@[price] > 100))
| !($)
```

#### Use Scope Reference
```
$
| @user := $[user]
| ?(@user[verified] == true and @user[age] >= 18)
| !(@user)
```

#### Array Methods
```
$
| ?($[items].any(@[category] == "electronics"))
| ~($[total] := $[items].sum(@[price]))
| !($)
```

### Advanced Queries

#### Complex Nested Filtering
```
$
| @od := $[order_details]
| ?(@od[customer] == "ACME")
| ?(@od[date] > "2025-01-01")
| ~($[items] := ?(@[price] > 50 and @[status] == "available"))
| ?($[items].count() > 0)
| !($)
```

#### Multiple Transforms
```
$
| ~($[items] := ?(@[category] == "electronics"))
| ~($[item_names] := $[items][@[name]])
| ~($[total] := $[items].sum(@[price]))
| ~($[average] := $[total] / $[items].count())
| !({"names": $[item_names], "total": $[total], "avg": $[average]})
```

#### With UDFs
```
&expensive,1 := ?(@1[price] > 100)
&active,1 := ?(@1[status] == "active")
&discount,2 := ~(@1 := @1 * (1 - @2))

$
| @items := $[items]
| ?(@items.any(&expensive[@]))
| ~($[items] := ?(&active[@]))
| ~($[items][@[price]] := &discount[@[price], 0.15])
| !($)
```

### Real-World Example

Given this JSON:
```json
{
  "orders": [
    {
      "id": 1,
      "customer": "ACME Corp",
      "date": "2025-01-15",
      "items": [
        {"name": "Widget", "price": 50, "category": "hardware"},
        {"name": "Gadget", "price": 150, "category": "electronics"}
      ]
    },
    {
      "id": 2,
      "customer": "TechStart",
      "date": "2025-02-01",
      "items": [
        {"name": "Doohickey", "price": 25, "category": "accessories"}
      ]
    }
  ]
}
```

Query: "Get all orders from 2025 with expensive electronics, apply 10% discount"
```
&expensive,1 := ?(@1[price] > 100)
&is_electronics,1 := ?(@1[category] == "electronics")

$[orders][*]
| ?(@[date] >= "2025-01-01")
| ?(@[items].any(&expensive[@] and &is_electronics[@]))
| ~(@[items] := ?(&is_electronics[@]))
| ~(@[items][@[price]] := @[price] * 0.9)
```

---

## Operator Precedence

From highest to lowest:

1. Accessors: `[]`, `.`
2. Multiplicative: `*`, `/`, `%`
3. Additive: `+`, `-`
4. Comparison: `==`, `!=`, `<`, `>`, `<=`, `>=`
5. Logical AND: `and`
6. Logical OR: `or`
7. Null-coalescing: `??`

Use parentheses `()` to override precedence.

---

## Error Handling

### Common Errors

- **Undefined scope reference**: Using `@name` before defining it
- **Invalid UDF arity**: Calling `&func[a, b]` when `&func,1` expects 1 arg
- **Type mismatch**: Comparing incompatible types
- **Null access**: Accessing fields on null values
- **Missing output**: Some contexts require explicit `!()`

### Best Practices

1. Always define scope references before use
2. Check array existence with `[?]` before filtering
3. Use explicit parentheses in complex conditions
4. Define UDFs at the top of your query
5. Use meaningful scope reference names

---

## Language Characteristics

### Type System

- **Dynamic**: Types determined at runtime
- **JSON-compatible**: Supports all JSON types
- **Null-safe**: Operations on null return null (no errors)

### Evaluation Model

- **Eager**: All operations execute immediately
- **Left-to-right**: Pipeline executes in order
- **Short-circuit**: `and`/`or` stop early when result is determined

### Immutability

- Original document is never modified
- Each operation produces new values
- Scope references are read-only aliases

---

## Reserved Keywords

Cannot be used as identifiers:

- `and`
- `or`
- `true`
- `false`
- `null`

## Reserved Operators

- `$` - Root document
- `@` - Context/lambda/argument reference
- `&` - UDF prefix
- `?` - Filter/existence check
- `~` - Transform
- `!` - Output
- `|` - Pipeline
- `:=` - Assignment

---

## Grammar Notation

The formal grammar uses EBNF notation:

- `=` defines a rule
- `,` sequence
- `|` alternation (or)
- `[ ]` optional
- `{ }` repetition (zero or more)
- `( )` grouping
- `" "` terminal (literal text)
- `? ?` special sequence (description)

---

## Version History

### 0.1.0 (2025-01-04)
- Initial specification
- Core operators: `?`, `~`, `!`
- Scope references with `@name`
- User-defined functions with `&`
- Built-in array and string methods
