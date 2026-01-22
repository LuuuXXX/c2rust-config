use std::fmt;

#[derive(Debug)]
pub enum ConfigError {
    ConfigDirNotFound,
    ConfigFileNotFound,
    FeatureNotFound(String),
    KeyNotFound(String),
    IoError(std::io::Error),
    TomlParseError(String),
    InvalidOperation(String),
}

impl fmt::Display for ConfigError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            ConfigError::ConfigDirNotFound => {
                write!(f, "Error: .c2rust directory not found. Please create it first.")
            }
            ConfigError::ConfigFileNotFound => {
                write!(f, "Error: config.toml file not found in .c2rust directory")
            }
            ConfigError::FeatureNotFound(feature) => {
                write!(f, "Error: feature '{}' not found in configuration", feature)
            }
            ConfigError::KeyNotFound(key) => {
                write!(f, "Error: key '{}' not found", key)
            }
            ConfigError::IoError(e) => write!(f, "IO error: {}", e),
            ConfigError::TomlParseError(e) => write!(f, "TOML parse error: {}", e),
            ConfigError::InvalidOperation(msg) => write!(f, "Invalid operation: {}", msg),
        }
    }
}

impl std::error::Error for ConfigError {}

impl From<std::io::Error> for ConfigError {
    fn from(err: std::io::Error) -> Self {
        ConfigError::IoError(err)
    }
}

impl From<toml_edit::TomlError> for ConfigError {
    fn from(err: toml_edit::TomlError) -> Self {
        ConfigError::TomlParseError(err.to_string())
    }
}

pub type Result<T> = std::result::Result<T, ConfigError>;
