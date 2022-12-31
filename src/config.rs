use toml::from_str;
use serde_derive::Deserialize;
use std::{
    io::{
        Write,
        Result,
    },
    path::Path,
    fs::{
        read_to_string,
        File,
        create_dir_all,
    }
};
use dirs::home_dir;

#[derive(Deserialize, PartialEq, Debug)]
struct Defaults {
    editor: String,
}

#[derive(Deserialize, PartialEq, Debug)]
pub struct Config {
    defaults: Defaults,
}

fn read_configs_from_str(toml_str: &str) -> Result<Config> {
    Ok(from_str(toml_str)?)
}

pub fn read_configs() -> Result<Config> {
    let hop_config_dir = format!("{}/.config/hop", home_dir().unwrap().into_os_string().into_string().unwrap());
    let conf_path = format!("{}/conf.toml", hop_config_dir);
    if !Path::new(conf_path.as_str()).exists() {
        create_dir_all(hop_config_dir)?;
        let mut new_conf = File::create(conf_path.as_str())?;
        new_conf.write_all(b"[defaults]\neditor=\"nvim\"")?;
    }
    let toml_str: String = read_to_string(conf_path.as_str()).unwrap();
    read_configs_from_str(&toml_str)
}

#[test]
fn test_reading_toml() {
    let toml_str: String = r#"[defaults]
editor="nvim""#.to_string();

    let expected = Config {
        defaults: Defaults { editor: "nvim".to_string() },
    };

    assert_eq!(read_configs_from_str(&toml_str).unwrap(), expected);
}
