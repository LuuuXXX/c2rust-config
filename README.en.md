# c2rust-config

A Rust configuration management tool for c2rust translation work. This tool manages configuration data stored in `.c2rust/config.toml` files.

## Installation

```bash
cargo build --release
# The binary will be available at target/release/c2rust-config
```

## Usage

The tool provides a `config` subcommand with three configuration modes:
- `--global`: For global configuration (e.g., compiler settings)
- `--model`: For model-related configuration (e.g., AI model API keys)
- `--make`: For build/clean/test-related configuration

### Command Structure

```bash
c2rust-config config [MODE] [OPERATION] [KEY] [VALUES...]
```

**Modes** (exactly one required):
- `--global`: Global configuration
- `--model`: Model configuration
- `--make`: Build/test configuration

**Operations** (exactly one required):
- `--set KEY VALUE...`: Set key to value(s)
- `--unset KEY`: Remove a key
- `--add KEY VALUE...`: Add value(s) to an array key
- `--del KEY VALUE...`: Remove value(s) from an array key
- `--list [KEY]`: List all values in the section, or specific key if provided

### Basic Examples

#### Complete Configuration Example

Here's a complete configuration setup with all recommended defaults:

```bash
# Set up global configuration
c2rust-config config --global --set compiler "gcc"

# Set up complete make configuration (default feature)
c2rust-config config --make --set build.dir "build"
c2rust-config config --make --set build.cmd "make"
c2rust-config config --make --set clean.dir "build"
c2rust-config config --make --set clean.cmd "make clean"
c2rust-config config --make --set test.dir "build"
c2rust-config config --make --set test.cmd "make test"
```

#### Individual Operations

```bash
# Global configuration
c2rust-config config --global --set compiler "gcc"
c2rust-config config --global --add compiler "clang"
c2rust-config config --global --list

# Model configuration
c2rust-config config --model --set api_key "your-api-key"
c2rust-config config --model --set model_name "gpt-4"
c2rust-config config --model --list

# Make configuration operations
c2rust-config config --make --set build.dir "build"
c2rust-config config --make --set build.cmd "make"
c2rust-config config --make --list

# Remove a key
c2rust-config config --make --unset build.dir

# Array operations
c2rust-config config --make --add build.flags "-O2" "-Wall"
c2rust-config config --make --del build.flags "-Wall"
```

### Features

Features allow you to maintain multiple configurations for different build scenarios (e.g., debug, release). The default feature is `default`.

**Note**: The `--feature` option can only be used with `--make`.

```bash
# Use a specific feature
c2rust-config config --make --feature debug --set build.dir "debug_build"
c2rust-config config --make --feature debug --set build.cmd "make DEBUG=1"

c2rust-config config --make --feature release --set build.dir "release_build"
c2rust-config config --make --feature release --set build.cmd "make RELEASE=1"

# List configuration for a specific feature
c2rust-config config --make --feature debug --list
```

Feature names are case-insensitive and will be automatically converted to lowercase.

## Configuration File Format

The configuration is stored in `.c2rust/config.toml`:

```toml
[global]
# Global settings, generally no configuration needed
compiler = ["gcc"]

[model]
# AI model related configuration
api_key = "your-api-key"
model_name = "gpt-4"

[feature.default]
# Paths are relative to the project root (the directory containing .c2rust)
"clean.dir" = "build"
"clean.cmd" = "make clean"
"test.dir" = "build"
"test.cmd" = "make test"
"build.dir" = "build"
"build.cmd" = "make"

[feature.debug]
"build.dir" = "debug_build"
"build.cmd" = "make DEBUG=1"
"clean.dir" = "debug_build"
"clean.cmd" = "make clean"
"test.dir" = "debug_build"
"test.cmd" = "make test"
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

## Error Handling

The tool provides clear, hierarchical error messages:

1. **Missing `.c2rust` directory**: Displays an error if the `.c2rust` directory is not found in the current directory
2. **Missing `config.toml` file**: Displays an error if the config file is not found
3. **Feature not found**: When attempting to access a non-existent feature
4. **Key not found**: When attempting to delete or access a non-existent key
5. **Invalid operations**: When command syntax is incorrect (e.g., missing required parameters)

## Validation and Warnings

The tool validates feature configurations and provides warnings for incomplete configurations:

**Complete Feature Configuration**: When configuring a feature with `--make`, all of the following keys should be set together for a complete configuration:
- `clean.dir` - Directory to clean
- `clean.cmd` - Clean command
- `test.dir` - Directory for testing
- `test.cmd` - Test command
- `build.dir` - Directory for build output
- `build.cmd` - Build command

If some but not all of these keys are present, a warning will be displayed listing the missing keys.

Example:
```bash
# Incomplete configuration - will show a warning
c2rust-config config --make --set build.dir "build"
# Warning: Feature 'feature.default' is missing required keys: clean.dir, clean.cmd, test.dir, test.cmd, build.cmd. All of [clean.dir, clean.cmd, test.dir, test.cmd, build.dir, build.cmd] should be configured together.

# Complete configuration - no warning
c2rust-config config --make --set build.dir "build"
c2rust-config config --make --set build.cmd "make"
c2rust-config config --make --set clean.dir "build"
c2rust-config config --make --set clean.cmd "make clean"
c2rust-config config --make --set test.dir "build"
c2rust-config config --make --set test.cmd "make test"
```

## Development

### Running Tests

```bash
cargo test
```

All tests are integration tests located in `tests/integration_test.rs`.

### Project Structure

```
c2rust-config/
├── src/
│   ├── main.rs         # CLI interface and command parsing
│   ├── config.rs       # Configuration file operations
│   ├── operations.rs   # Core operations (set, unset, add, del, list)
│   └── error.rs        # Error handling
├── tests/
│   └── integration_test.rs  # Integration tests
├── Cargo.toml
└── README.md
```

## License

This project is part of the c2rust translation toolkit.