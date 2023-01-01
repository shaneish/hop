use dirs::home_dir;
use serde_derive::Deserialize;
use std::{convert::AsRef, fs, io, io::Write, path::Path};
use symlink;
use toml::from_str;

#[derive(Deserialize, PartialEq, Debug)]
pub struct Defaults {
    pub editor: String,
}

#[derive(Deserialize, PartialEq, Debug)]
pub struct Config {
    pub defaults: Defaults,
}

// Suppressing assignment warnings as functionality that uses `config` will be added in the future.
#[allow(dead_code)]
pub struct Hopper {
    pub config: Config,
    pub home_dir: String,
    pub config_dir: String,
    pub config_file: String,
}

impl Hopper {
    pub fn from(toml_str: &str, config_location: &str) -> Self {
        let home_dir = home_dir().unwrap().into_os_string().into_string().unwrap();
        Hopper {
            config: Self::read_configs_from_str(toml_str).unwrap(),
            home_dir: home_dir.clone(),
            config_dir: format!("{}/{}", home_dir.clone(), config_location),
            config_file: format!("{}/{}/config.toml", home_dir, config_location),
        }
    }

    pub fn new(config_location: &str) -> Self {
        let home_dir = home_dir().unwrap().into_os_string().into_string().unwrap();
        let root_hopper = Hopper {
            config: Config {
                defaults: Defaults {
                    editor: "nvim".to_string(),
                },
            },
            home_dir: home_dir.clone(),
            config_dir: format!("{}/{}", home_dir.clone(), config_location),
            config_file: format!("{}/{}/config.toml", home_dir, config_location),
        };
        root_hopper.from_root()
    }

    fn from_root(&self) -> Self {
        Hopper {
            config: self.read_configs().unwrap(),
            home_dir: self.home_dir.clone(),
            config_dir: self.config_dir.clone(),
            config_file: self.config_file.clone(),
        }
    }

    // Below function is used for unit testing but not by compiled program.  That's why warnings
    // are suppressed.
    #[allow(unused_assignments)]
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
        Self::read_configs_from_str(&toml_str)
    }

    pub fn add_hop<T: AsRef<Path>>(&self, path: T, name: &str) -> String {
        let sym_result =
            symlink::symlink_dir(path.as_ref(), format!("{}/{}", self.config_dir, name));
        match sym_result {
            Ok(_) => format!(
                "echo \"[hop] {} -> {}\"",
                name,
                path.as_ref().display().to_string()
            ),
            Err(_) => format!(
                "echo \"[error] unable to add hop {} -> {}\"",
                name,
                path.as_ref().display().to_string()
            ),
        }
    }

    pub fn hop(&self, name: &str) -> String {
        format!("cd {}/{}", self.config_dir, name)
    }

    pub fn list_hops(&self) -> String {
        let output: String = fs::read_dir(self.config_dir.clone())
            .unwrap()
            .map(|p| p.unwrap().path().display().to_string())
            .filter(|p| fs::metadata(p).unwrap().is_dir())
            .map(|p| {
                format!(
                    "{} -> {}",
                    p.clone().split("/").last().unwrap(),
                    fs::read_link(p).unwrap().display().to_string()
                )
            })
            .collect::<Vec<String>>()
            .join("\n");
        format!("echo \"{}\"", output)
    }
}
