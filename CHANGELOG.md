# Changelog

All notable changes to faf-rust-sdk will be documented in this file.

## [Unreleased]

### Added
- Trusted Publishing (OIDC) workflow for crates.io releases (`publish-crate.yml`)
- Weekly cargo-audit security CI (`audit.yml`)

### Changed
- Rust edition 2021 → 2024
- Declared MSRV: `rust-version = "1.85"`

## [2.0.1] - 2026-03-20

### Added
- CHANGELOG.md
- README footer with faf-cli CTA and Anthropic MCP #2759 link
- See Also section with faf-wasm-sdk and mcpaas links

## [2.0.0] - 2026-03-15

### Added
- **FAFb v2 Binary Format** — String table architecture replacing hardcoded section enum
- **Chunk Classification** — DNA, Context, and Pointer section types (IFF-inspired)
- **String Table** — Unlimited section names via ELF/IFF-style string index
- **`compile_v2()`** — Compile any YAML key into a classified binary section
- **`DecompiledFafb`** — Rich query API: `dna_sections()`, `context_sections()`, `pointer_section()`, `get_section_by_name()`
- **Chunk Registry** — Automatic classification of known FAF keys
- **Version-aware decompile** — Handles both v1 and v2 FAFb binaries
- **175 tests** — Full coverage including string table, chunk registry, cross-version compat

### Changed
- Header `version_major` bumped to 2
- `reserved: u16` field repurposed as `string_table_index: u16` (wire layout unchanged)
- Section entry byte 0 now `section_name_index` (was enum `section_type`)
- Classification stored in flags bits 0-1 (zero layout change)

### Preserved
- All v1 tests pass unchanged
- v1 binaries decompile correctly with updated code
- 32-byte header layout fully backward compatible

## [1.3.0] - 2026-03-06

### Added
- Axum integration via `axum` feature flag
- `FafLayer` and `FafContext` for zero-cost per-request context

## [1.2.0] - 2026-02-28

### Added
- FAFb binary format module (v1.0)
- 11 section types with priority truncation
- CRC32 source verification

## [1.1.0] - 2026-02-20

### Changed
- Migrated `serde_yaml` to `serde_yaml_ng` (maintained fork)
- Fixed README install snippet

## [1.0.0] - 2025-11-15

### Added
- Initial release
- FAF YAML parsing with serde
- Validation and scoring
- Compression levels (Minimal, Standard, Full)
- Discovery module
