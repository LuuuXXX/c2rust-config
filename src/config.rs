use anyhow::{anyhow, bail, Context, Result};
use std::fs;
use std::path::{Path, PathBuf};
use toml_edit::{value, Array, DocumentMut, Item, Table, Value};

const CONFIG_DIR: &str = ".c2rust";
const CONFIG_FILE: &str = "config.toml";

pub struct ConfigManager {
    config_path: PathBuf,
}

impl ConfigManager {
    /// Create a new ConfigManager
    /// Checks that .c2rust directory exists, but doesn't auto-create
    pub fn new() -> Result<Self> {
        let config_dir = Path::new(CONFIG_DIR);
        
        if !config_dir.exists() {
            bail!(
                "Directory '{}' does not exist. Please create/initialize it first.",
                CONFIG_DIR
            );
        }

        let config_path = config_dir.join(CONFIG_FILE);
        
        Ok(ConfigManager { config_path })
    }

    /// Load configuration from file
    fn load_config(&self) -> Result<DocumentMut> {
        if !self.config_path.exists() {
            bail!("Configuration file not found: {}", self.config_path.display());
        }

        let content = fs::read_to_string(&self.config_path)
            .with_context(|| format!("Failed to read config file: {}", self.config_path.display()))?;

        content
            .parse::<DocumentMut>()
            .with_context(|| format!("Failed to parse config file: {}", self.config_path.display()))
    }

    /// Save configuration to file
    fn save_config(&self, doc: &DocumentMut) -> Result<()> {
        fs::write(&self.config_path, doc.to_string())
            .with_context(|| format!("Failed to write config file: {}", self.config_path.display()))
    }

    /// Set a key to value(s)
    pub fn set(&mut self, scope: &str, feature: &str, key: &str, values: &[String]) -> Result<()> {
        let mut doc = self.load_or_create_config()?;
        
        let table = self.get_or_create_table(&mut doc, scope, feature)?;
        
        // Navigate to the key path
        let (parent_table, final_key) = self.navigate_to_parent(table, key)?;
        
        // Set value: single value = scalar, multiple values = array
        let item = if values.len() == 1 {
            value(&values[0])
        } else {
            let mut arr = Array::new();
            for v in values {
                arr.push(v);
            }
            Item::Value(Value::Array(arr))
        };
        
        parent_table[final_key] = item;
        
        self.save_config(&doc)
    }

    /// Unset (remove) a key
    pub fn unset(&mut self, scope: &str, feature: &str, key: &str) -> Result<()> {
        let mut doc = self.load_or_create_config()?;
        
        let table = self.get_or_create_table(&mut doc, scope, feature)?;
        
        let (parent_table, final_key) = self.navigate_to_parent(table, key)?;
        
        parent_table.remove(final_key);
        
        self.save_config(&doc)
    }

    /// Add value(s) to an array
    pub fn add(&mut self, scope: &str, feature: &str, key: &str, values: &[String]) -> Result<()> {
        let mut doc = self.load_or_create_config()?;
        
        let table = self.get_or_create_table(&mut doc, scope, feature)?;
        
        let (parent_table, final_key) = self.navigate_to_parent(table, key)?;
        
        // Get or create array
        let item = parent_table.entry(final_key).or_insert(Item::Value(Value::Array(Array::new())));
        
        // If the existing item is not an array, we need to convert it or create a new array
        if !item.is_array() {
            // For now, convert scalar to array with the existing value
            if let Some(existing_str) = item.as_str() {
                let mut arr = Array::new();
                arr.push(existing_str);
                *item = Item::Value(Value::Array(arr));
            } else {
                // Replace with empty array
                *item = Item::Value(Value::Array(Array::new()));
            }
        }
        
        let arr = item
            .as_array_mut()
            .ok_or_else(|| anyhow!("Key '{}' is not an array", key))?;
        
        for v in values {
            arr.push(v);
        }
        
        self.save_config(&doc)
    }

    /// Delete value(s) from an array
    pub fn del(&mut self, scope: &str, feature: &str, key: &str, values: &[String]) -> Result<()> {
        let mut doc = self.load_or_create_config()?;
        
        let table = match self.get_table(&mut doc, scope, feature) {
            Some(t) => t,
            None => return Ok(()), // No-op if table doesn't exist
        };
        
        let (parent_table, final_key) = match self.try_navigate_to_parent(table, key) {
            Some(result) => result,
            None => return Ok(()), // No-op if key doesn't exist
        };
        
        if let Some(item) = parent_table.get_mut(final_key) {
            if let Some(arr) = item.as_array_mut() {
                // Remove matching values
                arr.retain(|v| {
                    if let Some(s) = v.as_str() {
                        !values.contains(&s.to_string())
                    } else {
                        true
                    }
                });
            }
        }
        
        self.save_config(&doc)
    }

