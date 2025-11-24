# toonconv Development Guidelines

## Project Description

**toonconv** is a high-performance Rust CLI tool designed to convert JSON data
into TOON (Token-Oriented Object Notation) format. It supports various input
methods including direct strings, standard input, single files, and recursive
directory batch processing.

## Active Technologies

- **Language**: Rust 1.75+ (stable)
- **Core Libraries**:
  - `serde`, `serde_json`: JSON parsing and serialization
  - `clap`: Command-line argument parsing
  - `walkdir`: Recursive directory traversal
  - `anyhow`, `thiserror`: Error handling
  - `indicatif`, `console`: Terminal UI and progress bars
  - `simd-json`: Optional SIMD-accelerated JSON parsing

## Project Structure

```text
toonconv/
├── src/
│   ├── cli/          # CLI argument definitions and parsing
│   ├── conversion/   # Core conversion logic
│   ├── formatter/    # TOON format output generation
│   ├── parser/       # JSON parsing logic
│   ├── validation/   # Input validation
│   ├── lib.rs        # Library entry point
│   └── main.rs       # Binary entry point
├── tests/            # Integration tests
├── benches/          # Performance benchmarks
├── specs/            # TOON format specifications
└── examples/         # Usage examples
```

## Commands

### Build & Run

- `cargo build --release`: Build optimized binary
- `cargo run -- [args]`: Run the tool locally

### Testing & Quality

- `cargo test`: Run unit and integration tests
- `cargo clippy`: Run linter
- `cargo fmt`: Format code
- `cargo bench`: Run benchmarks

## Usage Overview

The tool supports 4 main modes:

1. **Direct String**: `toonconv '{"a":1}'`
2. **Stdin**: `echo '{"a":1}' | toonconv --stdin`
3. **File**: `toonconv input.json -o output.toon`
4. **Directory**: `toonconv input_dir/ -o output_dir/`

See [USAGE_GUIDE.md](USAGE_GUIDE.md) for detailed documentation.

## Recent Changes

- Implemented Phase 5: Directory batch conversion and recursive processing.
- Added support for direct JSON string arguments.
- Enhanced error handling and progress reporting.

<!-- MANUAL ADDITIONS START -->
<!-- MANUAL ADDITIONS END -->
