use dirs::home_dir;
use serde_derive::Deserialize;
use std::{
    fs,
    io,
    io::Write,
    path::Path,
    convert::AsRef,
    env,
};
use symlink;
use toml::from_str;

#[derive(Deserialize, PartialEq, Debug)]
struct Defaults {
    editor: String,
}

#[derive(Deserialize, PartialEq, Debug)]
pub struct Config {
    defaults: Defaults,
}

fn read_configs_from_str(toml_str: &str) -> io::Result<Config> {
    Ok(from_str(toml_str)?)
}

pub fn read_configs() -> io::Result<Config> {
    let hop_config_dir = format!(
        "{}/.config/hop",
        home_dir().unwrap().into_os_string().into_string().unwrap()
    );
    let conf_path = format!("{}/conf.toml", hop_config_dir);
    if !Path::new(conf_path.as_str()).exists() {
        fs::create_dir_all(hop_config_dir)?;
        let mut new_conf = fs::File::create(conf_path.as_str())?;
        new_conf.write_all(b"[defaults]\neditor=\"nvim\"")?;
    }
    let toml_str: String = fs::read_to_string(conf_path.as_str()).unwrap();
    read_configs_from_str(&toml_str)
}


pub struct Hopper {
    config: Config,
    home_dir: String,
    config_dir: String,
    config_file: String,
}

impl Hopper {
    pub fn from(toml_str: &str) -> Self {
        let home_dir = home_dir().unwrap().into_os_string().into_string().unwrap();
        Hopper {
            config: self::read_configs_from_str(toml_str).unwrap(),
            home_dir: home_dir.clone(),
            config_dir: format!("{}/.config/hop", home_dir.clone()),
            config_file: format!("{}/.config/hop/config.toml", home_dir),

        }
    }

    pub fn new() -> Self {
        let home_dir = home_dir().unwrap().into_os_string().into_string().unwrap();
        let root_hopper = Hopper {
            config: Config { defaults: Defaults { editor: "nvim".to_string() }},
            home_dir: home_dir.clone(),
            config_dir: format!("{}/.config/hop", home_dir.clone()),
            config_file: format!("{}/.config/hop/config.toml", home_dir),
        };
        root_hopper.from_root()
    }

    pub fn from_root(&self) -> Self {
        let home_dir = home_dir().unwrap().into_os_string().into_string().unwrap();
        Hopper {
            config: self.read_configs().unwrap(),
            home_dir: self.home_dir.clone(),
            config_dir: self.config_dir.clone(),
            config_file: self.config_file.clone(),
        }
    }

    fn read_configs_from_str(toml_str: &str) -> io::Result<Config> {
        Ok(from_str(toml_str)?)
    }

    fn read_configs(&self) -> io::Result<Config> {
        if !Path::new(self.config_file.as_str()).exists() {
            fs::create_dir_all(self.config_dir.clone())?;
            let mut new_conf = fs::File::create(self.config_file.clone())?;
            new_conf.write_all(b"[defaults]\neditor=\"nvim\"")?;
        }
        let toml_str: String = fs::read_to_string(self.config_file.as_str()).unwrap();
        read_configs_from_str(&toml_str)
    }

    pub fn add_hop<T: AsRef<Path>>(&self, path: T, name: &str) -> io::Result<()> {
        let sym_result = symlink::symlink_dir(path.as_ref(), format!("{}/{}", self.config_dir, name));
        match sym_result {
            Ok(_) => println!("[hop] {} -> {}", name, path.as_ref().display().to_string()),
            Err(_) => println!(
                "[error] unable to add hop {} -> {}",
                name,
                path.as_ref().display().to_string()
            ),
        };
        sym_result
    }

    pub fn hop(&self, name: &str) -> io::Result<()> {
        env::set_current_dir(&format!("{}/{}", self.config_dir, name))
    }

    pub fn list_hops(&self) {
        fs::read_dir(self.config_dir.clone()).unwrap()
            .map(|p| p.unwrap().path().display().to_string())
            .filter(|p| fs::metadata(p).unwrap().is_dir())
            .map(|p| format!("{} -> {}", p.clone().split("/").last().unwrap(), fs::read_link(p).unwrap().display().to_string()))
            .for_each(|p| println!("{}", p));
    }
}

#[test]
fn test_reading_toml() {
    let toml_str = "[defaults]\neditor=\"nvim\"";
    let hopper = Hopper::from(toml_str);

    let expected = Config {
        defaults: Defaults {
            editor: "nvim".to_string(),
        },
    };

    assert_eq!(hopper.config, expected);
}
