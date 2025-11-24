# Building and Running ToonConv

This guide explains how to build and run the ToonConv JSON-to-TOON converter.

## Prerequisites

- **Rust Toolchain**: Install Rust 1.70 or later
  ```bash
  # Install Rust via rustup
  curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
  
  # Verify installation
  rustc --version
  cargo --version
  ```

## Building from Source

### 1. Clone the Repository

```bash
git clone https://github.com/lst97/toonconv.git
cd toonconv
```

### 2. Build the Project

**Development Build** (with debug symbols, faster compilation):
```bash
cargo build
```

**Release Build** (optimized, recommended for production):
```bash
cargo build --release
```

The compiled binary will be located at:
- Debug: `target/debug/toonconv`
- Release: `target/release/toonconv`

### 3. Run Tests

```bash
# Run all tests
cargo test

# Run TOON specification compliance tests
cargo test --test toon_spec_compliance_test

# Run with verbose output
cargo test -- --nocapture
```

## Running ToonConv

### Basic Usage

**Convert a JSON file to TOON:**
```bash
# Using debug build
./target/debug/toonconv input.json -o output.toon

# Using release build
./target/release/toonconv input.json -o output.toon
```

**Using cargo run:**
```bash
# Development
cargo run -- input.json -o output.toon

# Release mode
cargo run --release -- input.json -o output.toon
```

### Command-Line Options

```bash
toonconv [OPTIONS] <INPUT>

Arguments:
  <INPUT>  Path to input JSON file

Options:
  -o, --output <OUTPUT>    Output file path (default: stdout)
  -p, --pretty             Enable pretty printing with indentation
  -c, --compact            Compact output (minimal whitespace)
  -d, --delimiter <DELIM>  Field delimiter (default: comma)
  -i, --indent <SIZE>      Indentation size in spaces (default: 2)
  -v, --validate           Validate TOON output
  -h, --help               Print help information
  -V, --version            Print version information
```

### Examples

**1. Convert JSON file with pretty formatting:**
```bash
cargo run --release -- examples/sample.json -o output.toon --pretty
```

**2. Convert from stdin:**
```bash
echo '{"name": "Alice", "age": 30}' | cargo run --release -- - -o output.toon
```

**3. Validate output:**
```bash
cargo run --release -- input.json -o output.toon --validate
```

**4. Custom indentation and delimiter:**
```bash
cargo run --release -- input.json -o output.toon --indent 4 --delimiter "|"
```

## Example Input/Output

**Input JSON:**
```json
{
  "users": [
    {"id": 1, "name": "Alice"},
    {"id": 2, "name": "Bob"}
  ],
  "tags": ["rust", "json", "converter"]
}
```

**Output TOON:**
```
users[2]{id,name}:
1,Alice
2,Bob
tags[3]: rust,json,converter
```

## Installing Globally

To install ToonConv as a system-wide command:

```bash
cargo install --path .
```

Then you can run it from anywhere:
```bash
toonconv input.json -o output.toon
```

## Troubleshooting

### Build Errors

**Problem**: Missing dependencies
```bash
# Update Cargo.lock and rebuild
cargo update
cargo clean
cargo build --release
```

**Problem**: Rust version too old
```bash
# Update Rust toolchain
rustup update stable
```

### Runtime Errors

**Problem**: "File not found"
- Ensure the input file path is correct
- Use absolute paths if relative paths don't work

**Problem**: "Invalid JSON"
- Validate your JSON input with a JSON validator
- Check for trailing commas, which are not valid in JSON

## Performance Tips

1. **Use release builds** for production: `cargo build --release`
2. **Process large files** with streaming (if implemented)
3. **Enable validation** only when debugging: `--validate`

## Development Workflow

```bash
# 1. Make code changes
# 2. Run tests
cargo test

# 3. Check formatting
cargo fmt

# 4. Run linter
cargo clippy

# 5. Build release
cargo build --release

# 6. Test the binary
./target/release/toonconv examples/sample.json
```

## Next Steps

- See [README.md](README.md) for project overview
- See [TOON_SPEC.md](specs/TOON_SPEC.md) for format specification
- See [examples/](examples/) for more conversion examples
