//! Onboarding tutorial content for clove CLI

/// Get the interactive onboarding tutorial content
pub fn get_onboarding_content() -> &'static str {
    r#"WELCOME TO CLOVE

Clove is a query language for working with JSON data.

STEP 1: ROOT ACCESS
-------------------
All queries start with $ (the root document).

  echo '{"name": "Alice"}' | clove check '$'
  => {"name": "Alice"}

STEP 2: FIELD ACCESS
--------------------
Use $[field] to access object properties.

  echo '{"user": {"name": "Alice"}}' | clove check '$[user][name]'
  => "Alice"

STEP 3: ARRAY ACCESS
--------------------
Use $[array][index] for array elements (0-indexed).

  echo '{"items": ["a", "b", "c"]}' | clove check '$[items][1]'
  => "b"

STEP 4: ARITHMETIC
------------------
Operators work on extracted values.

  clove check '$[x] * 2' --input '{"x": 21}'
  => 42

STEP 5: FILTERING ARRAYS
------------------------
Use .filter() with @ representing each element.

  clove check '$[nums].filter(@ > 3)' --input '{"nums": [1, 5, 2, 8]}'
  => [5, 8]

STEP 6: TRANSFORMING ARRAYS
---------------------------
Use .map() to transform each element.

  clove check '$[prices].map(@ * 1.1)' --input '{"prices": [10, 20]}'
  => [11, 22]

STEP 7: CHAINING
----------------
Methods can be chained.

  clove check '$[users].filter(@[active]).map(@[name])' \
    --input '{"users": [{"name": "Alice", "active": true}, {"name": "Bob", "active": false}]}'
  => ["Alice"]

NEXT STEPS
----------
  clove docs              List all documentation categories
  clove doc syntax        Basic access notation
  clove doc operators     All operators
  clove doc array-methods Array manipulation
"#
}
