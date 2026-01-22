# c2rust-config

A Rust configuration management tool for c2rust translation work. This tool manages configuration data stored in `.c2rust/config.toml` files.

## Installation

```bash
cargo build --release
# The binary will be available at target/release/c2rust-config
```

## Usage

The tool provides two main command modes:
- `model`: For model-related configuration
- `make`: For build/clean/test-related configuration

### Basic Commands

```bash
# Set a single value
c2rust-config make set build.dir "build"

# Set multiple values (creates an array)
c2rust-config make set compiler "gcc" "clang"

# Add values to an array
c2rust-config make add build.options "-I../3rd/include -DDEBUG=1"
c2rust-config make add build.files.0 "main.c" "debug.c" "common.c"

# Delete values from an array
c2rust-config make del build.files.0 "debug.c"

# List all values for a key
c2rust-config make list build.dir

# Remove a key
c2rust-config make unset build.dir
```

### Features

Features allow you to maintain multiple configurations. The default feature is `default`.

```bash
# Use a specific feature
c2rust-config make --feature debug set build.dir "debug_build"
c2rust-config make --feature release set build.dir "release_build"
```

Feature names are case-insensitive and will be converted to lowercase.

### Model Configuration

```bash
c2rust-config model set api_key "your-key"
c2rust-config model list api_key
```

## Configuration File Format

The configuration is stored in `.c2rust/config.toml`:

```toml
[model]
api_key = "test-key"

[feature.default]
compiler = "gcc"
"build.dir" = "build"
build = "make"
"build.options" = ["-I../3rd/include -DDEBUG=1", "-I../3rd/include"]
"build.files.0" = ["main.c", "debug.c", "common.c"]
"build.files.1" = ["main.c", "release.c", "common.c"]
```

## Requirements

- The tool searches for `.c2rust` directory by traversing up from the current directory
- The `.c2rust` directory and `config.toml` file must exist before running the tool
- Create them manually if they don't exist:

```bash
mkdir .c2rust
echo "[model]" > .c2rust/config.toml
```

## Error Messages

The tool provides clear, hierarchical error messages:
1. If `.c2rust` directory is not found
2. If `config.toml` file is not found
3. If a feature is not found
4. If a key is not found

## Development

### Running Tests

```bash
cargo test
```

All tests are integration tests located in `tests/integration_test.rs`.