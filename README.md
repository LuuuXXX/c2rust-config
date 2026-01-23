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

# List all values in the section
c2rust-config make list

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
c2rust-config global list
```

### Model Configuration

```bash
c2rust-config model set api_key "your-key"
c2rust-config model list
```

## Configuration File Format

The configuration is stored in `.c2rust/config.toml`:

```toml
[global]
# 一般无需配置.
compiler = ["gcc"]

# 大模型相关的配置
[model]

#具体feature的配置
[feature.default]
# 相对工程根目录，即.c2rust所在目录的相对路径
"clean.dir" = "build"
clean = "make clean"
# 相对工程根目录，即.c2rust所在目录的相对路径
"test.dir" = "build"
test = "make test"
# 相对工程根目录，即.c2rust所在目录的相对路径
"build.dir" = "build"
build = "make"
```

## Requirements

- The tool searches for `.c2rust` directory in the current directory only (does not traverse parent directories)
- The `.c2rust` directory and `config.toml` file must exist in the current directory before running the tool
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

## Development

### Running Tests

```bash
cargo test
```

All tests are integration tests located in `tests/integration_test.rs`.