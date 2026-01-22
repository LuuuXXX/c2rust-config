use assert_cmd::Command;
use predicates::prelude::*;
use std::fs;
use tempfile::TempDir;

#[test]
fn test_missing_c2rust_directory() {
    let temp_dir = TempDir::new().unwrap();
    
    let mut cmd = Command::cargo_bin("c2rust-config").unwrap();
    cmd.current_dir(temp_dir.path())
        .args(&["config", "--model", "--set", "test_key", "value"]);
    
    cmd.assert()
        .failure()
        .stderr(predicate::str::contains("does not exist"));
}

#[test]
fn test_missing_config_file_on_list() {
    let temp_dir = TempDir::new().unwrap();
    let c2rust_dir = temp_dir.path().join(".c2rust");
    fs::create_dir(&c2rust_dir).unwrap();
    
    let mut cmd = Command::cargo_bin("c2rust-config").unwrap();
    cmd.current_dir(temp_dir.path())
        .args(&["config", "--model", "--list", "test_key"]);
    
    cmd.assert()
        .failure()
        .stderr(predicate::str::contains("not found"));
}

#[test]
fn test_model_set_scalar() {
    let temp_dir = TempDir::new().unwrap();
    let c2rust_dir = temp_dir.path().join(".c2rust");
    fs::create_dir(&c2rust_dir).unwrap();
    
    // Set a scalar value
    let mut cmd = Command::cargo_bin("c2rust-config").unwrap();
    cmd.current_dir(temp_dir.path())
        .args(&["config", "--model", "--set", "compiler", "gcc"]);
    cmd.assert().success();
    
    // Verify it was saved
    let config_path = c2rust_dir.join("config.toml");
    let content = fs::read_to_string(&config_path).unwrap();
    assert!(content.contains("compiler = \"gcc\""));
    
    // List the value
    let mut cmd = Command::cargo_bin("c2rust-config").unwrap();
    cmd.current_dir(temp_dir.path())
        .args(&["config", "--model", "--list", "compiler"]);
    cmd.assert()
        .success()
        .stdout(predicate::eq("gcc\n"));
}

#[test]
fn test_model_set_array() {
    let temp_dir = TempDir::new().unwrap();
    let c2rust_dir = temp_dir.path().join(".c2rust");
    fs::create_dir(&c2rust_dir).unwrap();
    
    // Set an array value
    let mut cmd = Command::cargo_bin("c2rust-config").unwrap();
    cmd.current_dir(temp_dir.path())
        .args(&["config", "--model", "--set", "options", "opt1", "opt2", "opt3"]);
    cmd.assert().success();
    
    // List the values
    let mut cmd = Command::cargo_bin("c2rust-config").unwrap();
    cmd.current_dir(temp_dir.path())
        .args(&["config", "--model", "--list", "options"]);
    cmd.assert()
        .success()
        .stdout(predicate::eq("opt1\nopt2\nopt3\n"));
}

#[test]
fn test_model_add_to_array() {
    let temp_dir = TempDir::new().unwrap();
    let c2rust_dir = temp_dir.path().join(".c2rust");
    fs::create_dir(&c2rust_dir).unwrap();
    
    // Set initial array
    Command::cargo_bin("c2rust-config").unwrap()
        .current_dir(temp_dir.path())
        .args(&["config", "--model", "--set", "flags", "flag1"])
        .assert().success();
    
    // Add to array
    Command::cargo_bin("c2rust-config").unwrap()
        .current_dir(temp_dir.path())
        .args(&["config", "--model", "--add", "flags", "flag2", "flag3"])
        .assert().success();
    
    // List the values
    Command::cargo_bin("c2rust-config").unwrap()
        .current_dir(temp_dir.path())
        .args(&["config", "--model", "--list", "flags"]);
    
    // Verify content
    let config_path = c2rust_dir.join("config.toml");
    let content = fs::read_to_string(&config_path).unwrap();
    assert!(content.contains("flag1"));
    assert!(content.contains("flag2"));
    assert!(content.contains("flag3"));
}

