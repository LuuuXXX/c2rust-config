use crate::config::Config;
use crate::error::Result;

pub enum Operation {
    Set,
    Unset,
    Add,
    Del,
    List,
}

/// Helper function to save config and validate feature configuration
fn save_and_validate(config: &mut Config, section: &str) -> Result<()> {
    config.save()?;
    let warnings = config.validate_feature(section);
    for warning in warnings {
        eprintln!("{}", warning);
    }
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
            save_and_validate(&mut config, section)?;
        }
        Operation::Unset => {
            config.unset(section, key)?;
            save_and_validate(&mut config, section)?;
        }
        Operation::Add => {
            config.add(section, key, values)?;
            save_and_validate(&mut config, section)?;
        }
        Operation::Del => {
            config.del(section, key, values)?;
            save_and_validate(&mut config, section)?;
        }
        Operation::List => {
            // 如果提供了 key，只输出该 key 的值
            if !key.is_empty() {
                let value = config.list(section, key)?;
                for v in value {
                    println!("{}", v);
                }
            } else {
                // 否则列出所有配置
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
