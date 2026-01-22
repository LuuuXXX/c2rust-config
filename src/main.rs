mod config;

use anyhow::Result;
use clap::{Args, Parser, Subcommand};
use config::ConfigManager;

#[derive(Parser)]
#[command(name = "c2rust-config")]
#[command(about = "C2Rust configuration tool", long_about = None)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Manage configuration
    Config(ConfigArgs),
}

#[derive(Args)]
struct ConfigArgs {
    #[command(flatten)]
    scope: ConfigScope,

    /// Feature name for --make operations (default: "default")
    #[arg(long, requires = "make")]
    feature: Option<String>,

    #[command(flatten)]
    operation: ConfigOperation,

    /// Configuration key (supports dot notation like build.dir)
    key: String,

    /// Values for the operation
    values: Vec<String>,
}

#[derive(Args)]
#[group(required = true, multiple = false)]
struct ConfigScope {
    /// Operate on model configuration
    #[arg(long)]
    model: bool,

    /// Operate on make/feature configuration
    #[arg(long)]
    make: bool,
}

#[derive(Args)]
#[group(required = true, multiple = false)]
struct ConfigOperation {
    /// Set key to value(s) - scalar for single value, array for multiple
    #[arg(long)]
    set: bool,

    /// Remove key
    #[arg(long)]
    unset: bool,

    /// Append value(s) to array
    #[arg(long)]
    add: bool,

    /// Remove value(s) from array
    #[arg(long)]
    del: bool,

    /// List values of key
    #[arg(long)]
    list: bool,
}

fn main() -> Result<()> {
    let cli = Cli::parse();

    match cli.command {
        Commands::Config(args) => handle_config(args)?,
    }

    Ok(())
}

fn handle_config(args: ConfigArgs) -> Result<()> {
    let mut config_manager = ConfigManager::new()?;

    // Determine scope
    let scope = if args.scope.model {
        "model"
    } else {
        "make"
    };

    // Get feature name (default to "default", normalize to lowercase)
    let feature = args
        .feature
        .as_deref()
        .unwrap_or("default")
        .to_lowercase();

    // Perform operation
    if args.operation.set {
        config_manager.set(scope, &feature, &args.key, &args.values)?;
    } else if args.operation.unset {
        config_manager.unset(scope, &feature, &args.key)?;
    } else if args.operation.add {
        config_manager.add(scope, &feature, &args.key, &args.values)?;
    } else if args.operation.del {
        config_manager.del(scope, &feature, &args.key, &args.values)?;
    } else if args.operation.list {
        let values = config_manager.list(scope, &feature, &args.key)?;
        for value in values {
            println!("{}", value);
        }
    }

    Ok(())
}