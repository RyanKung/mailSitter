use std::error::Error;
use serde_yaml;
use serde_json::Value;
use std::fs;
use std::path::PathBuf;
use serde::Serialize;
use serde::de::DeserializeOwned;

#[cfg(feature="ddep")]
pub mod ddep;
pub mod email;
#[cfg(test)]
pub mod tests;
pub mod utils;

/// Config trait
pub trait Config: Sized + Serialize + DeserializeOwned {
    /// Recursively merge two serde_json::Value objects.
    fn merge_configs(a: &mut Value, b: &Value) {
	match (a, b) {
            (Value::Object(a_obj), Value::Object(b_obj)) => {
		for (k, v) in b_obj {
                    Self::merge_configs(a_obj.entry(k).or_insert(Value::Null), v);
		}
            }
            (a, b) => {
		*a = b.clone();
            }
	}
    }

    /// Read config from path
    fn read(path: &str) -> Result<Self, Box<dyn Error>> {
        let config_content = fs::read_to_string(path)?;
        let config: Self = serde_yaml::from_str(&config_content)?;
        Ok(config)
    }


    /// Save config to path
    fn save(&self, path: &str) -> Result<(), Box<dyn Error>> {
	// Convert self to a YAML string.
	let new_config_content = serde_yaml::to_string(self)?;

	// Parse the new configuration as serde_json::Value.
	let new_config: Value = serde_yaml::from_str(&new_config_content)?;

	let config_path = PathBuf::from(path);

	// If the config file exists, read and merge it with the new configuration.
	if config_path.exists() {
	    // Read the existing configuration file.
	    let existing_config_content = fs::read_to_string(&config_path)?;
	    let mut existing_config: Value = serde_yaml::from_str(&existing_config_content)?;

	    // Merge new_config into existing_config.
	    Self::merge_configs(&mut existing_config, &new_config);

	    // Convert the merged configuration back to a YAML string.
	    let merged_config_content = serde_yaml::to_string(&existing_config)?;

	    // Write the merged configuration back to the file.
	    fs::write(config_path, merged_config_content)?;
	} else {
	    // If the file does not exist, write the new configuration.
	    if let Some(parent) = config_path.parent() {
		fs::create_dir_all(parent)?;
	    }
	    fs::write(config_path, new_config_content)?;
	}

	Ok(())
    }
}
