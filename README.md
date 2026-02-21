# clove-lang

A JSON query language for filtering, transforming, and validating JSON documents.

## Installation

### CLI

```bash
cargo install clove-lang
```

### Library

```toml
# Full (includes CLI binary)
clove-lang = "0.2"

# Library only (no clap/atty dependencies)
clove-lang = { version = "0.2", default-features = false }
```

## Quick Start

### CLI Usage

```bash
# Query JSON
echo '{"users": [{"name": "Alice", "age": 30}]}' | clove '$[users].filter(x => x[age] > 25)'

# Check assertions
echo '{"status": 200}' | clove check '$[status] == 200'

# Built-in docs
clove docs
```

### Library Usage

```rust
use clove_lang::{Lexer, Parser, Evaluator, EvalContext};
use serde_json::json;

let input = json!({"name": "Alice", "scores": [95, 87, 92]});
let query = "$[scores].filter(x => x > 90)";

let tokens = Lexer::new(query).tokenize().unwrap();
let statements = Parser::new(tokens).parse().unwrap();
let mut ctx = EvalContext::new(&input);
let result = Evaluator::new().eval_query(&statements, &mut ctx).unwrap();

println!("{}", result); // [95, 92]
```

The `cli` module is also available without the `cli` feature — it provides `execute_check`, docs generation, and conversion utilities with no extra dependencies:

```rust
use clove_lang::cli::{CheckOptions, execute_check};

let result = execute_check(&json_input, &CheckOptions {
    expression: "$[status] == 200".to_string(),
    ..Default::default()
});
```

## Language Features

- **Field access**: `$[field]`, `$[nested][field]`, `$.field`
- **Array indexing**: `$[items][0]`, `$[items][-1]`
- **Filtering**: `$[items].filter(x => x[price] > 10)`
- **Transforms**: `$[items] -> $[name] = "updated"`
- **Methods**: `.count()`, `.sum()`, `.map()`, `.sort()`, `.first()`, `.last()`, `.any()`, `.all()`, `.contains()`, `.upper()`, `.lower()`, `.unique()`, `.exists()`
- **Comparisons**: `==`, `!=`, `>`, `<`, `>=`, `<=`
- **Logic**: `&&`, `||`
- **Arithmetic**: `+`, `-`, `*`, `/`, `%`
- **Existence checks**: `$[field]?`
- **String concatenation**: `$[first] + " " + $[last]`
- **Environment variables**: `$ENV[VAR_NAME]`

See [REFERENCE.md](REFERENCE.md) for the full language specification.

## Feature Flags

| Feature | Default | Description |
|---------|---------|-------------|
| `cli` | Yes | Enables the `clove` binary (adds `clap` and `atty` dependencies) |

## License

MIT