#[test]
fn test_model_del_from_array() {
    let temp_dir = TempDir::new().unwrap();
    let c2rust_dir = temp_dir.path().join(".c2rust");
    fs::create_dir(&c2rust_dir).unwrap();
    
    // Set initial array
    Command::cargo_bin("c2rust-config").unwrap()
        .current_dir(temp_dir.path())
        .args(&["config", "--model", "--set", "items", "a", "b", "c", "d"])
        .assert().success();
    
    // Delete from array
    Command::cargo_bin("c2rust-config").unwrap()
        .current_dir(temp_dir.path())
        .args(&["config", "--model", "--del", "items", "b", "d"])
        .assert().success();
    
    // List remaining values
    Command::cargo_bin("c2rust-config").unwrap()
        .current_dir(temp_dir.path())
        .args(&["config", "--model", "--list", "items"])
        .assert()
        .success()
        .stdout(predicate::eq("a\nc\n"));
}

#[test]
fn test_model_unset() {
    let temp_dir = TempDir::new().unwrap();
    let c2rust_dir = temp_dir.path().join(".c2rust");
    fs::create_dir(&c2rust_dir).unwrap();
    
    // Set a value
    Command::cargo_bin("c2rust-config").unwrap()
        .current_dir(temp_dir.path())
        .args(&["config", "--model", "--set", "temp_key", "temp_value"])
        .assert().success();
    
    // Unset the value
    Command::cargo_bin("c2rust-config").unwrap()
        .current_dir(temp_dir.path())
        .args(&["config", "--model", "--unset", "temp_key"])
        .assert().success();
    
    // Try to list - should fail
    Command::cargo_bin("c2rust-config").unwrap()
        .current_dir(temp_dir.path())
        .args(&["config", "--model", "--list", "temp_key"])
        .assert()
        .failure()
        .stderr(predicate::str::contains("not found"));
}

#[test]
fn test_feature_default() {
    let temp_dir = TempDir::new().unwrap();
    let c2rust_dir = temp_dir.path().join(".c2rust");
    fs::create_dir(&c2rust_dir).unwrap();
    
    // Set value for default feature
    Command::cargo_bin("c2rust-config").unwrap()
        .current_dir(temp_dir.path())
        .args(&["config", "--make", "--set", "build.dir", "/tmp/build"])
        .assert().success();
    
    // Verify the config file structure
    let config_path = c2rust_dir.join("config.toml");
    let content = fs::read_to_string(&config_path).unwrap();
    assert!(content.contains("[feature.default]"));
    assert!(content.contains("dir = \"/tmp/build\""));
    
    // List the value
    Command::cargo_bin("c2rust-config").unwrap()
        .current_dir(temp_dir.path())
        .args(&["config", "--make", "--list", "build.dir"])
        .assert()
        .success()
        .stdout(predicate::eq("/tmp/build\n"));
}

#[test]
fn test_feature_case_insensitive() {
    let temp_dir = TempDir::new().unwrap();
    let c2rust_dir = temp_dir.path().join(".c2rust");
    fs::create_dir(&c2rust_dir).unwrap();
    
    // Set value with uppercase DEFAULT
    Command::cargo_bin("c2rust-config").unwrap()
        .current_dir(temp_dir.path())
        .args(&["config", "--make", "--feature", "DEFAULT", "--set", "test", "value1"])
        .assert().success();
    
    // Set value with lowercase default
    Command::cargo_bin("c2rust-config").unwrap()
        .current_dir(temp_dir.path())
        .args(&["config", "--make", "--feature", "default", "--set", "test2", "value2"])
        .assert().success();
    
    // Verify both are in the same feature section
    let config_path = c2rust_dir.join("config.toml");
    let content = fs::read_to_string(&config_path).unwrap();
    
    // Should only have one [feature.default] section
    let default_count = content.matches("[feature.default]").count();
    assert_eq!(default_count, 1, "Should have exactly one [feature.default] section");
    
    // Both keys should be present
    assert!(content.contains("test = \"value1\""));
    assert!(content.contains("test2 = \"value2\""));
}

#[test]
fn test_feature_named() {
    let temp_dir = TempDir::new().unwrap();
    let c2rust_dir = temp_dir.path().join(".c2rust");
    fs::create_dir(&c2rust_dir).unwrap();
    
    // Set value for custom feature
    Command::cargo_bin("c2rust-config").unwrap()
        .current_dir(temp_dir.path())
        .args(&["config", "--make", "--feature", "debug", "--set", "compiler", "clang"])
        .assert().success();
    
    // Verify the config file structure
    let config_path = c2rust_dir.join("config.toml");
    let content = fs::read_to_string(&config_path).unwrap();
    assert!(content.contains("[feature.debug]"));
    assert!(content.contains("compiler = \"clang\""));
}

