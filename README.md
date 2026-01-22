# c2rust-config

A Rust configuration management tool for c2rust translation work. This tool manages configuration data stored in `.c2rust/config.toml` files.

## Installation

```bash
cargo build --release
# The binary will be available at target/release/c2rust-config
```

## Usage

The tool provides three main command modes:
- `global`: For global configuration (e.g., compiler settings)
- `model`: For model-related configuration
- `make`: For build/clean/test-related configuration

### Basic Commands

```bash
# Global configuration
c2rust-config global set compiler "gcc"
c2rust-config global add compiler "clang"

# Set a single value
c2rust-config make set build.dir "build"

# Set multiple values (creates an array)
c2rust-config make set build "make"

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

### Global Configuration

```bash
c2rust-config global set compiler "gcc"
c2rust-config global list compiler
```

### Model Configuration

```bash
c2rust-config model set api_key "your-key"
c2rust-config model list api_key
```

## Configuration File Format

The configuration is stored in `.c2rust/config.toml`:

```toml
# Global configuration
[global]
compiler = ["gcc"]

# Model-related configuration
[model]

# Feature-specific configuration
[feature.default]
# Relative to project root (.c2rust directory)
"clean.dir" = "build"
clean = "make clean"
# Relative to project root
"test.dir" = "build"
test = "make test"
# Relative to project root
"build.dir" = "build"
build = "make"
# Build options for extracting target files to translate
# Different files may have different compilation options
# One build can generate both debug/release binaries
"build.options" = ["-I../3rd/include -DDEBUG=1", "-I../3rd/include"]
# files.x index corresponds to options index
# Each file list corresponds to one set of compilation options
"build.files.0" = ["main.c", "debug.c", "common.c"]
"build.files.1" = ["main.c", "release.c", "common.c"]
```

## Requirements

- The tool searches for `.c2rust` directory by traversing up from the current directory
- The `.c2rust` directory and `config.toml` file must exist before running the tool
- Create them manually if they don't exist:

```bash
mkdir .c2rust
cat > .c2rust/config.toml << 'EOF'
[global]

[model]
EOF
```

## Error Messages

The tool provides clear, hierarchical error messages:
1. If `.c2rust` directory is not found
2. If `config.toml` file is not found
3. If a feature is not found
4. If a key is not found

## Validation and Warnings

The tool validates feature configurations and provides warnings for:

1. **Incomplete Feature Configuration**: When configuring a feature, all of the following keys must be set together:
   - `clean.dir`
   - `clean`
   - `test.dir`
   - `test`
   - `build.dir`
   - `build`
   
   If some but not all of these keys are present, a warning will be displayed listing the missing keys.

2. **Build Files Index Validation**: The number of `build.files.X` entries should not exceed the length of the `build.options` array. For example, if you have 2 entries in `build.options`, you should only use `build.files.0` and `build.files.1`. Using `build.files.2` or higher will generate a warning.

**Optional Keys**: The following keys are optional and do not trigger warnings:
- `build.options`
- `build.files.0`, `build.files.1`, etc.

## Development

### Running Tests

```bash
cargo test
```

All tests are integration tests located in `tests/integration_test.rs`.