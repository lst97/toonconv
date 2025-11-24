# ToonConv Usage Guide

Complete guide to using the JSON-to-TOON converter with all supported input methods.

---

## Quick Start

ToonConv supports **4 different ways** to convert JSON to TOON format:

1. **Direct JSON String Argument** (NEW!)
2. **Standard Input (stdin)**
3. **Single File Conversion**
4. **Directory Batch Conversion**

---

## Method 1: Direct JSON String Argument

Pass JSON directly as a command-line argument - perfect for quick conversions!

### Basic Usage

```bash
# Convert a JSON object
toonconv '{"name": "Alice", "age": 30}'

# Convert a JSON array
toonconv '[1, 2, 3, 4, 5]'

# Nested structures
toonconv '{"user": {"name": "Bob", "email": "bob@example.com"}}'
```

### Example Output

```bash
$ toonconv '{"name": "Alice", "age": 30}'
{2}: name:Alice,age:30

$ toonconv '[1, 2, 3, 4, 5]'
[5]: 1,2,3,4,5
```

### When to Use
- Quick one-off conversions
- Testing TOON format
- Shell scripts with dynamic JSON
- Simple data transformations

---

## Method 2: Standard Input (stdin)

Pipe JSON data directly into toonconv - great for pipelines!

### Basic Usage

```bash
# From echo
echo '{"status": "ok"}' | toonconv --stdin

# From file via cat
cat data.json | toonconv --stdin

# From curl API response
curl -s https://api.example.com/data | toonconv --stdin

# Chain with other commands
jq '.results[]' data.json | toonconv --stdin
```

### Save Output to File

```bash
# Redirect stdout
echo '{"data": [1, 2, 3]}' | toonconv --stdin > output.toon

# Using --output flag
echo '{"data": [1, 2, 3]}' | toonconv --stdin -o output.toon
```

### When to Use
- Unix pipelines and data workflows
- Processing API responses
- Streaming data transformations
- Integration with other tools (jq, curl, etc.)

---

## Method 3: Single File Conversion

Convert individual JSON files to TOON format.

### Basic Usage

```bash
# Convert with default output (same name, .toon extension)
toonconv input.json

# Specify custom output path
toonconv input.json -o custom_output.toon

# Output to different directory
toonconv data/user.json -o results/user.toon
```

### Example

```bash
$ cat user.json
{
  "id": 123,
  "name": "Alice",
  "roles": ["admin", "user"]
}

$ toonconv user.json -o user.toon

$ cat user.toon
{3}: id:123,name:Alice,roles:[2]: admin,user
```

### When to Use
- Converting individual files
- Testing with sample data
- One-time conversions
- Custom output naming needed

---

## Method 4: Directory Batch Conversion

Convert entire directories of JSON files while preserving structure.

### Basic Usage

```bash
# Convert all JSON files (recursive by default)
toonconv input_dir/ --output output_dir/

# Explicit recursive flag
toonconv input_dir/ -o output_dir/ --recursive

# Non-recursive (top-level only)
toonconv input_dir/ -o output_dir/ --no-recursive
```

### Directory Structure Preservation

Input structure:
```
input_dir/
├── users.json
├── config.json
└── data/
    ├── items.json
    └── nested/
        └── deep.json
```

After conversion:
```
output_dir/
├── users.toon
├── config.toon
└── data/
    ├── items.toon
    └── nested/
        └── deep.toon
```

### Error Handling

```bash
# Continue on errors (default behavior)
toonconv input_dir/ -o output_dir/ --continue-on-error

# Stop on first error
toonconv input_dir/ -o output_dir/ --no-continue-on-error
```

### When to Use
- Migrating entire projects
- Batch processing large datasets
- CI/CD pipelines
- Automated backups and conversions

---

## Advanced Options

### Format Control

```bash
# Pretty-print TOON output (default)
toonconv data.json --format pretty

# Compact output (no whitespace)
toonconv data.json --format compact

# Minified output (absolute minimum size)
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

---

## Common Patterns

### Pattern 1: API to TOON Pipeline

```bash
# Fetch, filter, convert
curl -s https://api.example.com/users | \
  jq '.data.users' | \
  toonconv --stdin -o users.toon
```

### Pattern 2: Batch Convert with Filtering

```bash
# Convert all JSON files in directory, skip errors
find . -name "*.json" -type f | while read file; do
  toonconv "$file" -o "${file%.json}.toon" --continue-on-error
done
```

### Pattern 3: Test JSON String Quickly

```bash
# Quick syntax check
toonconv '{"test": true}' && echo "Valid JSON!"

# Compare sizes
echo '{"data": [1,2,3,4,5]}' | toonconv --stdin | wc -c
```

### Pattern 4: Configuration File Conversion

```bash
# Convert all config files
for env in dev staging prod; do
  toonconv config/${env}.json -o config/${env}.toon
done
```

---

## Error Handling Examples

### Invalid JSON

```bash
$ toonconv '{"invalid": json}'
Error: Invalid JSON: expected `,` or `}` at line 1 column 17

$ echo $?
1  # Non-zero exit code for errors
```

### File Not Found

```bash
$ toonconv nonexistent.json
Error: Input path does not exist: nonexistent.json
```

### Memory Limit Exceeded

```bash
$ toonconv huge.json --memory-limit 1048576  # 1MB limit
Error: JSON file too large: 5242880 bytes (limit: 1048576 bytes)
```

### Directory Errors with --continue-on-error

```bash
$ toonconv input/ -o output/ --continue-on-error
Warning: Failed to convert invalid.json: Invalid JSON syntax
Warning: Failed to convert broken.json: Unexpected EOF
Successfully converted: 23/25 files
```

---

## Performance Tips

1. **For large files**: Increase memory limit with `--memory-limit`
2. **For batch operations**: Use `--continue-on-error` to avoid stopping
3. **For scripting**: Use `--quiet` to suppress progress messages
4. **For debugging**: Use `--verbose` or `--debug` for detailed logs

---

## Integration Examples

### With Git Hooks

```bash
#!/bin/bash
# .git/hooks/pre-commit
# Convert JSON configs before commit

for file in config/*.json; do
  toonconv "$file" -o "${file%.json}.toon"
  git add "${file%.json}.toon"
done
```

### With Make

```makefile
# Makefile
JSONS := $(wildcard data/*.json)
TOONS := $(JSONS:.json=.toon)

all: $(TOONS)

%.toon: %.json
	toonconv $< -o $@

clean:
	rm -f $(TOONS)
```

### With npm Scripts

```json
{
  "scripts": {
    "convert": "toonconv data/ -o build/toon/",
    "convert:watch": "watchexec -e json 'npm run convert'"
  }
}
```

---

## Testing Your Conversions

### Quick Validation

```bash
# Test round-trip conversion
original='{"test": [1, 2, 3]}'
echo "$original" | toonconv --stdin | toonconv --stdin --format json
```

### Compare File Sizes

```bash
# See compression ratio
original_size=$(wc -c < data.json)
toon_size=$(toonconv data.json -o data.toon && wc -c < data.toon)
echo "Compression: $((100 - toon_size * 100 / original_size))%"
```

---

## Need Help?

- Run `toonconv --help` for command-line options
- Check `README.md` for architecture details
- See `PHASE5_COMPLETION_SUMMARY.md` for implementation status
- Run test suite: `cargo test` (requires Rust toolchain)

---

## Version Information

- **Format Version**: TOON 1.0
- **CLI Version**: See `Cargo.toml` or run `toonconv --version`
- **Supported JSON**: RFC 8259 compliant

---

**Happy Converting! 🚀**
