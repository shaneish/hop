use dirs::home_dir;
use std::path::{Path, PathBuf};
use std::{env::var, fs};

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
        let config_path = config_dir.join("bhop.toml");
        let script_dir = config_dir.join("scripts");
        let db_path = config_dir.join("db").join("bhop.db");
        if !config_dir.exists() {
            fs::create_dir(config_dir).expect("Failed to create config directory.");
        };
        if !config_path.exists() {
            // Add default hop.toml to config directory
            let default_toml = if cfg!(windows) {
                "src/defaults/windows_defaults.toml"
            } else {
                "src/defaults/unix_defaults.toml"
            };
            fs::copy(default_toml, &config_path)
                .expect("Failed to copy bhop.toml to config directory");
        };
        if !db_path.exists() {
            fs::create_dir(db_path.parent().unwrap())
                .expect("Failed to create database directory.");
            Self::create_database(&db_path).expect("Failed to create database");
        };
        if !script_dir.exists() {
            fs::create_dir(&script_dir).expect("Failed to create script directory.");
        };
        Environment {
            config_path,
            db_path,
        }
    }

    fn create_database(db_path: impl AsRef<Path>) -> anyhow::Result<()> {
        let conn = sqlite::open(db_path.as_ref())?;
        conn.execute(
            "CREATE TABLE IF NOT EXISTS shortcuts (
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
        Ok(())
    }
}

impl Default for Environment {
    fn default() -> Self {
        Self::new()
    }
}
