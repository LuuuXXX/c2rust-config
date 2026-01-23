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

        let data: ConfigFile = toml::from_str(&content)?;
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
                    }
                }
            } else if let Some(s) = value.as_str() {
                values.push(s.to_string());
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
                }
            }
        } else if let Some(s) = value.as_str() {
            values.push(s.to_string());
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
        if let Some(options_value) = table.get("build.options") {
            if let Some(options_array) = options_value.as_array() {
                let options_count = options_array.len();
                
                // Track maximum build.files.X index
                let mut max_files_index: Option<usize> = None;
                for key in table.keys() {
                    if key.starts_with("build.files.") {
                        if let Some(index_str) = key.strip_prefix("build.files.") {
                            if let Ok(index) = index_str.parse::<usize>() {
                                max_files_index =
                                    Some(max_files_index.map_or(index, |m| m.max(index)));
                            }
                        }
                    }
                }
                
                if let Some(idx) = max_files_index {
                    if idx >= options_count {
                        warnings.push(format!(
                            "Warning: Feature '{}' has build.files.{} but only {} build.options entries. build.files.X indices should not exceed build.options array length.",
                            section,
                            idx,
                            options_count
                        ));
                    }
                }
            }
        }

        warnings
    }
}
