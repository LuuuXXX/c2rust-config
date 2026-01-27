mod config;
mod error;
mod operations;

use clap::{Args, Parser, Subcommand};
use config::Config;
use error::ConfigError;
use operations::Operation;

#[derive(Parser)]
#[command(name = "c2rust-config")]
#[command(about = "Configuration management tool for c2rust translation work")]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Configuration management
    Config(ConfigArgs),
}

#[derive(Args)]
struct ConfigArgs {
    /// Global configuration (e.g., compiler settings)
    #[arg(long, group = "mode")]
    global: bool,

    /// Model-related configuration
    #[arg(long, group = "mode")]
    model: bool,

    /// Build/clean/test-related configuration
    #[arg(long, group = "mode")]
    make: bool,

    /// Feature name (default: "default") - only for --make
    #[arg(long, requires = "make")]
    feature: Option<String>,

    /// Set key-value(s)
    #[arg(long, group = "operation")]
    set: bool,

    /// Delete key-value
    #[arg(long, group = "operation")]
    unset: bool,

    /// Add value(s) to array key
    #[arg(long, group = "operation")]
    add: bool,

    /// Delete value(s) from array key
    #[arg(long, group = "operation")]
    del: bool,

    /// List all values in the section, or specific key if provided
    #[arg(long, group = "operation")]
    list: bool,

    /// Key to operate on
    key: Option<String>,

    /// Values to set, add, or delete
    #[arg(allow_hyphen_values = true)]
    values: Vec<String>,
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

    match cli.command {
        Commands::Config(args) => {
            // Validate exactly one mode is selected
            let mode_count = [args.global, args.model, args.make].iter().filter(|&&x| x).count();
            if mode_count != 1 {
                return Err(ConfigError::InvalidOperation(
                    "Exactly one of --global, --model, or --make must be specified".to_string(),
                ));
            }

            // Validate exactly one operation is selected
            let op_count = [args.set, args.unset, args.add, args.del, args.list].iter().filter(|&&x| x).count();
            if op_count != 1 {
                return Err(ConfigError::InvalidOperation(
                    "Exactly one of --set, --unset, --add, --del, or --list must be specified".to_string(),
                ));
            }

            // Validate --feature is only used with --make
            if args.feature.is_some() && !args.make {
                return Err(ConfigError::InvalidOperation(
                    "--feature can only be used with --make".to_string(),
                ));
            }

            // Determine the section
            let section = if args.global {
                "global".to_string()
            } else if args.model {
                "model".to_string()
            } else {
                let feature_name = args.feature.unwrap_or_else(|| "default".to_string()).to_lowercase();
                format!("feature.{}", feature_name)
            };

            // Determine and execute the operation
            let operation = if args.set {
                Operation::Set
            } else if args.unset {
                Operation::Unset
            } else if args.add {
                Operation::Add
            } else if args.del {
                Operation::Del
            } else {
                Operation::List
            };

            // Validate operation-specific requirements
            let key = match operation {
                Operation::List => args.key.unwrap_or_default(),
                _ => args.key.ok_or_else(|| {
                    ConfigError::InvalidOperation(format!("--{:?} requires a key", operation).to_lowercase())
                })?,
            };

            if matches!(operation, Operation::Set | Operation::Add | Operation::Del) && args.values.is_empty() {
                return Err(ConfigError::InvalidOperation(
                    format!("--{:?} requires at least one value", operation).to_lowercase(),
                ));
            }

            operations::execute(config, operation, &section, &key, args.values)?;
        }
    }

    Ok(())
}