#[test]
fn test_feature_missing_on_list() {
    let temp_dir = TempDir::new().unwrap();
    let c2rust_dir = temp_dir.path().join(".c2rust");
    fs::create_dir(&c2rust_dir).unwrap();
    
    // Create config file
    fs::write(c2rust_dir.join("config.toml"), "[model]\n").unwrap();
    
    // Try to list from non-existent feature
    Command::cargo_bin("c2rust-config").unwrap()
        .current_dir(temp_dir.path())
        .args(&["config", "--make", "--feature", "nonexistent", "--list", "key"])
        .assert()
        .failure()
        .stderr(predicate::str::contains("Feature"));
}

#[test]
fn test_key_not_found_on_list() {
    let temp_dir = TempDir::new().unwrap();
    let c2rust_dir = temp_dir.path().join(".c2rust");
    fs::create_dir(&c2rust_dir).unwrap();
    
    // Create config with a feature
    Command::cargo_bin("c2rust-config").unwrap()
        .current_dir(temp_dir.path())
        .args(&["config", "--make", "--set", "existing", "value"])
        .assert().success();
    
    // Try to list non-existent key
    Command::cargo_bin("c2rust-config").unwrap()
        .current_dir(temp_dir.path())
        .args(&["config", "--make", "--list", "nonexistent"])
        .assert()
        .failure()
        .stderr(predicate::str::contains("not found"));
}

#[test]
fn test_dot_notation_nested_keys() {
    let temp_dir = TempDir::new().unwrap();
    let c2rust_dir = temp_dir.path().join(".c2rust");
    fs::create_dir(&c2rust_dir).unwrap();
    
    // Set nested value
    Command::cargo_bin("c2rust-config").unwrap()
        .current_dir(temp_dir.path())
        .args(&["config", "--model", "--set", "build.compiler.name", "gcc"])
        .assert().success();
    
    // Verify structure
    let config_path = c2rust_dir.join("config.toml");
    let content = fs::read_to_string(&config_path).unwrap();
    assert!(content.contains("[model.build.compiler]"));
    assert!(content.contains("name = \"gcc\""));
    
    // List the value
    Command::cargo_bin("c2rust-config").unwrap()
        .current_dir(temp_dir.path())
        .args(&["config", "--model", "--list", "build.compiler.name"])
        .assert()
        .success()
        .stdout(predicate::eq("gcc\n"));
}

#[test]
fn test_persistence() {
    let temp_dir = TempDir::new().unwrap();
    let c2rust_dir = temp_dir.path().join(".c2rust");
    fs::create_dir(&c2rust_dir).unwrap();
    
    // Set multiple values
    Command::cargo_bin("c2rust-config").unwrap()
        .current_dir(temp_dir.path())
        .args(&["config", "--model", "--set", "key1", "value1"])
        .assert().success();
    
    Command::cargo_bin("c2rust-config").unwrap()
        .current_dir(temp_dir.path())
        .args(&["config", "--make", "--set", "key2", "value2"])
        .assert().success();
    
    // Verify both persist
    Command::cargo_bin("c2rust-config").unwrap()
        .current_dir(temp_dir.path())
        .args(&["config", "--model", "--list", "key1"])
        .assert()
        .success()
        .stdout(predicate::eq("value1\n"));
    
    Command::cargo_bin("c2rust-config").unwrap()
        .current_dir(temp_dir.path())
        .args(&["config", "--make", "--list", "key2"])
        .assert()
        .success()
        .stdout(predicate::eq("value2\n"));
}

#[test]
fn test_del_on_nonexistent_key_is_noop() {
    let temp_dir = TempDir::new().unwrap();
    let c2rust_dir = temp_dir.path().join(".c2rust");
    fs::create_dir(&c2rust_dir).unwrap();
    
    // Del from non-existent key should not fail
    Command::cargo_bin("c2rust-config").unwrap()
        .current_dir(temp_dir.path())
        .args(&["config", "--model", "--del", "nonexistent", "value"])
        .assert()
        .success();
}
