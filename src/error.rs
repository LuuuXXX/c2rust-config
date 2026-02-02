use std::fmt;
use std::path::PathBuf;

#[derive(Debug)]
pub enum ConfigError {
    ConfigDirNotFound(PathBuf),
    FeatureNotFound(String),
    KeyNotFound(String),
    IoError(std::io::Error),
    TomlParseError(String),
    InvalidOperation(String),
}

impl fmt::Display for ConfigError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            ConfigError::ConfigDirNotFound(path) => {
                // Multi-line error message for better readability in CLI output
                write!(f, "错误：未能找到 .c2rust 目录。\n搜索起始路径：{}\n已向上遍历至根目录但未找到项目根目录。\n请在项目根目录创建 .c2rust 目录，或从项目目录内运行此工具。", path.display())
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

impl From<toml::de::Error> for ConfigError {
    fn from(err: toml::de::Error) -> Self {
        ConfigError::TomlParseError(err.to_string())
    }
}

impl From<toml::ser::Error> for ConfigError {
    fn from(err: toml::ser::Error) -> Self {
        ConfigError::TomlParseError(err.to_string())
    }
}

pub type Result<T> = std::result::Result<T, ConfigError>;
