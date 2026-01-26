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
        .args(&["config", "--make", "--set", "build.cmd", "make"])
        .assert()
        .success();
    
    // Set clean directory and command
    get_cmd(&temp_dir)
        .args(&["config", "--make", "--set", "clean.dir", "build"])
        .assert()
        .success();
    
    get_cmd(&temp_dir)
        .args(&["config", "--make", "--set", "clean.cmd", "make clean"])
        .assert()
        .success();
    
    // Set test directory and command
    get_cmd(&temp_dir)
        .args(&["config", "--make", "--set", "test.dir", "build"])
        .assert()
        .success();
    
    get_cmd(&temp_dir)
        .args(&["config", "--make", "--set", "test.cmd", "make test"])
        .assert()
        .success();
    
    // Verify the config
    let config = read_config(&temp_dir);
    assert!(config.contains(r#"compiler = "gcc""#));
    assert!(config.contains(r#""build.dir" = "build""#) || config.contains(r#"build.dir = "build""#));
    assert!(config.contains(r#""build.cmd" = "make""#) || config.contains(r#"build.cmd = "make""#));
    assert!(config.contains(r#""clean.dir" = "build""#) || config.contains(r#"clean.dir = "build""#));
    assert!(config.contains(r#""clean.cmd" = "make clean""#) || config.contains(r#"clean.cmd = "make clean""#));
    assert!(config.contains(r#""test.dir" = "build""#) || config.contains(r#"test.dir = "build""#));
    assert!(config.contains(r#""test.cmd" = "make test""#) || config.contains(r#"test.cmd = "make test""#));
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
        .args(&["config", "--make", "--set", "build.cmd", "make"])
        .assert()
        .success();
    
    get_cmd(&temp_dir)
        .args(&["config", "--make", "--set", "clean.dir", "build"])
        .assert()
        .success();
    
    get_cmd(&temp_dir)
        .args(&["config", "--make", "--set", "clean.cmd", "make clean"])
        .assert()
        .success();
    
    get_cmd(&temp_dir)
        .args(&["config", "--make", "--set", "test.dir", "build"])
        .assert()
        .success();
    
    // Last one should have no warnings
    let output = get_cmd(&temp_dir)
        .args(&["config", "--make", "--set", "test.cmd", "make test"])
        .assert()
        .success()
        .get_output()
        .stderr
        .clone();
    
    let stderr = String::from_utf8(output).unwrap();
    assert!(!stderr.contains("missing required keys"));
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

// ===== Tests for Optional Key Parameter in --list Operation =====

#[test]
fn test_list_specific_key_single_value() {
    let temp_dir = setup_test_env();
    
    // Set up a configuration
    get_cmd(&temp_dir)
        .args(&["config", "--make", "--set", "build.dir", "build"])
        .assert()
        .success();
    
    get_cmd(&temp_dir)
        .args(&["config", "--make", "--set", "clean.cmd", "make clean"])
        .assert()
        .success();
    
    // List specific key - should output only the value
    get_cmd(&temp_dir)
        .args(&["config", "--make", "--list", "build.dir"])
        .assert()
        .success()
        .stdout("build\n");
    
    // List another specific key
    get_cmd(&temp_dir)
        .args(&["config", "--make", "--list", "clean.cmd"])
        .assert()
        .success()
        .stdout("make clean\n");
}

#[test]
fn test_list_specific_key_array_values() {
    let temp_dir = setup_test_env();
    
    // Set up array configuration
    get_cmd(&temp_dir)
        .args(&["config", "--make", "--add", "build.files.0", "main.c", "test.c", "common.c"])
        .assert()
        .success();
    
    // List specific key with array - should output each value on a separate line
    let output = get_cmd(&temp_dir)
        .args(&["config", "--make", "--list", "build.files.0"])
        .assert()
        .success()
        .get_output()
        .stdout
        .clone();
    
    let stdout = String::from_utf8(output).unwrap();
    assert_eq!(stdout, "main.c\ntest.c\ncommon.c\n");
}

#[test]
fn test_list_nonexistent_key() {
    let temp_dir = setup_test_env();
    
    // Set up some configuration
    get_cmd(&temp_dir)
        .args(&["config", "--make", "--set", "build.dir", "build"])
        .assert()
        .success();
    
    // Try to list non-existent key - should fail with KeyNotFound error
    get_cmd(&temp_dir)
        .args(&["config", "--make", "--list", "nonexistent.key"])
        .assert()
        .failure()
        .stderr(predicate::str::contains("key 'nonexistent.key' not found"));
}

#[test]
fn test_list_all_vs_specific_key() {
    let temp_dir = setup_test_env();
    
    // Set up multiple configurations
    get_cmd(&temp_dir)
        .args(&["config", "--make", "--set", "build.dir", "build"])
        .assert()
        .success();
    
    get_cmd(&temp_dir)
        .args(&["config", "--make", "--set", "clean.dir", "build"])
        .assert()
        .success();
    
    get_cmd(&temp_dir)
        .args(&["config", "--make", "--set", "test.dir", "test"])
        .assert()
        .success();
    
    // List all - should show all keys with "key = value" format
    let output_all = get_cmd(&temp_dir)
        .args(&["config", "--make", "--list"])
        .assert()
        .success()
        .get_output()
        .stdout
        .clone();
    
    let stdout_all = String::from_utf8(output_all).unwrap();
    assert!(stdout_all.contains("build.dir = build"));
    assert!(stdout_all.contains("clean.dir = build"));
    assert!(stdout_all.contains("test.dir = test"));
    
    // List specific key - should only show value
    get_cmd(&temp_dir)
        .args(&["config", "--make", "--list", "build.dir"])
        .assert()
        .success()
        .stdout("build\n");
}

#[test]
fn test_list_non_string_values() {
    let temp_dir = setup_test_env();
    
    // Manually create a config with non-string values (integer, boolean)
    let config_path = temp_dir.path().join(".c2rust/config.toml");
    let config_content = r#"[global]

[model]

[feature.default]
port = 8080
debug = true
ratio = 3.14
"#;
    fs::write(&config_path, config_content).unwrap();
    
    // List integer value
    let output = get_cmd(&temp_dir)
        .args(&["config", "--make", "--list", "port"])
        .assert()
        .success()
        .get_output()
        .stdout
        .clone();
    
    let stdout = String::from_utf8(output).unwrap();
    assert_eq!(stdout, "8080\n");
    
    // List boolean value
    let output = get_cmd(&temp_dir)
        .args(&["config", "--make", "--list", "debug"])
        .assert()
        .success()
        .get_output()
        .stdout
        .clone();
    
    let stdout = String::from_utf8(output).unwrap();
    assert_eq!(stdout, "true\n");
    
    // List float value
    let output = get_cmd(&temp_dir)
        .args(&["config", "--make", "--list", "ratio"])
        .assert()
        .success()
        .get_output()
        .stdout
        .clone();
    
    let stdout = String::from_utf8(output).unwrap();
    assert_eq!(stdout, "3.14\n");
}

// ===== Tests for config --set Override Behavior =====

#[test]
fn test_set_override_existing_value() {
    let temp_dir = setup_test_env();
    
    // Set initial value
    get_cmd(&temp_dir)
        .args(&["config", "--make", "--set", "build.dir", "build"])
        .assert()
        .success();
    
    // Verify it was set
    let config = read_config(&temp_dir);
    assert!(config.contains(r#""build.dir" = "build""#) || config.contains(r#"build.dir = "build""#));
    
    // Override with new value
    get_cmd(&temp_dir)
        .args(&["config", "--make", "--set", "build.dir", "new_build"])
        .assert()
        .success();
    
    // Verify override worked - should only have new value
    let config = read_config(&temp_dir);
    assert!(config.contains(r#""build.dir" = "new_build""#) || config.contains(r#"build.dir = "new_build""#));
    assert!(!config.contains(r#"= "build""#) || config.contains(r#"= "new_build""#));
}

#[test]
fn test_nested_structure_flattening() {
    let temp_dir = TempDir::new().unwrap();
    let c2rust_dir = temp_dir.path().join(".c2rust");
    fs::create_dir(&c2rust_dir).unwrap();
    
    // Create a config with nested structure (the problematic format from the issue)
    let config_path = c2rust_dir.join("config.toml");
    let config_content = r#"[global]
compiler = ["gcc"]

[model]
api_key = "your-api-key"

[feature.default.clean]
cmd = "make clean"
dir = "build"

[feature.default.test]
cmd = "make test"
dir = "build"

[feature.default.build]
cmd = "make"
dir = "build"
"#;
    fs::write(&config_path, config_content).unwrap();
    
    // List should work with nested structure
    let output = get_cmd(&temp_dir)
        .args(&["config", "--make", "--list"])
        .assert()
        .success()
        .get_output()
        .stdout
        .clone();
    
    let stdout = String::from_utf8(output).unwrap();
    assert!(stdout.contains("clean.cmd = make clean"));
    assert!(stdout.contains("clean.dir = build"));
    assert!(stdout.contains("test.cmd = make test"));
    assert!(stdout.contains("build.cmd = make"));
    
    // Set a value to override - should flatten the structure
    get_cmd(&temp_dir)
        .args(&["config", "--make", "--set", "build.cmd", "make VERBOSE=1"])
        .assert()
        .success();
    
    let config = read_config(&temp_dir);
    
    // Should have flattened structure now
    assert!(config.contains("[feature.default]"));
    
    // Should NOT have nested sections anymore
    assert!(!config.contains("[feature.default.clean]"));
    assert!(!config.contains("[feature.default.test]"));
    assert!(!config.contains("[feature.default.build]"));
    
    // Should have all values as dotted keys
    assert!(config.contains(r#""clean.cmd" = "make clean""#) || config.contains(r#"clean.cmd = "make clean""#));
    assert!(config.contains(r#""build.cmd" = "make VERBOSE=1""#) || config.contains(r#"build.cmd = "make VERBOSE=1""#));
}

#[test]
fn test_set_single_key_no_duplicates() {
    let temp_dir = setup_test_env();
    
    // Set a key multiple times
    get_cmd(&temp_dir)
        .args(&["config", "--make", "--set", "build.dir", "build"])
        .assert()
        .success();
    
    get_cmd(&temp_dir)
        .args(&["config", "--make", "--set", "build.dir", "debug"])
        .assert()
        .success();
    
    get_cmd(&temp_dir)
        .args(&["config", "--make", "--set", "build.dir", "release"])
        .assert()
        .success();
    
    let config = read_config(&temp_dir);
    
    // Count occurrences of build.dir - should only appear once
    let occurrences = config.matches("build.dir").count();
    assert_eq!(occurrences, 1, "build.dir should only appear once in config, but found {} occurrences", occurrences);
    
    // Should have the latest value
    assert!(config.contains(r#""build.dir" = "release""#) || config.contains(r#"build.dir = "release""#));
}


