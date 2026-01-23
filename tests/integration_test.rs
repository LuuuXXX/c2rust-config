use assert_cmd::Command;
use predicates::prelude::*;
use std::fs;
use tempfile::TempDir;

/// Helper to set up a test environment with .c2rust directory
fn setup_test_env() -> TempDir {
    let temp_dir = TempDir::new().unwrap();
    let c2rust_dir = temp_dir.path().join(".c2rust");
    fs::create_dir(&c2rust_dir).unwrap();
    
    // Create an initial config file with [global] and [model] sections
    let config_path = c2rust_dir.join("config.toml");
    fs::write(&config_path, "[global]\n\n[model]\n").unwrap();
    
    temp_dir
}

/// Helper to get command in test directory
fn get_cmd(temp_dir: &TempDir) -> Command {
    let mut cmd = Command::cargo_bin("c2rust-config").unwrap();
    cmd.current_dir(temp_dir.path());
    cmd
}

/// Helper to read config file
fn read_config(temp_dir: &TempDir) -> String {
    let config_path = temp_dir.path().join(".c2rust/config.toml");
    fs::read_to_string(config_path).unwrap()
}

#[test]
fn test_no_c2rust_directory() {
    let temp_dir = TempDir::new().unwrap();
    let mut cmd = Command::cargo_bin("c2rust-config").unwrap();
    cmd.current_dir(temp_dir.path());
    cmd.args(&["config", "--make", "--list"]);
    
    cmd.assert()
        .failure()
        .stderr(predicate::str::contains(".c2rust directory not found"));
}

