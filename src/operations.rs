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
            config.save()?;
        }
        Operation::Unset => {
            config.unset(section, key)?;
            config.save()?;
        }
        Operation::Add => {
            config.add(section, key, values)?;
            config.save()?;
        }
        Operation::Del => {
            config.del(section, key, values)?;
            config.save()?;
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
