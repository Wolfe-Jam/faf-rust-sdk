# faf-sdk (Rust)

High-performance Rust SDK for **FAF (Foundational AI-context Format)** - optimized for inference workloads.

**IANA Media Type:** `application/vnd.faf+yaml`

## Installation

```toml
[dependencies]
faf-sdk = "1.0"
```

## Quick Start

```rust
use faf_sdk::{parse, validate, compress, CompressionLevel};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let content = r#"
faf_version: 2.5.0
ai_score: "85%"
project:
  name: my-app
  goal: Build something great
instant_context:
  what_building: CLI tool
  tech_stack: Rust, Python
  key_files:
    - src/main.rs
stack:
  backend: Rust
"#;

    // Parse
    let faf = parse(content)?;

    // Access
    println!("Project: {}", faf.project_name());
    println!("Stack: {:?}", faf.tech_stack());
    println!("Score: {:?}", faf.score());

    // Validate
    let result = validate(&faf);
    println!("Valid: {}, Score: {}%", result.valid, result.score);

    // Compress for token optimization
    let minimal = compress(&faf, CompressionLevel::Minimal);

    Ok(())
}
```

## Features

### Zero-Copy Parsing

Designed for high-throughput inference:

```rust
use faf_sdk::parse;

// Parse in ~1ms
let faf = parse(content)?;

// Direct field access - no allocation
let name = faf.project_name();
let stack = faf.tech_stack();
```

### Compression Levels

Optimize for context window constraints:

```rust
use faf_sdk::{compress, CompressionLevel};

// Level 1: ~150 tokens
let minimal = compress(&faf, CompressionLevel::Minimal);

// Level 2: ~400 tokens
let standard = compress(&faf, CompressionLevel::Standard);

// Level 3: ~800 tokens
let full = compress(&faf, CompressionLevel::Full);
```

### Validation

Check structure and completeness:

```rust
use faf_sdk::validate;

let result = validate(&faf);
if result.valid {
    println!("Score: {}%", result.score);
} else {
    println!("Errors: {:?}", result.errors);
}
```

## API

### Core Functions

| Function | Description |
|----------|-------------|
| `parse(content)` | Parse YAML string |
| `parse_file(path)` | Parse from file |
| `validate(&faf)` | Validate structure |
| `compress(&faf, level)` | Compress for tokens |
| `stringify(&faf)` | Convert back to YAML |

### FafFile Methods

| Method | Returns | Description |
|--------|---------|-------------|
| `project_name()` | `&str` | Project name |
| `goal()` | `Option<&str>` | Project goal |
| `score()` | `Option<u8>` | AI score (0-100) |
| `tech_stack()` | `Option<&str>` | Technology stack |
| `what_building()` | `Option<&str>` | What's being built |
| `key_files()` | `&[String]` | Key file paths |
| `is_high_quality()` | `bool` | Score >= 70% |

## Performance

Optimized for inference workloads:

| Operation | Time |
|-----------|------|
| Parse | <1ms |
| Validate | <0.1ms |
| Compress | <0.1ms |

## Why Rust?

For native AI inference embedding:

- **Zero-copy** where possible
- **No GC** pauses
- **Predictable** latency
- **Easy FFI** to Python/C++

## Links

- **Spec:** [github.com/Wolfe-Jam/faf](https://github.com/Wolfe-Jam/faf)
- **Site:** [faf.one](https://faf.one)
- **Python SDK:** [faf-python-sdk](https://github.com/Wolfe-Jam/faf-python-sdk)

## License

MIT
