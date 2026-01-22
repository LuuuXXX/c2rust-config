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

    /// Save configuration to file
    pub fn save(&self) -> Result<()> {
        fs::write(&self.config_path, self.document.to_string())?;
        Ok(())
    }

    /// Get the table for a specific section (model or feature.xxx)
    fn get_table_mut(&mut self, section: &str, create: bool) -> Result<&mut Table> {
        if !self.document.contains_key(section) {
            if !create {
                return Err(ConfigError::FeatureNotFound(section.to_string()));
            }
            self.document[section] = toml_edit::table();
        }

        self.document[section]
            .as_table_mut()
            .ok_or_else(|| ConfigError::TomlParseError(format!("'{}' is not a table", section)))
    }

    /// Get nested value from table using dot-separated key
    fn get_nested<'a>(table: &'a Table, key_parts: &[&str]) -> Option<&'a Item> {
        if key_parts.is_empty() {
            return None;
        }

        let mut current = table.get(key_parts[0])?;
        for &part in &key_parts[1..] {
            current = current.get(part)?;
        }
        Some(current)
    }

    /// Get mutable nested value from table using dot-separated key
    fn get_nested_mut<'a>(
        table: &'a mut Table,
        key_parts: &[&str],
        create: bool,
    ) -> Option<&'a mut Item> {
        if key_parts.is_empty() {
            return None;
        }

        let mut current = if create {
            if !table.contains_key(key_parts[0]) {
                table[key_parts[0]] = Item::None;
            }
            table.get_mut(key_parts[0])?
        } else {
            table.get_mut(key_parts[0])?
        };

        for &part in &key_parts[1..] {
            if create && !current.as_table_like().is_some() {
                *current = toml_edit::table();
            }
            current = current.as_table_like_mut()?.get_mut(part)?;
        }
        Some(current)
    }

    /// List all values for a key
    pub fn list(&self, section: &str, key: &str) -> Result<Vec<String>> {
        let table = self
            .document
            .get(section)
            .and_then(|item| item.as_table())
            .ok_or_else(|| ConfigError::FeatureNotFound(section.to_string()))?;

        let key_parts: Vec<&str> = key.split('.').collect();
        let value = Self::get_nested(table, &key_parts)
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
    fn set_nested_static(table: &mut Table, key_parts: &[&str], value: Item) -> Result<()> {
        if key_parts.is_empty() {
            return Err(ConfigError::InvalidOperation("Empty key".to_string()));
        }

        if key_parts.len() == 1 {
            table[key_parts[0]] = value;
            return Ok(());
        }

        // Navigate to parent and set the value
        let parent_parts = &key_parts[..key_parts.len() - 1];
        let last_key = key_parts[key_parts.len() - 1];

        let mut current = table;
        for &part in parent_parts {
            if !current.contains_key(part) {
                current[part] = toml_edit::table();
            }
            current = current[part]
                .as_table_mut()
                .ok_or_else(|| ConfigError::InvalidOperation("Not a table".to_string()))?;
        }

        current[last_key] = value;
        Ok(())
    }

    /// Unset (remove) a key
    pub fn unset(&mut self, section: &str, key: &str) -> Result<()> {
        let table = self.get_table_mut(section, false)?;
        let key_parts: Vec<&str> = key.split('.').collect();

        if key_parts.len() == 1 {
            table.remove(key_parts[0]);
            return Ok(());
        }

        // Navigate to parent and remove the key
        let parent_parts = &key_parts[..key_parts.len() - 1];
        let last_key = key_parts[key_parts.len() - 1];

        let mut current = table;
        for &part in parent_parts {
            current = current
                .get_mut(part)
                .and_then(|item| item.as_table_mut())
                .ok_or_else(|| ConfigError::KeyNotFound(key.to_string()))?;
        }

        current.remove(last_key);
        Ok(())
    }

    /// Add values to an array key
    pub fn add(&mut self, section: &str, key: &str, values: Vec<String>) -> Result<()> {
        let key_parts: Vec<&str> = key.split('.').collect();

        // First check if key exists, if not create it
        {
            let table = self.get_table_mut(section, true)?;
            if Self::get_nested(table, &key_parts).is_none() {
                // Create new nested structure with empty array
                let empty_array = Item::Value(toml_edit::Array::new().into());
                Self::set_nested_static(table, &key_parts, empty_array)?;
            }
        }

        // Now add values to the array
        {
            let table = self.get_table_mut(section, false)?;
            let item = Self::get_nested_mut(table, &key_parts, false)
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
        let key_parts: Vec<&str> = key.split('.').collect();

        let item = Self::get_nested_mut(table, &key_parts, false)
            .ok_or_else(|| ConfigError::KeyNotFound(key.to_string()))?;

        let array = item.as_array_mut().ok_or_else(|| {
            ConfigError::InvalidOperation(format!("'{}' is not an array", key))
        })?;

        // Remove matching values
        for value_to_remove in &values {
            let mut i = 0;
            while i < array.len() {
                if let Some(s) = array.get(i).and_then(|v| v.as_str()) {
                    if s == value_to_remove {
                        array.remove(i);
                        continue;
                    }
                }
                i += 1;
            }
        }

        Ok(())
    }
}

/// Create a new empty config file
pub fn create_empty_config() -> Result<()> {
    let c2rust_dir = Config::find_c2rust_dir()?;
    let config_path = c2rust_dir.join("config.toml");

    if config_path.exists() {
        return Err(ConfigError::InvalidOperation(
            "Config file already exists".to_string(),
        ));
    }

    // Create a minimal config with [model] section
    let content = "[model]\n";
    fs::write(&config_path, content)?;
    Ok(())
}
