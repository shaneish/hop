use dirs::home_dir;
use serde_derive::Deserialize;
use std::collections::HashMap;
use std::path::PathBuf;
use std::{env::var, fs};
use toml::from_str;

#[derive(Debug, Clone)]
pub struct Environment {
    pub config_path: PathBuf,
    pub script_dir: PathBuf,
    pub db_path: PathBuf,
}

impl Environment {
    pub fn new() -> Self {
        let mut home_dir = home_dir().unwrap_or(PathBuf::from("~/"));
        let config_dir = match var("HOP_CONFIG_DIRECTORY") {
            Ok(loc) => PathBuf::from(&loc),
            Err(_) => {
                let config_dir_temp = home_dir.clone();
                config_dir_temp.push(".config");
                config_dir_temp.push("bhop");
                config_dir_temp
            }
        };
        let config_path = config_dir.clone().join("bhop.toml");
        let mut script_dir = match var("HOP_SCRIPT_DIRECTORY") {
            Ok(loc) => PathBuf::from(&loc),
            Err(_) => {
                let mut script_dir_temp =
                    PathBuf::from(format!("{}", &config_dir.as_path().display()));
                script_dir_temp.push("scripts");
                script_dir_temp
            }
        };
        let mut db_path = match var("HOP_DATABASE_DIRECTORY") {
            Ok(loc) => PathBuf::from(&loc),
            Err(_) => {
                let mut db_dir_temp = PathBuf::from(format!("{}", &config_dir.as_path().display()));
                db_dir_temp.push("db");
                db_dir_temp.push("bhop.db");
                db_dir_temp
            }
        };
        Environment {
            config_path,
            script_dir,
            db_path,
        }
    }
}

#[derive(Deserialize, PartialEq, Debug)]
struct ReadConfig {
    pub settings: Option<ReadSettings>,
    pub editors: Option<HashMap<String, String>>,
}

#[derive(Deserialize, PartialEq, Debug)]
struct ReadSettings {
    pub default_editor: Option<String>,
    pub max_history: Option<usize>,
    pub ls_display_block: Option<usize>,
    pub print_color_primary: Option<[u8; 3]>,
    pub print_color_secondary: Option<[u8; 3]>,
    pub verbose: Option<bool>,
}

impl ReadConfig {
    pub fn new(environment: Environment) -> Self {
        let toml_str: String = fs::read_to_string(environment.config_path)
            .expect("[error] Unable to read config file location.");
        let read_config: ReadConfig =
            from_str(&toml_str).expect("[error] Unable to parse configuration TOML.");
        let read_settings = match read_config.settings {
            Some(settings) => settings,
            None => ReadSettings {
                default_editor: None,
                max_history: None,
                ls_display_block: None,
                print_color_primary: None,
                print_color_secondary: None,
                verbose: None,
            },
        };
        let read_editors = match read_config.editors {
            Some(editors) => editors,
            None => HashMap::new(),
        };
        ReadConfig {
            settings: Some(read_settings),
            editors: Some(read_editors),
        }
    }
}

#[derive(Debug)]
pub struct Configs {
    pub default_editor: String,
    pub max_history: usize,
    pub ls_display_block: usize,
    pub print_color_primary: [u8; 3],
    pub print_color_secondary: [u8; 3],
    pub verbose: bool,
    pub editors: HashMap<String, String>,
    pub environment: Environment,
}

impl Configs {
    pub fn new(environment: Environment) -> Self {
        let read_config = ReadConfig::new(environment.clone());
        let default_editor = match read_config.settings.unwrap().default_editor {
            Some(editor) => editor,
            None => match var("EDITOR") {
                Ok(editor) => editor,
                Err(_) => {
                    if cfg!(target_os = "windows") {
                        String::from("notepad")
                    } else {
                        String::from("vi")
                    }
                }
            },
        };
        let max_history = match read_config.settings.unwrap().max_history {
            Some(history) => history,
            None => 0,
        };
        let ls_display_block = match read_config.settings.unwrap().ls_display_block {
            Some(block) => block,
            None => 0,
        };
        let print_color_primary = match read_config.settings.unwrap().print_color_primary {
            Some(color) => color,
            None => [51, 255, 255],
        };
        let print_color_secondary = match read_config.settings.unwrap().print_color_secondary {
            Some(color) => color,
            None => [51, 255, 153],
        };
        let verbose = match read_config.settings.unwrap().verbose {
            Some(verbose) => verbose,
            None => false,
        };
        let editors = match read_config.editors {
            Some(editors) => editors,
            None => HashMap::new(),
        };
        Configs {
            default_editor,
            max_history,
            ls_display_block,
            print_color_primary,
            print_color_secondary,
            verbose,
            editors,
            environment,
        }
    }
}
