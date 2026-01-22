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

    /// List all values in the section (array elements displayed on separate lines)
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
            // Validate that exactly one mode is selected
            let mode_count = [args.global, args.model, args.make].iter().filter(|&&x| x).count();
            if mode_count != 1 {
                return Err(ConfigError::InvalidOperation(
                    "Exactly one of --global, --model, or --make must be specified".to_string(),
                ));
            }

            // Validate that exactly one operation is selected
            let op_count = [args.set, args.unset, args.add, args.del, args.list].iter().filter(|&&x| x).count();
            if op_count != 1 {
                return Err(ConfigError::InvalidOperation(
                    "Exactly one of --set, --unset, --add, --del, or --list must be specified".to_string(),
                ));
            }

            // Determine the section based on mode flags
            let section = if args.global {
                "global".to_string()
            } else if args.model {
                "model".to_string()
            } else if args.make {
                let feature_name = args.feature.unwrap_or_else(|| "default".to_string()).to_lowercase();
                format!("feature.{}", feature_name)
            } else {
                unreachable!("Mode validation ensures one of global/model/make is set");
            };

            // Execute the operation based on operation flags
            if args.set {
                let key = args.key.ok_or_else(|| {
                    ConfigError::InvalidOperation("--set requires a key".to_string())
                })?;
                if args.values.is_empty() {
                    return Err(ConfigError::InvalidOperation(
                        "No values provided for set operation".to_string(),
                    ));
                }
                operations::execute(config, Operation::Set, &section, &key, args.values)?;
            } else if args.unset {
                let key = args.key.ok_or_else(|| {
                    ConfigError::InvalidOperation("--unset requires a key".to_string())
                })?;
                operations::execute(config, Operation::Unset, &section, &key, vec![])?;
            } else if args.add {
                let key = args.key.ok_or_else(|| {
                    ConfigError::InvalidOperation("--add requires a key".to_string())
                })?;
                if args.values.is_empty() {
                    return Err(ConfigError::InvalidOperation(
                        "No values provided for add operation".to_string(),
                    ));
                }
                operations::execute(config, Operation::Add, &section, &key, args.values)?;
            } else if args.del {
                let key = args.key.ok_or_else(|| {
                    ConfigError::InvalidOperation("--del requires a key".to_string())
                })?;
                if args.values.is_empty() {
                    return Err(ConfigError::InvalidOperation(
                        "No values provided for del operation".to_string(),
                    ));
                }
                operations::execute(config, Operation::Del, &section, &key, args.values)?;
            } else if args.list {
                operations::execute(config, Operation::List, &section, "", vec![])?;
            } else {
                unreachable!("Operation validation ensures one operation is set");
            }
        }
    }

    Ok(())
}