# c2rust-config

A configuration management tool for C2Rust projects.

## Features

- Persistent configuration storage in `.c2rust/config.toml`
- Support for model-level and feature-level configurations
- CRUD operations on TOML configuration
- Case-insensitive feature names
- Dot notation for nested keys

## Usage

### Prerequisites

Create a `.c2rust` directory in your project root:

```bash
mkdir .c2rust
```

### Model Configuration

Model-level configuration is stored under `[model]` section:

```bash
# Set a scalar value
c2rust-config config --model --set compiler gcc

# Set an array
c2rust-config config --model --set options -O2 -Wall

# Add to array
c2rust-config config --model --add options -Werror

# Remove from array
c2rust-config config --model --del options -Wall

# List values
c2rust-config config --model --list compiler

# Unset a key
c2rust-config config --model --unset compiler
```

### Feature Configuration

Feature-level configuration is stored under `[feature.<name>]` sections:

```bash
# Use default feature
c2rust-config config --make --set build.dir /tmp/build

# Use named feature (case-insensitive)
c2rust-config config --make --feature debug --set build.dir /tmp/debug

# Feature names are normalized to lowercase
c2rust-config config --make --feature DEBUG --set test value
# Same as:
c2rust-config config --make --feature debug --set test value
```

### Nested Keys

Use dot notation for nested configuration:

```bash
c2rust-config config --model --set build.compiler.name gcc
c2rust-config config --model --set build.compiler.version 11
c2rust-config config --model --list build.compiler.name
```

## Configuration File Format

The tool uses TOML format and stores configuration in `.c2rust/config.toml`:

```toml
[model]
compiler = "gcc"
options = ["-O2", "-Wall"]

[feature.default]
[feature.default.build]
dir = "/tmp/build"

[feature.debug]
[feature.debug.build]
dir = "/tmp/debug"
```

## Error Handling

The tool provides clear error messages for common issues:

- Missing `.c2rust` directory: "Directory '.c2rust' does not exist. Please create/initialize it first."
- Missing configuration file: "Configuration file not found"
- Missing feature: "Feature 'name' not found"
- Missing key: "Key 'name' not found"

## Development

### Build

```bash
cargo build
```

### Test

```bash
cargo test
```

### Release

```bash
cargo build --release
```