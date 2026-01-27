use crate::config::Config;
use crate::error::Result;

#[derive(Debug)]
pub enum Operation {
    Set,
    Unset,
    Add,
    Del,
    List,
}

/// Helper function to save config
fn save_config(config: &mut Config) -> Result<()> {
    config.save()?;
    Ok(())
}

pub fn execute(
    mut config: Config,
    operation: Operation,
    section: &str,
    key: &str,
    values: Vec<String>,
) -> Result<()> {
    match operation {
        Operation::Set => {
            config.set(section, key, values)?;
            save_config(&mut config)?;
        }
        Operation::Unset => {
            config.unset(section, key)?;
            save_config(&mut config)?;
        }
        Operation::Add => {
            config.add(section, key, values)?;
            save_config(&mut config)?;
        }
        Operation::Del => {
            config.del(section, key, values)?;
            save_config(&mut config)?;
        }
        Operation::List => {
            // If a key is provided, only output that key's values
            if !key.is_empty() {
                let value = config.list(section, key)?;
                for v in value {
                    println!("{}", v);
                }
            } else {
                // Otherwise, list all configurations
                let results = config.list_all(section)?;
                for (key, values) in results {
                    if values.len() == 1 {
                        println!("{} = {}", key, values[0]);
                    } else {
                        println!("{} = [", key);
                        for value in values {
                            println!("  {}", value);
                        }
                        println!("]");
                    }
                }
            }
        }
    }
    Ok(())
}