#[test]
fn test_make_set_single_value() {
    let temp_dir = setup_test_env();
    
    get_cmd(&temp_dir)
        .args(&["config", "--make", "--set", "build.dir", "build"])
        .assert()
        .success();
    
    let config = read_config(&temp_dir);
    assert!(config.contains("[feature.default]"));
    assert!(config.contains(r#""build.dir" = "build""#) || config.contains(r#"build.dir = "build""#));
}

#[test]
fn test_make_set_multiple_values() {
    let temp_dir = setup_test_env();
    
    get_cmd(&temp_dir)
        .args(&["config", "--make", "--set", "compiler", "gcc", "clang"])
        .assert()
        .success();
    
    let config = read_config(&temp_dir);
    assert!(config.contains("compiler"));
    assert!(config.contains("gcc"));
    assert!(config.contains("clang"));
}

#[test]
fn test_make_add_to_array() {
    let temp_dir = setup_test_env();
    
    // Add first set of values
    get_cmd(&temp_dir)
        .args(&["config", "--make", "--add", "build.files.0", "main.c", "debug.c"])
        .assert()
        .success();
    
    // Add more values
    get_cmd(&temp_dir)
        .args(&["config", "--make", "--add", "build.files.0", "common.c"])
        .assert()
        .success();
    
    let config = read_config(&temp_dir);
    assert!(config.contains("main.c"));
    assert!(config.contains("debug.c"));
    assert!(config.contains("common.c"));
}

#[test]
fn test_make_del_from_array() {
    let temp_dir = setup_test_env();
    
    // Add values
    get_cmd(&temp_dir)
        .args(&["config", "--make", "--add", "build.files.0", "main.c", "debug.c", "test.c"])
        .assert()
        .success();
    
    // Delete a value
    get_cmd(&temp_dir)
        .args(&["config", "--make", "--del", "build.files.0", "debug.c"])
        .assert()
        .success();
    
    let config = read_config(&temp_dir);
    assert!(config.contains("main.c"));
    assert!(config.contains("test.c"));
    assert!(!config.contains("debug.c"));
}

#[test]
fn test_make_list_single_value() {
    let temp_dir = setup_test_env();
    
    get_cmd(&temp_dir)
        .args(&["config", "--make", "--set", "build.dir", "build"])
        .assert()
        .success();
    
    get_cmd(&temp_dir)
        .args(&["config", "--make", "--list"])
        .assert()
        .success()
        .stdout(predicate::str::contains("build.dir = build"));
}

#[test]
fn test_make_list_array_values() {
    let temp_dir = setup_test_env();
    
    get_cmd(&temp_dir)
        .args(&["config", "--make", "--add", "compiler", "gcc", "clang", "msvc"])
        .assert()
        .success();
    
    let output = get_cmd(&temp_dir)
        .args(&["config", "--make", "--list"])
        .assert()
        .success()
        .get_output()
        .stdout
        .clone();
    
    let output_str = String::from_utf8(output).unwrap();
    assert!(output_str.contains("compiler = ["));
    assert!(output_str.contains("gcc"));
    assert!(output_str.contains("clang"));
    assert!(output_str.contains("msvc"));
}

#[test]
fn test_make_unset_key() {
    let temp_dir = setup_test_env();
    
    get_cmd(&temp_dir)
        .args(&["config", "--make", "--set", "build.dir", "build"])
        .assert()
        .success();
    
    get_cmd(&temp_dir)
        .args(&["config", "--make", "--unset", "build.dir"])
        .assert()
        .success();
    
    let config = read_config(&temp_dir);
    assert!(!config.contains("build.dir"));
}


#[test]
fn test_make_list_nonexistent_feature() {
    let temp_dir = setup_test_env();
    
    get_cmd(&temp_dir)
        .args(&["config", "--make", "--feature", "nonexistent", "--list"])
        .assert()
        .failure()
        .stderr(predicate::str::contains("feature 'feature.nonexistent' not found"));
}

#[test]
fn test_make_with_custom_feature() {
    let temp_dir = setup_test_env();
    
    get_cmd(&temp_dir)
        .args(&["config", "--make", "--feature", "debug", "--set", "build.dir", "debug_build"])
        .assert()
        .success();
    
    let config = read_config(&temp_dir);
    assert!(config.contains("[feature.debug]"));
    assert!(config.contains(r#""build.dir" = "debug_build""#) || config.contains(r#"build.dir = "debug_build""#));
}

#[test]
fn test_feature_name_lowercase() {
    let temp_dir = setup_test_env();
    
    get_cmd(&temp_dir)
        .args(&["config", "--make", "--feature", "DEBUG", "--set", "build.dir", "build"])
        .assert()
        .success();
    
    let config = read_config(&temp_dir);
    // Feature name should be lowercase
    assert!(config.contains("[feature.debug]"));
}

#[test]
fn test_model_set() {
    let temp_dir = setup_test_env();
    
    get_cmd(&temp_dir)
        .args(&["config", "--model", "--set", "api_key", "test-key-123"])
        .assert()
        .success();
    
    let config = read_config(&temp_dir);
    assert!(config.contains("[model]"));
    assert!(config.contains(r#"api_key = "test-key-123""#));
}

#[test]
fn test_model_list() {
    let temp_dir = setup_test_env();
    
    get_cmd(&temp_dir)
        .args(&["config", "--model", "--set", "api_key", "test-key-123"])
        .assert()
        .success();
    
    get_cmd(&temp_dir)
        .args(&["config", "--model", "--list"])
        .assert()
        .success()
        .stdout(predicate::str::contains("api_key = test-key-123"));
}

#[test]
fn test_global_set() {
    let temp_dir = setup_test_env();
    
    get_cmd(&temp_dir)
        .args(&["config", "--global", "--set", "compiler", "gcc", "clang"])
        .assert()
        .success();
    
    let config = read_config(&temp_dir);
    assert!(config.contains("[global]"));
    assert!(config.contains("compiler"));
    assert!(config.contains("gcc"));
    assert!(config.contains("clang"));
}

#[test]
fn test_global_list() {
    let temp_dir = setup_test_env();
    
    get_cmd(&temp_dir)
        .args(&["config", "--global", "--set", "compiler", "gcc"])
        .assert()
        .success();
    
    get_cmd(&temp_dir)
        .args(&["config", "--global", "--list"])
        .assert()
        .success()
        .stdout(predicate::str::contains("compiler = gcc"));
}

#[test]
fn test_nested_keys() {
    let temp_dir = setup_test_env();
    
    // Test deeply nested keys
    get_cmd(&temp_dir)
        .args(&["config", "--make", "--set", "build.options.debug", "-g", "-O0"])
        .assert()
        .success();
    
    let config = read_config(&temp_dir);
    assert!(config.contains("-g"));
    assert!(config.contains("-O0"));
}

#[test]
fn test_complex_workflow() {
    let temp_dir = setup_test_env();
    
    // Set compiler
    get_cmd(&temp_dir)
        .args(&["config", "--make", "--set", "compiler", "gcc"])
        .assert()
        .success();
    
    // Set build directory
    get_cmd(&temp_dir)
        .args(&["config", "--make", "--set", "build.dir", "build"])
        .assert()
        .success();
    
    // Set build command
    get_cmd(&temp_dir)
        .args(&["config", "--make", "--set", "build", "make"])
        .assert()
        .success();
    
    // Add build options
    get_cmd(&temp_dir)
        .args(&["config", "--make", "--add", "build.options", "-I../3rd/include -DDEBUG=1"])
        .assert()
        .success();
    
    get_cmd(&temp_dir)
        .args(&["config", "--make", "--add", "build.options", "-I../3rd/include"])
        .assert()
        .success();
    
    // Add files for first option set
    get_cmd(&temp_dir)
        .args(&["config", "--make", "--add", "build.files.0", "main.c", "debug.c", "common.c"])
        .assert()
        .success();
    
    // Add files for second option set
    get_cmd(&temp_dir)
        .args(&["config", "--make", "--add", "build.files.1", "main.c", "release.c", "common.c"])
        .assert()
        .success();
    
    // Set test directory and command
    get_cmd(&temp_dir)
        .args(&["config", "--make", "--set", "test.dir", "build"])
        .assert()
        .success();
    
    get_cmd(&temp_dir)
        .args(&["config", "--make", "--set", "test", "make test"])
        .assert()
        .success();
    
    // Verify the config
    let config = read_config(&temp_dir);
    assert!(config.contains(r#"compiler = "gcc""#));
    assert!(config.contains(r#""build.dir" = "build""#) || config.contains(r#"build.dir = "build""#));
    assert!(config.contains(r#"build = "make""#));
    assert!(config.contains("build.options"));
    assert!(config.contains("-I../3rd/include -DDEBUG=1"));
    assert!(config.contains("-I../3rd/include"));
    assert!(config.contains("build.files.0"));
    assert!(config.contains("main.c"));
    assert!(config.contains("debug.c"));
    assert!(config.contains("release.c"));
    assert!(config.contains("common.c"));
}

#[test]
fn test_no_config_file() {
    let temp_dir = TempDir::new().unwrap();
    let c2rust_dir = temp_dir.path().join(".c2rust");
    fs::create_dir(&c2rust_dir).unwrap();
    // Don't create config.toml
    
    let mut cmd = Command::cargo_bin("c2rust-config").unwrap();
    cmd.current_dir(temp_dir.path());
    cmd.args(&["config", "--make", "--list"]);
    
    cmd.assert()
        .failure()
        .stderr(predicate::str::contains("config.toml file not found"));
}

#[test]
fn test_feature_incomplete_warning() {
    let temp_dir = setup_test_env();
    
    // Set only build.dir, should warn about missing required keys
    let output = get_cmd(&temp_dir)
        .args(&["config", "--make", "--set", "build.dir", "build"])
        .assert()
        .success()
        .get_output()
        .stderr
        .clone();
    
    let stderr = String::from_utf8(output).unwrap();
    assert!(stderr.contains("Warning"));
    assert!(stderr.contains("missing required keys"));
}

#[test]
fn test_feature_complete_no_warning() {
    let temp_dir = setup_test_env();
    
    // Set all required keys
    get_cmd(&temp_dir)
        .args(&["config", "--make", "--set", "build.dir", "build"])
        .assert()
        .success();
    
    get_cmd(&temp_dir)
        .args(&["config", "--make", "--set", "build", "make"])
        .assert()
        .success();
    
    get_cmd(&temp_dir)
        .args(&["config", "--make", "--set", "clean.dir", "build"])
        .assert()
        .success();
    
    get_cmd(&temp_dir)
        .args(&["config", "--make", "--set", "clean", "make clean"])
        .assert()
        .success();
    
    get_cmd(&temp_dir)
        .args(&["config", "--make", "--set", "test.dir", "build"])
        .assert()
        .success();
    
    // Last one should have no warnings
    let output = get_cmd(&temp_dir)
        .args(&["config", "--make", "--set", "test", "make test"])
        .assert()
        .success()
        .get_output()
        .stderr
        .clone();
    
    let stderr = String::from_utf8(output).unwrap();
    assert!(!stderr.contains("missing required keys"));
}

#[test]
fn test_build_files_exceeds_options_warning() {
    let temp_dir = setup_test_env();
    
    // Set up a complete feature first
    get_cmd(&temp_dir)
        .args(&["config", "--make", "--set", "build.dir", "build"])
        .assert()
        .success();
    get_cmd(&temp_dir)
        .args(&["config", "--make", "--set", "build", "make"])
        .assert()
        .success();
    get_cmd(&temp_dir)
        .args(&["config", "--make", "--set", "clean.dir", "build"])
        .assert()
        .success();
    get_cmd(&temp_dir)
        .args(&["config", "--make", "--set", "clean", "make clean"])
        .assert()
        .success();
    get_cmd(&temp_dir)
        .args(&["config", "--make", "--set", "test.dir", "build"])
        .assert()
        .success();
    get_cmd(&temp_dir)
        .args(&["config", "--make", "--set", "test", "make test"])
        .assert()
        .success();
    
    // Add only 2 build.options
    get_cmd(&temp_dir)
        .args(&["config", "--make", "--add", "build.options", "-O2"])
        .assert()
        .success();
    get_cmd(&temp_dir)
        .args(&["config", "--make", "--add", "build.options", "-g"])
        .assert()
        .success();
    
    // Add build.files.0 and build.files.1 (valid)
    get_cmd(&temp_dir)
        .args(&["config", "--make", "--add", "build.files.0", "main.c"])
        .assert()
        .success();
    get_cmd(&temp_dir)
        .args(&["config", "--make", "--add", "build.files.1", "test.c"])
        .assert()
        .success();
    
    // Add build.files.2 (should warn - index 2 exceeds array of length 2)
    let output = get_cmd(&temp_dir)
        .args(&["config", "--make", "--add", "build.files.2", "extra.c"])
        .assert()
        .success()
        .get_output()
        .stderr
        .clone();
    
    let stderr = String::from_utf8(output).unwrap();
    assert!(stderr.contains("Warning"));
    assert!(stderr.contains("build.files.2"));
    assert!(stderr.contains("build.options"));
}

#[test]
fn test_list_empty_section() {
    let temp_dir = setup_test_env();
    
    // List all in empty global section
    get_cmd(&temp_dir)
        .args(&["config", "--global", "--list"])
        .assert()
        .success()
        .stdout(predicate::str::is_empty());
}

#[test]
fn test_list_global() {
    let temp_dir = setup_test_env();
    
    // Set some global values
    get_cmd(&temp_dir)
        .args(&["config", "--global", "--set", "compiler", "gcc"])
        .assert()
        .success();
    
    // List all global configuration
    get_cmd(&temp_dir)
        .args(&["config", "--global", "--list"])
        .assert()
        .success()
        .stdout(predicate::str::contains("compiler = gcc"));
}

#[test]
fn test_list_with_arrays() {
    let temp_dir = setup_test_env();
    
    // Set single value
    get_cmd(&temp_dir)
        .args(&["config", "--make", "--set", "build.dir", "build"])
        .assert()
        .success();
    
    // Add array values
    get_cmd(&temp_dir)
        .args(&["config", "--make", "--add", "build.files.0", "main.c", "test.c"])
        .assert()
        .success();
    
    // List all - should show both single value and array with elements on separate lines
    let output = get_cmd(&temp_dir)
        .args(&["config", "--make", "--list"])
        .assert()
        .success()
        .get_output()
        .stdout
        .clone();
    
    let stdout = String::from_utf8(output).unwrap();
    assert!(stdout.contains("build.dir = build"));
    assert!(stdout.contains("build.files.0 = ["));
    assert!(stdout.contains("main.c"));
    assert!(stdout.contains("test.c"));
}

// ===== Validation Tests =====

#[test]
fn test_list_with_key_single_value() {
    let temp_dir = setup_test_env();
    
    // Set up complete feature configuration
    get_cmd(&temp_dir)
        .args(&["config", "--make", "--set", "build.dir", "build"])
        .assert()
        .success();
    
    get_cmd(&temp_dir)
        .args(&["config", "--make", "--set", "build", "make"])
        .assert()
        .success();
    
    get_cmd(&temp_dir)
        .args(&["config", "--make", "--set", "clean.dir", "build"])
        .assert()
        .success();
    
    get_cmd(&temp_dir)
        .args(&["config", "--make", "--set", "clean", "make clean"])
        .assert()
        .success();
    
    get_cmd(&temp_dir)
        .args(&["config", "--make", "--set", "test.dir", "test"])
        .assert()
        .success();
    
    get_cmd(&temp_dir)
        .args(&["config", "--make", "--set", "test", "make test"])
        .assert()
        .success();
    
    // Query specific single-value keys
    get_cmd(&temp_dir)
        .args(&["config", "--make", "--list", "build.dir"])
        .assert()
        .success()
        .stdout(predicate::str::diff("build\n"));
    
    get_cmd(&temp_dir)
        .args(&["config", "--make", "--list", "build"])
        .assert()
        .success()
        .stdout(predicate::str::diff("make\n"));
    
    get_cmd(&temp_dir)
        .args(&["config", "--make", "--list", "clean.dir"])
        .assert()
        .success()
        .stdout(predicate::str::diff("build\n"));
    
    get_cmd(&temp_dir)
        .args(&["config", "--make", "--list", "clean"])
        .assert()
        .success()
        .stdout(predicate::str::diff("make clean\n"));
    
    get_cmd(&temp_dir)
        .args(&["config", "--make", "--list", "test.dir"])
        .assert()
        .success()
        .stdout(predicate::str::diff("test\n"));
    
    get_cmd(&temp_dir)
        .args(&["config", "--make", "--list", "test"])
        .assert()
        .success()
        .stdout(predicate::str::diff("make test\n"));
}

#[test]
fn test_list_with_key_array_values() {
    let temp_dir = setup_test_env();
    
    // Add array values
    get_cmd(&temp_dir)
        .args(&["config", "--make", "--add", "compiler", "gcc", "clang", "msvc"])
        .assert()
        .success();
    
    // Query array key - each value should be on its own line
    let output = get_cmd(&temp_dir)
        .args(&["config", "--make", "--list", "compiler"])
        .assert()
        .success()
        .get_output()
        .stdout
        .clone();
    
    let stdout = String::from_utf8(output).unwrap();
    assert_eq!(stdout, "gcc\nclang\nmsvc\n");
}

#[test]
fn test_list_with_nonexistent_key() {
    let temp_dir = setup_test_env();
    
    // Set up feature with some data
    get_cmd(&temp_dir)
        .args(&["config", "--make", "--set", "build.dir", "build"])
        .assert()
        .success();
    
    // Query non-existent key
    get_cmd(&temp_dir)
        .args(&["config", "--make", "--list", "nonexistent"])
        .assert()
        .failure()
        .stderr(predicate::str::contains("key 'nonexistent' not found"));
}

#[test]
fn test_list_without_key_unchanged() {
    let temp_dir = setup_test_env();
    
    // Set up some data
    get_cmd(&temp_dir)
        .args(&["config", "--make", "--set", "build.dir", "build"])
        .assert()
        .success();
    
    get_cmd(&temp_dir)
        .args(&["config", "--make", "--set", "build", "make"])
        .assert()
        .success();
    
    // List without key should show all (existing behavior)
    let output = get_cmd(&temp_dir)
        .args(&["config", "--make", "--list"])
        .assert()
        .success()
        .get_output()
        .stdout
        .clone();
    
    let stdout = String::from_utf8(output).unwrap();
    assert!(stdout.contains("build.dir = build"));
    assert!(stdout.contains("build = make"));
}

// ===== Validation Tests =====

#[test]
fn test_validation_no_mode_specified() {
    let temp_dir = setup_test_env();
    
    get_cmd(&temp_dir)
        .args(&["config", "--set", "test", "value"])
        .assert()
        .failure()
        .stderr(predicate::str::contains("Exactly one of --global, --model, or --make must be specified"));
}

#[test]
fn test_validation_multiple_modes() {
    let temp_dir = setup_test_env();
    
    // Test --global and --model together
    get_cmd(&temp_dir)
        .args(&["config", "--global", "--model", "--set", "test", "value"])
        .assert()
        .failure()
        .stderr(predicate::str::contains("cannot be used with"));
}

#[test]
fn test_validation_no_operation_specified() {
    let temp_dir = setup_test_env();
    
    get_cmd(&temp_dir)
        .args(&["config", "--global", "test", "value"])
        .assert()
        .failure()
        .stderr(predicate::str::contains("Exactly one of --set, --unset, --add, --del, or --list must be specified"));
}

#[test]
fn test_validation_multiple_operations() {
    let temp_dir = setup_test_env();
    
    // Test --set and --unset together
    get_cmd(&temp_dir)
        .args(&["config", "--global", "--set", "--unset", "test", "value"])
        .assert()
        .failure()
        .stderr(predicate::str::contains("cannot be used with"));
}

#[test]
fn test_validation_feature_without_make() {
    let temp_dir = setup_test_env();
    
    // Test --feature with --global
    get_cmd(&temp_dir)
        .args(&["config", "--global", "--feature", "debug", "--set", "test", "value"])
        .assert()
        .failure()
        .stderr(predicate::str::contains("--feature can only be used with --make"));
    
    // Test --feature with --model
    get_cmd(&temp_dir)
        .args(&["config", "--model", "--feature", "debug", "--set", "test", "value"])
        .assert()
        .failure()
        .stderr(predicate::str::contains("--feature can only be used with --make"));
}

#[test]
fn test_validation_feature_with_make_works() {
    let temp_dir = setup_test_env();
    
    // This should succeed (validation should pass)
    get_cmd(&temp_dir)
        .args(&["config", "--make", "--feature", "debug", "--set", "compiler", "gcc"])
        .assert()
        .success();
}