    /// List value(s) of a key
    pub fn list(&self, scope: &str, feature: &str, key: &str) -> Result<Vec<String>> {
        let mut doc = self.load_config()?;
        
        let table = self.get_table(&mut doc, scope, feature)
            .ok_or_else(|| {
                if scope == "make" {
                    anyhow!("Feature '{}' not found", feature)
                } else {
                    anyhow!("Section '{}' not found", scope)
                }
            })?;
        
        let item = self.navigate_to_key(table, key)
            .ok_or_else(|| anyhow!("Key '{}' not found", key))?;
        
        // Convert to string vector
        if let Some(arr) = item.as_array() {
            Ok(arr
                .iter()
                .filter_map(|v| v.as_str().map(String::from))
                .collect())
        } else if let Some(s) = item.as_str() {
            Ok(vec![s.to_string()])
        } else if let Some(i) = item.as_integer() {
            Ok(vec![i.to_string()])
        } else if let Some(b) = item.as_bool() {
            Ok(vec![b.to_string()])
        } else {
            Ok(vec![item.to_string()])
        }
    }

    /// Load config or create minimal structure
    fn load_or_create_config(&self) -> Result<DocumentMut> {
        if self.config_path.exists() {
            self.load_config()
        } else {
            // Create minimal config
            let doc = DocumentMut::new();
            fs::create_dir_all(self.config_path.parent().unwrap())?;
            self.save_config(&doc)?;
            Ok(doc)
        }
    }

    /// Get or create a table for the given scope and feature
    fn get_or_create_table<'a>(
        &self,
        doc: &'a mut DocumentMut,
        scope: &str,
        feature: &str,
    ) -> Result<&'a mut Table> {
        if scope == "model" {
            Ok(doc.entry("model")
                .or_insert(Item::Table(Table::new()))
                .as_table_mut()
                .unwrap())
        } else {
            // scope == "make"
            // Get or create the "feature" table first
            let feature_table = doc.entry("feature")
                .or_insert(Item::Table(Table::new()))
                .as_table_mut()
                .unwrap();
            
            // Then get or create the specific feature table
            Ok(feature_table.entry(feature)
                .or_insert(Item::Table(Table::new()))
                .as_table_mut()
                .unwrap())
        }
    }

    /// Get a table for the given scope and feature (without creating)
    fn get_table<'a>(
        &self,
        doc: &'a mut DocumentMut,
        scope: &str,
        feature: &str,
    ) -> Option<&'a mut Table> {
        if scope == "model" {
            doc.get_mut("model")?.as_table_mut()
        } else {
            // scope == "make"
            doc.get_mut("feature")?.as_table_mut()?.get_mut(feature)?.as_table_mut()
        }
    }

    /// Navigate to parent table and return final key
    fn navigate_to_parent<'a>(
        &self,
        table: &'a mut Table,
        key: &'a str,
    ) -> Result<(&'a mut Table, &'a str)> {
        let parts: Vec<&str> = key.split('.').collect();
        
        if parts.is_empty() {
            bail!("Empty key path");
        }
        
        if parts.len() == 1 {
            return Ok((table, key));
        }
        
        let mut current = table;
        for part in &parts[..parts.len() - 1] {
            current = current
                .entry(part)
                .or_insert(Item::Table(Table::new()))
                .as_table_mut()
                .ok_or_else(|| anyhow!("Path component '{}' is not a table", part))?;
        }
        
        Ok((current, parts.last().unwrap()))
    }

    /// Try to navigate to parent table (returns None if path doesn't exist)
    fn try_navigate_to_parent<'a>(
        &self,
        table: &'a mut Table,
        key: &'a str,
    ) -> Option<(&'a mut Table, &'a str)> {
        let parts: Vec<&str> = key.split('.').collect();
        
        if parts.is_empty() {
            return None;
        }
        
        if parts.len() == 1 {
            return Some((table, key));
        }
        
        let mut current = table;
        for part in &parts[..parts.len() - 1] {
            current = current.get_mut(part)?.as_table_mut()?;
        }
        
        Some((current, parts.last().unwrap()))
    }

    /// Navigate to a key and return the item
    fn navigate_to_key<'a>(&self, table: &'a Table, key: &str) -> Option<&'a Item> {
        let parts: Vec<&str> = key.split('.').collect();
        
        if parts.is_empty() {
            return None;
        }
        
        if parts.len() == 1 {
            return table.get(key);
        }
        
        let mut current_table = table;
        for part in &parts[..parts.len() - 1] {
            current_table = current_table.get(part)?.as_table()?;
        }
        
        current_table.get(parts.last().unwrap())
    }
}
