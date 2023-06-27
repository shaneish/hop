use dirs::home_dir;
use serde_derive::Deserialize;
use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::{env::var, fs};
use toml::from_str;

#[derive(Debug, Clone)]
pub struct Environment {
    pub config_path: PathBuf,
    pub db_path: PathBuf,
}

impl Environment {
    pub fn new() -> Self {
        let config_dir = match var("HOP_CONFIG_DIRECTORY") {
            Ok(loc) => PathBuf::from(&loc),
            Err(_) => {
                let mut config_dir_temp = home_dir().unwrap_or(PathBuf::from("~/"));
                config_dir_temp.push(".config");
                config_dir_temp.push("bhop");
                config_dir_temp
            }
        };
        let config_path = config_dir.clone().join("bhop.toml");
        let script_dir = config_dir.clone().join("scripts");
        let db_path = config_dir.clone().join("db").join("bhop.db");
        if !config_dir.exists() {
            // Move runner scripts to script config directory
            Self::copy_dir_all("runners", &script_dir)
                .expect("Failed to copy runner scripts to script directory");

            // Add default hop.toml to config directory
            let default_toml = if cfg!(windows) {
                "defaults/windows_defaults.toml"
            } else {
                "defaults/unix_defaults.toml"
            };
            fs::copy(default_toml, &config_path)
                .expect("Failed to copy hop.toml to config directory");

            // Create database if it doesn't exist
            Self::create_database(&config_path, &db_path).expect("Failed to create database");
        };
        Environment {
            config_path,
            db_path,
        }
    }

    fn copy_dir_all(src: impl AsRef<Path>, dst: impl AsRef<Path>) -> anyhow::Result<()> {
        fs::create_dir_all(&dst)?;
        for entry in fs::read_dir(src)? {
            let entry = entry?;
            let ty = entry.file_type()?;
            if ty.is_dir() {
                Self::copy_dir_all(entry.path(), dst.as_ref().join(entry.file_name()))?;
            } else {
                fs::copy(entry.path(), dst.as_ref().join(entry.file_name()))?;
            }
        }
        Ok(())
    }

    fn create_database(
        config_path: impl AsRef<Path>,
        db_path: impl AsRef<Path>,
    ) -> anyhow::Result<()> {
        let db_path = db_path.as_ref();
        let conn = sqlite::open(db_path)?;
        conn.execute(
            "CREATE TABLE IF NOT EXISTS named_hops (
            name TEXT PRIMARY KEY,
            location TEXT NOT NULL
            )",
        )?;
        conn.execute(
            "CREATE TABLE IF NOT EXISTS history (
            name TEXT NOT NULL,
            location TEXT NOT NULL,
            usage INTEGER NOT NULL,
            PRIMARY KEY (name, location)
            )",
        )?;
        conn.execute(format!(
            "INSERT INTO named_hops (\"config\", \"{}\")",
            config_path.as_ref().display().to_string()
        ))?;
        Ok(())
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
    pub ls_display_block: Option<usize>,
    pub print_color_primary: Option<[u8; 3]>,
    pub print_color_secondary: Option<[u8; 3]>,
    pub verbose: Option<bool>,
    pub prioritize_shortcuts: Option<bool>,
    pub always_jump: Option<bool>,
}

impl ReadConfig {
    pub fn new(config_path: &PathBuf) -> Self {
        let toml_str: String =
            fs::read_to_string(config_path).expect("[error] Unable to read config file location.");
        let read_config: ReadConfig =
            from_str(&toml_str).expect("[error] Unable to parse configuration TOML.");
        let read_settings = match read_config.settings {
            Some(settings) => settings,
            None => ReadSettings {
                default_editor: None,
                ls_display_block: None,
                print_color_primary: None,
                print_color_secondary: None,
                verbose: None,
                prioritize_shortcuts: None,
                always_jump: None,
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
    pub ls_display_block: usize,
    pub print_color_primary: [u8; 3],
    pub print_color_secondary: [u8; 3],
    pub verbose: bool,
    pub prioritize_shortcuts: bool,
    pub always_jump: bool,
    pub editors: HashMap<String, String>,
}

impl Configs {
    pub fn new(config_path: &PathBuf) -> Self {
        let read_config = ReadConfig::new(config_path);
        let settings = read_config.settings.unwrap();
        let default_editor = match settings.default_editor {
            Some(editor) => editor,
            None => match var("EDITOR") {
                Ok(sys_editor) => sys_editor,
                Err(_) => {
                    if cfg!(target_os = "windows") {
                        String::from("notepad")
                    } else {
                        String::from("vi")
                    }
                }
            },
        };
        let ls_display_block = settings.ls_display_block.unwrap_or(0);
        let print_color_primary = settings.print_color_primary.unwrap_or([51, 255, 255]);
        let print_color_secondary = settings.print_color_secondary.unwrap_or([51, 255, 153]);
        let verbose = settings.verbose.unwrap_or(false);
        let prioritize_shortcuts = settings.prioritize_shortcuts.unwrap_or(false);
        let always_jump = settings.always_jump.unwrap_or(false);
        let editors = read_config.editors.unwrap_or(HashMap::new());
        Configs {
            default_editor,
            ls_display_block,
            print_color_primary,
            print_color_secondary,
            verbose,
            prioritize_shortcuts,
            always_jump,
            editors,
        }
    }
}
