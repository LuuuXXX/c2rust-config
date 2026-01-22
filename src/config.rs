use crate::error::{ConfigError, Result};
use std::fs;
use std::path::PathBuf;
use toml_edit::{DocumentMut, Item, Table};

pub struct Config {
    config_path: PathBuf,
    document: DocumentMut,
}

impl Config {
    /// Find .c2rust directory by traversing up from current directory
    fn find_c2rust_dir() -> Result<PathBuf> {
        let mut current = std::env::current_dir()?;
        loop {
            let c2rust_path = current.join(".c2rust");
            if c2rust_path.exists() && c2rust_path.is_dir() {
                return Ok(c2rust_path);
            }
            match current.parent() {
                Some(parent) => current = parent.to_path_buf(),
                None => return Err(ConfigError::ConfigDirNotFound),
            }
        }
    }

    /// Load configuration from file
    pub fn load() -> Result<Self> {
        let c2rust_dir = Self::find_c2rust_dir()?;
        let config_path = c2rust_dir.join("config.toml");

        let content = match fs::read_to_string(&config_path) {
            Ok(content) => content,
            Err(e) if e.kind() == std::io::ErrorKind::NotFound => {
                return Err(ConfigError::ConfigFileNotFound);
            }
            Err(e) => return Err(e.into()),
        };

        let document = content.parse::<DocumentMut>()?;
        Ok(Config {
            config_path,
            document,
        })
    }

    /// Initialize a new configuration file
    pub fn init() -> Result<()> {
        // Check if .c2rust directory exists in current directory
        let current_dir = std::env::current_dir()?;
        let c2rust_dir = current_dir.join(".c2rust");
        
        if !c2rust_dir.exists() {
            return Err(ConfigError::InvalidOperation(
                "Error: .c2rust directory not found in current path. Please create it first:\n  mkdir .c2rust".to_string()
            ));
        }

        let config_path = c2rust_dir.join("config.toml");
        
        // Check if config.toml already exists
        if config_path.exists() {
            return Err(ConfigError::InvalidOperation(
                "Configuration file already exists at .c2rust/config.toml".to_string()
            ));
        }

        // Create the template configuration with global, model, and feature sections
        let template = r#"# Global configuration
[global]
# Compiler settings (usually no need to configure)
# compiler = ["gcc"]

# Model-related configuration
[model]

# Feature-specific configuration
[feature.default]
# Relative to project root (.c2rust directory)
# clean.dir = "build"
# clean = "make clean"
# Relative to project root
# test.dir = "build"
# test = "make test"
# Relative to project root
# build.dir = "build"
# build = "make"
# Build options for extracting target files to translate
# Different files may have different compilation options
# One build can generate both debug/release binaries
# build.options = ["-I../3rd/include -DDEBUG=1", "-I../3rd/include"]
# files.x index corresponds to options index
# Each file list corresponds to one set of compilation options
# build.files.0 = ["main.c", "debug.c", "common.c"]
# build.files.1 = ["main.c", "release.c", "common.c"]
"#;

        fs::write(&config_path, template)?;
        Ok(())
    }

    /// Save configuration to file
    pub fn save(&self) -> Result<()> {
        fs::write(&self.config_path, self.document.to_string())?;
        Ok(())
    }

    /// Get the table for a specific section (model or feature.xxx)
    fn get_table_mut(&mut self, section: &str, create: bool) -> Result<&mut Table> {
        // Handle dotted keys by splitting them
        let parts: Vec<&str> = section.split('.').collect();
        
        if parts.is_empty() {
            return Err(ConfigError::InvalidOperation("Empty section name".to_string()));
        }
        
        // Navigate to the correct table
        let mut current_table = self.document.as_table_mut();
        
        for (i, &part) in parts.iter().enumerate() {
            let is_last = i == parts.len() - 1;
            
            if !current_table.contains_key(part) {
                if !create {
                    return Err(ConfigError::FeatureNotFound(section.to_string()));
                }
                
                // Create new table
                let mut new_table = toml_edit::Table::new();
                new_table.set_implicit(!is_last); // Last one should be explicit (has [header])
                current_table.insert(part, toml_edit::Item::Table(new_table));
            }
            
            current_table = current_table
                .get_mut(part)
                .and_then(|item| item.as_table_mut())
                .ok_or_else(|| ConfigError::TomlParseError(format!("'{}' is not a table", part)))?;
        }
        
        Ok(current_table)
    }

    /// List all values for a key
    pub fn list(&self, section: &str, key: &str) -> Result<Vec<String>> {
        // Handle dotted section names
        let section_parts: Vec<&str> = section.split('.').collect();
        
        let mut current_item = self.document.as_item();
        for &part in &section_parts {
            current_item = current_item
                .get(part)
                .ok_or_else(|| ConfigError::FeatureNotFound(section.to_string()))?;
        }
        
        let table = current_item
            .as_table()
            .ok_or_else(|| ConfigError::FeatureNotFound(section.to_string()))?;

        // Use dotted key directly
        let value = table
            .get(key)
            .ok_or_else(|| ConfigError::KeyNotFound(key.to_string()))?;

        let mut results = Vec::new();
        if let Some(array) = value.as_array() {
            for item in array.iter() {
                if let Some(s) = item.as_str() {
                    results.push(s.to_string());
                }
            }
        } else if let Some(s) = value.as_str() {
            results.push(s.to_string());
        }

        Ok(results)
    }

