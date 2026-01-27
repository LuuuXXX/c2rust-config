use crate::error::{ConfigError, Result};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs;
use std::path::PathBuf;

#[derive(Debug, Serialize, Deserialize, Default)]
pub struct ConfigFile {
    #[serde(default)]
    pub global: HashMap<String, toml::Value>,
    #[serde(default)]
    pub model: HashMap<String, toml::Value>,
    #[serde(default, rename = "feature")]
    pub features: HashMap<String, HashMap<String, toml::Value>>,
}

pub struct Config {
    config_path: PathBuf,
    data: ConfigFile,
}

impl Config {
    /// Find .c2rust directory in current directory
    fn find_c2rust_dir() -> Result<PathBuf> {
        let current = std::env::current_dir()?;
        let c2rust_path = current.join(".c2rust");
        if c2rust_path.exists() && c2rust_path.is_dir() {
            Ok(c2rust_path)
        } else {
            Err(ConfigError::ConfigDirNotFound)
        }
    }

    /// Flatten nested table structures into dotted keys (recursively)
    /// Converts structures like:
    ///   clean = Table({"cmd": "make clean", "dir": "build"})
    /// Into:
    ///   "clean.cmd" = "make clean", "clean.dir" = "build"
    /// Also handles deeply nested structures:
    ///   build = Table({"options": Table({"debug": true})})
    /// Into:
    ///   "build.options.debug" = true
    fn flatten_table(table: &mut HashMap<String, toml::Value>) {
        fn flatten_value(prefix: &str, value: &toml::Value, result: &mut HashMap<String, toml::Value>) {
            if let Some(nested_table) = value.as_table() {
                // Recursively flatten nested tables
                for (nested_key, nested_value) in nested_table {
                    let new_prefix = if prefix.is_empty() {
                        nested_key.clone()
                    } else {
                        format!("{}.{}", prefix, nested_key)
                    };
                    flatten_value(&new_prefix, nested_value, result);
                }
            } else {
                // Leaf value - add it to the result
                result.insert(prefix.to_string(), value.clone());
            }
        }

        let mut flattened = HashMap::new();
        let mut keys_to_remove = Vec::new();

        for (key, value) in table.iter() {
            if value.as_table().is_some() {
                // This is a nested table, flatten it recursively
                keys_to_remove.push(key.clone());
                flatten_value(key, value, &mut flattened);
            }
        }

        // Remove the nested table keys
        for key in keys_to_remove {
            table.remove(&key);
        }

        // Add the flattened keys
        table.extend(flattened);
    }

    /// Load configuration from file
    /// Auto-creates config.toml if it doesn't exist
    pub fn load() -> Result<Self> {
        let c2rust_dir = Self::find_c2rust_dir()?;
        let config_path = c2rust_dir.join("config.toml");

        let content = match fs::read_to_string(&config_path) {
            Ok(content) => content,
            Err(e) if e.kind() == std::io::ErrorKind::NotFound => {
                // Auto-create config.toml with default sections including feature.default
                let default_content = "[global]\n\n[model]\n\n[feature.default]\n";
                fs::write(&config_path, default_content)?;
                default_content.to_owned()
            }
            Err(e) => return Err(e.into()),
        };

        let mut data: ConfigFile = toml::from_str(&content)?;
        
        // Flatten nested structures in all sections
        Self::flatten_table(&mut data.global);
        Self::flatten_table(&mut data.model);
        for (_, feature_table) in data.features.iter_mut() {
            Self::flatten_table(feature_table);
        }
        
        Ok(Config { config_path, data })
    }

    /// Save configuration to file
    pub fn save(&self) -> Result<()> {
        let content = toml::to_string_pretty(&self.data)?;
        fs::write(&self.config_path, content)?;
        Ok(())
    }

    /// Get the table for a specific section
    fn get_table_mut(&mut self, section: &str, create: bool) -> Result<&mut HashMap<String, toml::Value>> {
        if section == "global" {
            return Ok(&mut self.data.global);
        } else if section == "model" {
            return Ok(&mut self.data.model);
        } else if let Some(feature_name) = section.strip_prefix("feature.") {
            if !self.data.features.contains_key(feature_name) {
                if !create {
                    return Err(ConfigError::FeatureNotFound(section.to_string()));
                }
                self.data.features.insert(feature_name.to_string(), HashMap::new());
            }
            return Ok(self.data.features.get_mut(feature_name).unwrap());
        }
        Err(ConfigError::InvalidOperation(format!("Invalid section: {}", section)))
    }

