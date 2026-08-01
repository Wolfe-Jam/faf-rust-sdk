# faf-rust-sdk (historical tree)

> **Publish source of truth has moved.**  
> Live crates.io **`faf-rust-sdk` 3.x** is the facade over `faf-kernel` + `faf-fafb`, developed and released from:  
> **[github.com/Wolfe-Jam/faf-rust](https://github.com/Wolfe-Jam/faf-rust)** · map: [CRATE-SUPERSESSION.md](https://github.com/Wolfe-Jam/faf-rust/blob/main/docs/CRATE-SUPERSESSION.md)  
>  
> This repository is a **historical monorepo** (v2-era in-tree SDK). Do **not** publish from here.  
> Install: `faf-rust-sdk = "3"` · Owner: FAF format steward (namespace reserved intentionally).

---

# faf-rust-sdk v2 (archive readme)

**Persistent Project Context for Rust. Parse, validate, score, FAFb (binary).**

**FAF defines. MD instructs. AI codes.**

High-performance Rust SDK for **FAF (Foundational AI-context Format)** — parsing, scoring, validation, and the FAFb binary format.

**IANA Media Type:** `application/vnd.faf+yaml`

## v2 — The Definitive Edition

The definitive binary format for AI context. v2 ships FAFb — the compiled form of `.faf` files, future-proofed for any repo size, complexity, or organization. The FAF creator fell in love with IFF in the 90s — working with the Interchange File Format that Commodore created for the Amiga across early computer graphics engines and apps. That chunked binary architecture influenced everything that came after. Microsoft literally riffed on it with RIFF. IFF got it right the first time.

FAFb brings that same architecture into the AI era: a **string table** replacing the fixed enum, the same pattern ELF and IFF have used for decades. FAF creator realized every YAML key can just become a named section. No limits. No "Unknown" fallback. No artificial ceiling. Sections are classified as DNA (core identity), Context (supplementary), or Pointer (documentation). Works for a solo dev or a 680-engineer enterprise.

This is a significant free upgrade. The SDK is MIT, the format is an IANA-registered open standard, and the binary spec is public. We're making the standard bulletproof so everyone can build on it.

## Installation

```toml
[dependencies]
faf-rust-sdk = "2.0"
```

## Quick Start

```rust
use faf_rust_sdk::{parse, validate, compress, CompressionLevel};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let content = r#"
faf_version: 2.5.0
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

    // Validate
    let result = validate(&faf);
    println!("Valid: {}, Score: {}%", result.valid, result.score);

    // Compress for token optimization
    let minimal = compress(&faf, CompressionLevel::Minimal);

    Ok(())
}
```

## FAFb Binary Format

Compile `.faf` YAML to a sealed binary. YAML is source code, FAFb is the compiled output.

```rust
use faf_rust_sdk::binary::{compile, decompile, CompileOptions};

// Compile YAML → binary
let yaml = "faf_version: 2.5.0\nproject:\n  name: my-project\n";
let opts = CompileOptions { use_timestamp: false };
let bytes = compile(yaml, &opts).unwrap();

// Decompile binary → structured result
let result = decompile(&bytes).unwrap();
let name = result.get_section_string_by_name("project").unwrap();

// Query by classification
let dna = result.dna_sections();         // Core identity sections
let ctx = result.context_sections();     // Supplementary sections
let ptr = result.pointer_section();      // Documentation references
```

### Binary Layout

```
HEADER (32 bytes)       — Magic "FAFB", version, flags, CRC32 checksum
SECTION DATA (variable) — Each YAML key → one section, priority-ordered
STRING TABLE (appended) — Section name index, unlimited names, O(1) lookup
SECTION TABLE (at end)  — 16 bytes per entry: name, priority, offset, length, tokens, classification
```

## Features

### Parsing & Validation

```rust
use faf_rust_sdk::{parse, validate};

let faf = parse(content)?;
let result = validate(&faf);
println!("Score: {}%", result.score);
```

### Compression Levels

Optimize for context window constraints:

```rust
use faf_rust_sdk::{compress, CompressionLevel};

let minimal = compress(&faf, CompressionLevel::Minimal);   // ~150 tokens
let standard = compress(&faf, CompressionLevel::Standard);  // ~400 tokens
let full = compress(&faf, CompressionLevel::Full);           // ~800 tokens
```

### Axum Integration

Add FAF project context to any Axum server:

```toml
[dependencies]
faf-rust-sdk = { version = "2.0", features = ["axum"] }
```

```rust
use axum::{Router, routing::get};
use faf_rust_sdk::axum::{FafLayer, FafContext};

let app: Router = Router::new()
    .route("/", get(handler))
    .layer(FafLayer::new());

async fn handler(faf: FafContext) -> String {
    format!("Project: {}", faf.project_name())
}
```

The `.faf` file is parsed **once** at startup. Per-request cost is a single `Arc::clone`.

## Testing

**175 tests passing** — WJTTC Championship-Grade coverage:

```bash
cargo test
```

## See Also

- **[faf-wasm-sdk](https://github.com/faf-foundation/faf-wasm-sdk)** — Same FAFb format compiled to WASM for browsers and edge compute
- **[mcpaas](https://crates.io/crates/mcpaas)** — Stream FAF context live via Radio Protocol

If `faf-rust-sdk` has been useful, consider starring the repo — it helps others find it.

## Links

- [faf.one](https://faf.one) — project home
- [IANA Registration](https://www.iana.org/assignments/media-types/application/vnd.faf+yaml) — `application/vnd.faf+yaml`
- [FAF on Zenodo](https://doi.org/10.5281/zenodo.18251362) — academic paper
- [FAF on Grokipedia](https://grokipedia.com/page/faf-file-format) — 28 citations

## License

MIT

---

### Get the CLI

> **faf-cli** — The original AI-Context CLI. A must-have for every builder.

```bash
npx faf-cli auto
```

**Anthropic MCP [#2759](https://github.com/modelcontextprotocol/servers/pull/2759)** · **IANA Registered:** `application/vnd.faf+yaml` · [faf.one](https://faf.one) · [npm](https://www.npmjs.com/package/faf-cli)