    /// Set a key to one or more values
    pub fn set(&mut self, section: &str, key: &str, values: Vec<String>) -> Result<()> {
        let key_parts: Vec<&str> = key.split('.').collect();

        if values.len() == 1 {
            let value = Item::Value(values[0].clone().into());
            self.set_value_in_section(section, &key_parts, value)?;
        } else {
            let array = toml_edit::Array::from_iter(values.iter().map(|v| v.as_str()));
            let value = Item::Value(array.into());
            self.set_value_in_section(section, &key_parts, value)?;
        }

        Ok(())
    }

    /// Helper to set a value in a section
    fn set_value_in_section(&mut self, section: &str, key_parts: &[&str], value: Item) -> Result<()> {
        let table = self.get_table_mut(section, true)?;
        Self::set_nested_static(table, key_parts, value)
    }

    /// Helper to set nested values (static method)
    /// Uses dotted keys (e.g., build.dir = "value") instead of nested tables
    fn set_nested_static(table: &mut Table, key_parts: &[&str], value: Item) -> Result<()> {
        if key_parts.is_empty() {
            return Err(ConfigError::InvalidOperation("Empty key".to_string()));
        }

        if key_parts.len() == 1 {
            table[key_parts[0]] = value;
        } else {
            // For multi-part keys, create a dotted key entry
            // The format will be: build.dir = "value" (or with quotes if needed)
            let dotted_key = key_parts.join(".");
            table[&dotted_key] = value;
        }
        Ok(())
    }

    /// Unset (remove) a key
    pub fn unset(&mut self, section: &str, key: &str) -> Result<()> {
        let table = self.get_table_mut(section, false)?;
        
        // For dotted keys, just remove using the full dotted key
        let dotted_key = key;
        table.remove(dotted_key);
        Ok(())
    }

    /// Add values to an array key
    pub fn add(&mut self, section: &str, key: &str, values: Vec<String>) -> Result<()> {
        // First check if key exists, if not create it
        {
            let table = self.get_table_mut(section, true)?;
            if !table.contains_key(key) {
                // Create new array with dotted key
                let empty_array = Item::Value(toml_edit::Array::new().into());
                table[key] = empty_array;
            }
        }

        // Now add values to the array
        {
            let table = self.get_table_mut(section, false)?;
            let item = table
                .get_mut(key)
                .ok_or_else(|| ConfigError::KeyNotFound(key.to_string()))?;

            let array = item.as_array_mut().ok_or_else(|| {
                ConfigError::InvalidOperation(format!("'{}' is not an array", key))
            })?;

            for value in values {
                array.push(value);
            }
        }

        Ok(())
    }

    /// Delete values from an array key
    pub fn del(&mut self, section: &str, key: &str, values: Vec<String>) -> Result<()> {
        let table = self.get_table_mut(section, false)?;

        let item = table
            .get_mut(key)
            .ok_or_else(|| ConfigError::KeyNotFound(key.to_string()))?;

        let array = item.as_array_mut().ok_or_else(|| {
            ConfigError::InvalidOperation(format!("'{}' is not an array", key))
        })?;

        // Use retain for O(n) complexity instead of repeated removals
        let values_to_remove: std::collections::HashSet<String> = values.into_iter().collect();
        array.retain(|item| {
            item.as_str()
                .map(|s| !values_to_remove.contains(s))
                .unwrap_or(true)
        });

        Ok(())
    }

    /// Validate that a feature has all required configuration keys
    /// Returns warnings if any required keys are missing
    pub fn validate_feature(&self, section: &str) -> Vec<String> {
        let mut warnings = Vec::new();
        
        // Only validate feature sections, not global or model
        if !section.starts_with("feature.") {
            return warnings;
        }

        // Get the feature table
        let section_parts: Vec<&str> = section.split('.').collect();
        let mut current_item = self.document.as_item();
        for &part in &section_parts {
            match current_item.get(part) {
                Some(item) => current_item = item,
                None => return warnings, // Section doesn't exist yet, no validation needed
            }
        }
        
        let table = match current_item.as_table() {
            Some(t) => t,
            None => return warnings,
        };

        // Required keys that must be configured together
        let required_keys = [
            "clean.dir",
            "clean",
            "test.dir",
            "test",
            "build.dir",
            "build",
        ];

        let mut missing_keys = Vec::new();
        for key in &required_keys {
            if !table.contains_key(*key) {
                missing_keys.push(*key);
            }
        }

        // If some but not all required keys are present, warn about missing ones
        if !missing_keys.is_empty() && missing_keys.len() < required_keys.len() {
            warnings.push(format!(
                "Warning: Feature '{}' is missing required keys: {}. All of [clean.dir, clean, test.dir, test, build.dir, build] should be configured together.",
                section,
                missing_keys.join(", ")
            ));
        }

        // Validate build.files.X count doesn't exceed build.options length
        if let Some(options_item) = table.get("build.options") {
            if let Some(options_array) = options_item.as_array() {
                let options_count = options_array.len();
                
                // Count build.files.X entries
                let mut max_files_index = -1i32;
                for (key, _) in table.iter() {
                    if key.starts_with("build.files.") {
                        if let Some(index_str) = key.strip_prefix("build.files.") {
                            if let Ok(index) = index_str.parse::<i32>() {
                                if index > max_files_index {
                                    max_files_index = index;
                                }
                            }
                        }
                    }
                }
                
                if max_files_index >= options_count as i32 {
                    warnings.push(format!(
                        "Warning: Feature '{}' has build.files.{} but only {} build.options entries. build.files.X indices should not exceed build.options array length.",
                        section,
                        max_files_index,
                        options_count
                    ));
                }
            }
        }

        warnings
    }
}