    /// Get the table for reading
    fn get_table(&self, section: &str) -> Result<&HashMap<String, toml::Value>> {
        if section == "global" {
            return Ok(&self.data.global);
        } else if section == "model" {
            return Ok(&self.data.model);
        } else if let Some(feature_name) = section.strip_prefix("feature.") {
            return self.data.features
                .get(feature_name)
                .ok_or_else(|| ConfigError::FeatureNotFound(section.to_string()));
        }
        Err(ConfigError::InvalidOperation(format!("Invalid section: {}", section)))
    }

    /// List all keys and values in a section
    pub fn list_all(&self, section: &str) -> Result<Vec<(String, Vec<String>)>> {
        let table = self.get_table(section)?;
        
        let mut results = Vec::new();
        for (key, value) in table.iter() {
            let mut values = Vec::new();
            if let Some(array) = value.as_array() {
                for item in array.iter() {
                    if let Some(s) = item.as_str() {
                        values.push(s.to_string());
                    } else {
                        // Convert non-string array elements to string representation
                        values.push(item.to_string());
                    }
                }
            } else if let Some(s) = value.as_str() {
                values.push(s.to_string());
            } else {
                // Handle other scalar value types (integer, float, boolean, datetime)
                // by converting them to their string representation. Tables are
                // flattened in Config::load(), so they should not appear here.
                values.push(value.to_string());
            }
            if !values.is_empty() {
                results.push((key.clone(), values));
            }
        }

        Ok(results)
    }

    /// Get values for a specific key in a section
    pub fn list(&self, section: &str, key: &str) -> Result<Vec<String>> {
        let table = self.get_table(section)?;
        
        let value = table.get(key)
            .ok_or_else(|| ConfigError::KeyNotFound(key.to_string()))?;
        
        let mut values = Vec::new();
        if let Some(array) = value.as_array() {
            for item in array.iter() {
                if let Some(s) = item.as_str() {
                    values.push(s.to_string());
                } else {
                    // Convert non-string array elements to string representation
                    values.push(item.to_string());
                }
            }
        } else if let Some(s) = value.as_str() {
            values.push(s.to_string());
        } else {
            // Handle other value types (integer, float, boolean, datetime, table)
            // by converting them to their string representation
            values.push(value.to_string());
        }
        
        Ok(values)
    }

    /// Set a key to one or more values
    pub fn set(&mut self, section: &str, key: &str, values: Vec<String>) -> Result<()> {
        let table = self.get_table_mut(section, true)?;

        let value = if values.len() == 1 {
            toml::Value::String(values[0].clone())
        } else {
            let array: Vec<toml::Value> = values.into_iter()
                .map(toml::Value::String)
                .collect();
            toml::Value::Array(array)
        };

        table.insert(key.to_string(), value);
        Ok(())
    }

    /// Unset (remove) a key
    pub fn unset(&mut self, section: &str, key: &str) -> Result<()> {
        let table = self.get_table_mut(section, false)?;
        table.remove(key);
        Ok(())
    }

    /// Add values to an array key
    pub fn add(&mut self, section: &str, key: &str, values: Vec<String>) -> Result<()> {
        let table = self.get_table_mut(section, true)?;

        let current = table.entry(key.to_string()).or_insert_with(|| toml::Value::Array(vec![]));
        
        let array = current.as_array_mut()
            .ok_or_else(|| ConfigError::InvalidOperation(format!("'{}' is not an array", key)))?;

        for value in values {
            array.push(toml::Value::String(value));
        }

        Ok(())
    }

    /// Delete values from an array key
    pub fn del(&mut self, section: &str, key: &str, values: Vec<String>) -> Result<()> {
        let table = self.get_table_mut(section, false)?;

        let current = table.get_mut(key)
            .ok_or_else(|| ConfigError::KeyNotFound(key.to_string()))?;

        let array = current.as_array_mut()
            .ok_or_else(|| ConfigError::InvalidOperation(format!("'{}' is not an array", key)))?;

        // Use HashSet for O(n+m) performance instead of O(n*m)
        let values_set: std::collections::HashSet<_> = values.iter().map(|s| s.as_str()).collect();
        array.retain(|v| {
            v.as_str()
                .map(|s| !values_set.contains(s))
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
        let table = match self.get_table(section) {
            Ok(t) => t,
            Err(_) => return warnings, // Section doesn't exist yet, no validation needed
        };

        // Required keys that must be configured together
        let required_keys = [
            "clean.dir",
            "clean.cmd",
            "test.dir",
            "test.cmd",
            "build.dir",
            "build.cmd",
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
                "Warning: Feature '{}' is missing required keys: {}. All of [{}] should be configured together.",
                section,
                missing_keys.join(", "),
                required_keys.join(", ")
            ));
        }

        warnings
    }
}
