mod config;
mod error;
mod operations;

use clap::{Parser, Subcommand};
use config::Config;
use error::ConfigError;
use operations::Operation;

#[derive(Parser)]
#[command(name = "c2rust-config")]
#[command(about = "Configuration management tool for c2rust translation work")]
struct Cli {
    #[command(subcommand)]
    mode: Mode,
}

#[derive(Subcommand)]
enum Mode {
    /// Model-related configuration
    #[command(name = "model")]
    Model {
        #[command(subcommand)]
        operation: OperationCmd,
    },
    /// Build/clean/test-related configuration
    #[command(name = "make")]
    Make {
        /// Feature name (default: "default")
        #[arg(long)]
        feature: Option<String>,
        #[command(subcommand)]
        operation: OperationCmd,
    },
}

#[derive(Subcommand)]
enum OperationCmd {
    /// Set key-value(s)
    Set {
        /// Key to set
        key: String,
        /// Values to set
        values: Vec<String>,
    },
    /// Delete key-value
    Unset {
        /// Key to unset
        key: String,
    },
    /// Add value(s) to array key
    Add {
        /// Key to add values to
        key: String,
        /// Values to add
        values: Vec<String>,
    },
    /// Delete value(s) from array key
    Del {
        /// Key to delete values from
        key: String,
        /// Values to delete
        values: Vec<String>,
    },
    /// List all values for a key
    List {
        /// Key to list values for
        key: String,
    },
}

fn main() {
    if let Err(e) = run() {
        eprintln!("{}", e);
        std::process::exit(1);
    }
}

fn run() -> Result<(), ConfigError> {
    let cli = Cli::parse();

    let config = Config::load()?;

    match cli.mode {
        Mode::Model { operation } => {
            execute_operation(config, "model", operation)?;
        }
        Mode::Make { feature, operation } => {
            let feature_name = feature.unwrap_or_else(|| "default".to_string()).to_lowercase();
            let section = format!("feature.{}", feature_name);
            execute_operation(config, &section, operation)?;
        }
    }

    Ok(())
}

fn execute_operation(
    config: Config,
    section: &str,
    operation: OperationCmd,
) -> Result<(), ConfigError> {
    match operation {
        OperationCmd::Set { key, values } => {
            if values.is_empty() {
                return Err(ConfigError::InvalidOperation(
                    "No values provided for set operation".to_string(),
                ));
            }
            operations::execute(config, Operation::Set, section, &key, values)?;
        }
        OperationCmd::Unset { key } => {
            operations::execute(config, Operation::Unset, section, &key, vec![])?;
        }
        OperationCmd::Add { key, values } => {
            if values.is_empty() {
                return Err(ConfigError::InvalidOperation(
                    "No values provided for add operation".to_string(),
                ));
            }
            operations::execute(config, Operation::Add, section, &key, values)?;
        }
        OperationCmd::Del { key, values } => {
            if values.is_empty() {
                return Err(ConfigError::InvalidOperation(
                    "No values provided for del operation".to_string(),
                ));
            }
            operations::execute(config, Operation::Del, section, &key, values)?;
        }
        OperationCmd::List { key } => {
            operations::execute(config, Operation::List, section, &key, vec![])?;
        }
    }
    Ok(())
}