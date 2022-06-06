use std::{fs, collections::HashSet};

use serde_derive::Deserialize;

use crate::restaurant::Menu;

#[derive(Debug, Clone, Deserialize)]
pub struct RestaurantConfig {
    #[serde(rename = "table")]
    pub n_table: u64,
    pub menus: HashSet<Menu>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct NetworkConfig {
    pub ip: String,
    pub port: u64,
}

#[derive(Debug, Clone, Deserialize)]
pub struct Config {
    pub restaurant: RestaurantConfig,
    pub network: NetworkConfig,
}

impl Config {
    pub fn from_file(file: &str) -> Self {
        let toml_string = fs::read_to_string(file).expect(
            "Error when trying to read the config file.\
            Make sure restaurant.toml is in /config directory.",
        );
        Self::from_toml_string(&toml_string)
    }

    pub fn from_toml_string(input: &str) -> Self {
        toml::from_str(&input).expect("Error parsing TOML in the config file.")
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_read_config() {
        let _config = Config::from_toml_string(
            r###"
[restaurant]
table = 1234
menus = ["a", "b"]

[network]
ip = "1.1.1.1"
port = 1234
"###,
        );
        assert!(matches!(
            Config {
                restaurant: RestaurantConfig {
                    n_table: 1234,
                    menus: vec![
                        "a".into(),
                        "b".into(),
                        "c".into(),
                    ].into_iter().collect()
                },
                network: NetworkConfig { ip: "1.1.1.1".into(), port: 1234 }
            },
            _config
        ));
    }
}
