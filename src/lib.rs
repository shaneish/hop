pub mod args;
use dirs::home_dir;
use press_btn_continue;
use project_root::get_project_root;
use rusqlite::Connection;
use serde_derive::Deserialize;
use std::{
    env::{current_dir, var},
    fs,
    fs::read_dir,
    io::{Error, Write},
    iter::empty,
    path::{Path, PathBuf},
    time::SystemTime,
};
use toml::from_str;

#[derive(Deserialize, PartialEq, Debug)]
pub struct Config {
    pub editor: String,
    pub max_history_entries: usize,
    pub ls_display_block: usize,
}

pub fn extract_path_ending(current_path: String) -> String {
    match &current_path.rsplit_once("/") {
        Some((_, end)) => end.to_string(),
        None => match &current_path.rsplit_once("\\") {
            Some((_, end)) => end.to_string(),
            None => current_path,
        },
    }
}

pub struct Env {
    pub config_file: PathBuf,
    pub database_file: PathBuf,
}

impl Env {
    fn read() -> Self {
        let home_dir = home_dir().unwrap_or(PathBuf::from("~/"));
        let mut config_dir = match var("HOP_CONFIG_DIRECTORY") {
            Ok(loc) => PathBuf::from(&loc),
            Err(_) => {
                home_dir.push(".config");
                home_dir.push("hop");
                home_dir
            }
        };
        match var("HOP_CONFIG_FILE_NAME") {
            Ok(name) => config_dir.push(name),
            Err(_) => config_dir.push("hop.toml"),
        };
        let mut database_dir = match var("HOP_DATABASE_DIRECTORY") {
            Ok(loc) => PathBuf::from(&loc),
            Err(_) => {
                let mut db_dir_temp = PathBuf::from(&config_dir);
                db_dir_temp.push("db");
                db_dir_temp
            }
        };
        match var("HOP_DATABASE_FILE_NAME") {
            Ok(name) => database_dir.push(name),
            Err(_) => database_dir.push("hop.sqlite"),
        };

        Env {
            config_file: config_file,
            database_file: database_file,
        }
    }
}

// Suppressing assignment warnings as functionality that uses `config` will be added in the future.
#[allow(dead_code)]
pub struct Hopper {
    config: Config,
    env: Env,
    db: Connection,
}

impl Hopper {
    pub fn new() -> Self {
        let env = Env::read();
        if !env.config_file.exists() {
            fs::create_dir_all(
                env.config_file
                    .parent()
                    .expect("[error] Unable to create config directory."),
            )
            .expect("[error] Unable to create config directory.");
            let mut new_conf = fs::File::create(env.config_file.clone())
                .expect("[error] Unable to create config file.");
            new_conf
                .write_all(b"editor=\"nvim\"\nmax_history_entries=200\nls_display_block=10")
                .expect("[error] Unable to generate default config file.");
        };
        let toml_str: String = fs::read_to_string(env.config_file.clone()).unwrap();
        let configs = from_str(&toml_str).unwrap_or(Config {
            editor: "nvim".to_string(),
            max_history_entries: 200,
            ls_display_block: 10,
        });
        let db_doesnt_exist = !env.database_file.exists();
        let mut conn = Connection::open(&env.database_file)
            .expect("[error] Unable to create new database at specified location.");
        if db_doesnt_exist {
            conn.execute(
                "CREATE TABLE IF NOT EXISTS named_hops (
                name TEXT PRIMARY KEY,
                location TEXT NOT NULL,
                )",
                [],
            );
            conn.execute(
                "CREATE TABLE IF NOT EXISTS history (
                time INTEGER,
                name TEXT NOT NULL unique,
                location TEXT NOT NULL,
                )",
                [],
            );
        }

        Hopper {
            config: configs,
            env: env,
            db: conn,
        }
    }

    pub fn add_hop<T: AsRef<Path>>(&mut self, path: T, name: &str) -> rusqlite::Result<String> {
        let query = format!(
            "INSERT OR REPLACE INTO named_hops (name, location) VALUES (\"{}\", \"{}\")",
            name,
            path.as_ref().display().to_string()
        );
        self.db.execute(&query, [])?;
        Ok(format!("[info] Hop created for {}.", name))
    }

    pub fn hop(&self, name: &str) -> rusqlite::Result<String> {
        let query = format!("SELECT location FROM named_hops WHERE name=\"{}\"", name);
        let mut query_result = self.db.prepare(&query)?;
        let location = query_result.query_map([], |row| Ok(row.get(0)?)).next()?;
        Ok(format!("__cd__:{}", location))
    }

    pub fn groove(&self, name: &str) -> String {
        match self.hop(name) {
            Ok(cmd) => cmd,
            Err(_) => match self.cd(name) {
                Some((dir, name)) => {
                    self.log_history(dir, name);
                    format!("__cd__:{}", dir)
                }
                None => {
                    println!("[error] Unable to find or disambiguate hop to {}.", name);
                    "".to_string()
                }
            },
        }
    }

    pub fn log_history(&mut self, location: String, name: String) -> rusqlite::Result<()> {
        let query = format!(
            "INSERT INTO history (time, name, location) VALUES ({}, \"{}\", \"{}\") ",
            SystemTime::now(),
            name,
            location
        );
        self.db.execute(&query, [])?;
        Ok(())
    }

    pub fn cd(&self, name: &str) -> Result<String> {
        read_dir(current_dir().unwrap())
            .expect("[error] Unable to search contents of current directory.")
            .map(|f| f.unwrap().path().display().to_string())
            .map(|f| (f, extract_path_ending(f)))
            .find(|(f, path_end)| &path_end == name)
    }

    pub fn list_hops(&self) -> rusqlite::Result<String> {
        let query = format!("SELECT name, location FROM named_hops");
        let mut query_result = self.db.prepare(&query)?;
        let hops: Vec<(String, String)> = query_result
            .query_map([], |row| Ok((row.get(0)?, row.get(1)?)))
            .collect();
        let max_name_size = hops.iter().map(|(name, _)| name.len()).max().unwrap_or(0);
        let formatted_hops: Vec<String> = hops
            .into_iter()
            .map(|(name, location)| {
                (
                    String::from_utf8(vec![b' '; max_name_size - name.len() + 1])
                        .unwrap_or(" ".to_string()),
                    name,
                    location,
                )
            })
            .map(|(ws, name, location)| format!("{}{}-> {}", name, ws, location))
            .collect();
        formatted_hops.sort();
        for (idx, hop) in formatted_hops.into_iter().enumerate() {
            println!("{}", hop);
            if idx % self.config.ls_display_block == 0 {
                press_btn_continue::wait("Press any key to continue...")
                    .expect("[error] User input failed.");
            }
        }
        Ok("".to_string())
    }

    pub fn hop_names(&self) -> rusqlite::Result<Vec<String>> {
        let query = format!("SELECT name FROM named_hops");
        let mut query_result = self.db.prepare(&query)?;
        Ok(query_result.query_map([], |row| Ok(row.get(0)?)).collect())
    }

    pub fn brb<T: AsRef<Path>>(&self, path: T) -> Result<String, rusqlite::Error> {
        self.add_hop(path.as_ref(), "back")?;
        Ok("".to_string())
    }

    pub fn execute(cmd: args::Cmd) {
        todo!()
    }
}
