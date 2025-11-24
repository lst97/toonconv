# toonconv 🦀

[![Rust Version](https://img.shields.io/badge/rust-1.75+-blue.svg)](https://www.rust-lang.org/)
[![License](https://img.shields.io/badge/license-MIT-green.svg)](LICENSE)
[![GitHub stars](https://img.shields.io/github/stars/lst97/toonconv.svg)](https://github.com/lst97/toonconv/stargazers)
[![GitHub issues](https://img.shields.io/github/issues/lst97/toonconv.svg)](https://github.com/lst97/toonconv/issues)

**toonconv** is a blazingly fast, high-performance Rust CLI tool for converting JSON data into TOON (Token-Oriented Object Notation) format. It's designed for LLM applications, data processing pipelines, and scenarios where token efficiency matters.

## ✨ Key Features

- **🚀 Ultra-Fast Performance**: ~190x faster than JavaScript implementations
- **💾 Memory Efficient**: Optional SIMD acceleration and configurable memory limits
- **📦 Multiple Input Methods**: Direct strings, stdin, files, and recursive directories
- **🔄 Batch Processing**: Process entire directory structures automatically
- **📊 Token Optimization**: Reduces token count by 35-54% compared to JSON
- **🛡️ Robust Error Handling**: Continues processing on errors with detailed reporting
- **📈 Built-in Performance Monitoring**: Real-time progress bars and statistics

## 🚀 Performance Benchmarks

| Implementation | CLI Execution Time | Token Efficiency |
|----------------|--------------------|------------------|
| **toonconv (Rust)** | **~5ms** | **35-54% reduction** |
| @toon-format/cli (JS) | ~959ms | Baseline |

> Benchmarks run on Apple M1 Mac Air. See [BENCHMARK.md](BENCHMARK.md) for detailed analysis.

## 📦 Installation

### Using Cargo (Easiest)

```bash
cargo install toonconv
```

### From Source

```bash
# Clone the repository
git clone https://github.com/lst97/toonconv.git
cd toonconv

# Build release version
cargo build --release

# Install globally (optional)
cargo install --path .
```

### Prerequisites

- Rust 1.75+ ([Install Rust](https://rustup.rs/))

## 🎯 Quick Start

### Basic Usage

```bash
# Direct JSON string (NEW!)
toonconv '{"name": "Alice", "age": 30}'

# From file
toonconv input.json -o output.toon

# From stdin
echo '{"status": "ok"}' | toonconv --stdin

# Batch directory conversion
toonconv input_dir/ -o output_dir/
```

### Example Output

```bash
# Input JSON
{"users": [{"id": 1, "name": "Alice"}, {"id": 2, "name": "Bob"}]}

# Output TOON
users[2]{id,name}: 1,Alice 2,Bob
```

## 📚 Usage Methods

toonconv supports **4 different ways** to convert JSON to TOON:

### 1. Direct JSON String Argument 🎯

Perfect for quick conversions and testing.

```bash
# Convert a JSON object
toonconv '{"name": "Alice", "age": 30}'

# Convert a JSON array
toonconv '[1, 2, 3, 4, 5]'

# Nested structures
toonconv '{"user": {"name": "Bob", "email": "bob@example.com"}}'
```

### 2. Standard Input (stdin) 🔄

Great for Unix pipelines and data workflows.

```bash
# From echo
echo '{"status": "ok"}' | toonconv --stdin

# From API response
curl -s https://api.example.com/data | toonconv --stdin

# Chain with jq
jq '.results[]' data.json | toonconv --stdin
```

### 3. Single File Conversion 📄

Convert individual files with custom output paths.

```bash
# Basic conversion
toonconv input.json

# Custom output
toonconv input.json -o custom_output.toon

# Pretty formatting
toonconv input.json --format pretty
```

### 4. Directory Batch Conversion 📁

Process entire directories while preserving structure.

```bash
# Recursive conversion (default)
toonconv input_dir/ -o output_dir/

# Non-recursive
toonconv input_dir/ -o output_dir/ --no-recursive

# Continue on errors
toonconv input_dir/ -o output_dir/ --continue-on-error
```

## ⚙️ Advanced Options

### Format Control

```bash
# Pretty-print output (default)
toonconv data.json --format pretty

# Compact output
toonconv data.json --format compact

# Minified output
toonconv data.json --format minified
```

### Memory Management

```bash
# Set memory limit (default: 512MB)
toonconv large.json --memory-limit 1073741824  # 1GB

# For very large files
toonconv huge.json --memory-limit 2147483648   # 2GB
```

### Verbosity

```bash
# Quiet mode (errors only)
toonconv data.json --quiet

# Verbose mode (detailed progress)
toonconv data.json --verbose

# Debug mode (maximum detail)
toonconv data.json --debug
```

## 🏗️ Architecture

```
toonconv/
├── src/
│   ├── cli/              # CLI argument parsing and commands
│   ├── conversion/       # Core conversion engine
│   ├── formatter/        # TOON format output generation
│   ├── parser/           # JSON parsing with validation
│   ├── validation/       # Input validation and error handling
│   ├── error/            # Custom error types
│   ├── lib.rs            # Library entry point
│   └── main.rs           # Binary entry point
├── tests/                # Comprehensive test suite
├── benches/              # Performance benchmarks
├── examples/             # Usage examples
└── specs/                # TOON format specifications
```

### Core Technologies

- **Language**: Rust 1.75+
- **JSON Processing**: `serde`, `serde_json`
- **CLI Framework**: `clap` with derive macros
- **Performance**: Optional `simd-json` support
- **Terminal UI**: `indicatif`, `console`
- **File Operations**: `walkdir` for recursive traversal

## 🧪 Testing

Run the comprehensive test suite:

```bash
# All tests
cargo test

# TOON specification compliance
cargo test --test toon_spec_compliance_test

# Performance benchmarks
cargo bench

# Code quality
cargo clippy
cargo fmt
```

## 📊 Benchmarking

Compare performance with the official JavaScript implementation:

```bash
# Run built-in benchmarks
cargo bench --bench token_efficiency
cargo bench --bench speed_comparison

# Compare with JavaScript version
node bench/speed.js
```

See [BENCHMARK.md](BENCHMARK.md) for detailed performance analysis.

## 🔧 Development

### Building

```bash
# Development build
cargo build

# Release build (optimized)
cargo build --release

# Check code quality
cargo clippy
cargo fmt --check
```

### Contributing

1. Fork the repository
2. Create a feature branch: `git checkout -b feature-name`
3. Make your changes and add tests
4. Run the test suite: `cargo test`
5. Submit a pull request

See [AGENTS.md](AGENTS.md) for development guidelines.

## 📖 Documentation

- **[USAGE_GUIDE.md](USAGE_GUIDE.md)**: Comprehensive usage examples and patterns
- **[BUILD.md](BUILD.md)**: Build instructions and troubleshooting
- **[BENCHMARK.md](BENCHMARK.md)**: Performance analysis and comparisons
- **[specs/](specs/)**: TOON format specifications

## 💡 Common Patterns

### API to TOON Pipeline

```bash
# Fetch, filter, convert
curl -s https://api.example.com/users | \
  jq '.data.users' | \
  toonconv --stdin -o users.toon
```

### Batch Convert with Error Handling

```bash
# Convert all JSON files, skip errors
find . -name "*.json" -type f | while read file; do
  toonconv "$file" -o "${file%.json}.toon" --continue-on-error
done
```

### Git Hook Integration

```bash
#!/bin/bash
# .git/hooks/pre-commit
for file in config/*.json; do
  toonconv "$file" -o "${file%.json}.toon"
  git add "${file%.json}.toon"
done
```

## 🆘 Troubleshooting

### Build Issues

```bash
# Clean build
cargo clean
cargo update
cargo build --release
```

### Runtime Errors

- **Invalid JSON**: Validate input with a JSON validator
- **Memory limits**: Increase with `--memory-limit` flag
- **File not found**: Use absolute paths if relative paths fail

## 📄 License

This project is licensed under the MIT License - see the [LICENSE](LICENSE) file for details.

## 🙏 Acknowledgments

- TOON format specification contributors
- Rust ecosystem for excellent tooling
- Performance benchmarking frameworks

---

<div align="center">

**[Website](https://github.com/lst97/toonconv)** •
**[Documentation](https://github.com/lst97/toonconv/wiki)** •
**[Issues](https://github.com/lst97/toonconv/issues)** •
**[Discussions](https://github.com/lst97/toonconv/discussions)**

Made with ❤️ using Rust

</div>
