use serde_derive::Deserialize;
use std::collections::HashMap;
use std::path::PathBuf;
use std::{env::var, fs};
use toml::from_str;

#[derive(Deserialize, PartialEq, Debug, Default)]
pub struct ReadConfig {
    pub settings: Option<ReadSettings>,
    pub editors: Option<HashMap<String, String>>,
}

#[derive(Deserialize, PartialEq, Debug, Default)]
pub struct ReadSettings {
    pub default_editor: Option<String>,
    pub ls_display_block: Option<usize>,
    pub print_color_primary: Option<[u8; 3]>,
    pub print_color_secondary: Option<[u8; 3]>,
    pub verbose: Option<bool>,
    pub prioritize_shortcuts: Option<bool>,
    pub always_jump: Option<bool>,
    pub search_match_prefix: Option<String>,
    pub search_match_suffix: Option<String>,
}

impl ReadConfig {
    pub fn new(config_path: &PathBuf) -> Self {
        let toml_str: String =
            fs::read_to_string(config_path).expect("[error] Unable to read config file location.");
        let read_config: ReadConfig =
            from_str(&toml_str).expect("[error] Unable to parse configuration TOML.");
        let read_settings = match read_config.settings {
            Some(settings) => settings,
            None => ReadSettings::default(),
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
    pub search_match_prefix: String,
    pub search_match_suffix: String,
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
        let search_match_prefix = settings.search_match_prefix.unwrap_or("*".to_string());
        let search_match_suffix = settings.search_match_suffix.unwrap_or("*".to_string());
        let editors = read_config.editors.unwrap_or(HashMap::new());
        Configs {
            default_editor,
            ls_display_block,
            print_color_primary,
            print_color_secondary,
            verbose,
            prioritize_shortcuts,
            always_jump,
            search_match_prefix,
            search_match_suffix,
            editors,
        }
    }
}
