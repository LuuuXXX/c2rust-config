use crate::config::Config;
use crate::error::Result;

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
            // Validate feature configuration after save
            let warnings = config.validate_feature(section);
            for warning in warnings {
                eprintln!("{}", warning);
            }
        }
        Operation::Unset => {
            config.unset(section, key)?;
            config.save()?;
            // Validate feature configuration after save
            let warnings = config.validate_feature(section);
            for warning in warnings {
                eprintln!("{}", warning);
            }
        }
        Operation::Add => {
            config.add(section, key, values)?;
            config.save()?;
            // Validate feature configuration after save
            let warnings = config.validate_feature(section);
            for warning in warnings {
                eprintln!("{}", warning);
            }
        }
        Operation::Del => {
            config.del(section, key, values)?;
            config.save()?;
            // Validate feature configuration after save
            let warnings = config.validate_feature(section);
            for warning in warnings {
                eprintln!("{}", warning);
            }
        }
        Operation::List => {
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
    Ok(())
}
