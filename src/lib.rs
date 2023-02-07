pub mod args;
use dirs::home_dir;
use press_btn_continue;
use serde_derive::Deserialize;
use chrono::Local;
use std::{
    env::{current_dir, var},
    fs,
    fs::read_dir,
    io::Write,
    path::{Path, PathBuf},
};
use toml::from_str;
use sqlite;
use anyhow;

#[derive(Deserialize, PartialEq, Debug)]
pub struct Config {
    pub editor: String,
    pub max_history_entries: usize,
    pub ls_display_block: usize,
}

pub fn extract_path_ending<T: AsRef<Path>>(current_path: T) -> String {
    let current_path_string = current_path.as_ref().display().to_string();
    match current_path_string.rsplit_once("/") {
        Some((_, end)) => end.to_string(),
        None => match current_path_string.rsplit_once("\\") {
            Some((_, end)) => end.to_string(),
            None => current_path_string,
        },
    }
}

#[derive(Debug)]
pub struct Env {
    pub config_file: PathBuf,
    pub database_file: PathBuf,
}

impl Env {
    fn read() -> Self {
        let mut home_dir = home_dir().unwrap_or(PathBuf::from("~/"));
        let config_dir = match var("HOP_CONFIG_DIRECTORY") {
            Ok(loc) => PathBuf::from(&loc),
            Err(_) => {
                home_dir.push(".config");
                home_dir.push("hop");
                home_dir
            }
        };
        let mut hop_config_file = PathBuf::from(&config_dir);
        match var("HOP_CONFIG_FILE_NAME") {
            Ok(name) => hop_config_file.push(name),
            Err(_) => hop_config_file.push("hop.toml"),
        };
        let mut database_dir = match var("HOP_DATABASE_DIRECTORY") {
            Ok(loc) => PathBuf::from(&loc),
            Err(_) => {
                let mut db_dir_temp = PathBuf::from(&format!("{}", &config_dir.as_path().display().to_string()));
                db_dir_temp.push("db");
                db_dir_temp
            }
        };
        if !Path::new(&database_dir).exists() {
            match fs::create_dir_all(&database_dir) {
                Ok(_) => {},
                Err(e) => println!("[error] Error creating database directory: {}", e),
            };
        };
        match var("HOP_DATABASE_FILE_NAME") {
            Ok(name) => database_dir.push(name),
            Err(_) => database_dir.push("hop.sqlite"),
        };

        Env {
            config_file: hop_config_file,
            database_file: database_dir,
        }
    }
}

// Suppressing assignment warnings as functionality that uses `config` will be added in the future.
#[allow(dead_code)]
pub struct Hopper {
    config: Config,
    env: Env,
    db: sqlite::Connection,
}

impl Hopper {
    pub fn new() -> anyhow::Result<Self> {
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
                .write_all(b"editor=\"nvim\"\nmax_history_entries=200\nls_display_block=0")
                .expect("[error] Unable to generate default config file.");
        };
        let toml_str: String = fs::read_to_string(env.config_file.clone()).unwrap();
        let configs = from_str(&toml_str).unwrap_or(Config {
            editor: "nvim".to_string(),
            max_history_entries: 200,
            ls_display_block: 10,
        });
        let conn = sqlite::open(&env.database_file)?;
        conn.execute(
            "CREATE TABLE IF NOT EXISTS named_hops (
            name TEXT PRIMARY KEY,
            location TEXT NOT NULL
            )"
        )?;
        conn.execute(
            "CREATE TABLE IF NOT EXISTS history (
            time TEXT,
            name TEXT NOT NULL unique,
            location TEXT NOT NULL
            )",
        )?;

        Ok(Hopper {
            config: configs,
            env: env,
            db: conn,
        })
    }

    pub fn add_hop<T: AsRef<Path>>(&mut self, path: T, name: &str) -> anyhow::Result<String> {
        let query = format!(
            "INSERT OR REPLACE INTO named_hops (name, location) VALUES (\"{}\", \"{}\")",
            name,
            path.as_ref().display().to_string()
        );
        self.db.execute(&query)?;
        Ok(format!("[info] Hop created for {}.", name))
    }

    pub fn hop(&self, name: &str) -> anyhow::Result<String> {
        let query = format!("SELECT location FROM named_hops WHERE name=\"{}\"", name);
        let statement = self.db.prepare(&query)?;
        let location = statement.read::<String, _>("location")?;
        Ok(format!("__cd__:{}", location))
    }

    pub fn groove(&mut self, name: &str) -> anyhow::Result<String> {
        match self.hop(name) {
            Ok(cmd) => Ok(cmd),
            Err(e) => match self.cd(name) {
                Some((dir, n)) => {
                    self.log_history(dir.clone(), n)?;
                    Ok(format!("__cd__:{}", dir))
                }
                None => {
                    Err(e)
                }
            },
        }
    }

    pub fn log_history(&mut self, location: String, name: String) -> anyhow::Result<()> {
        let query = format!(
            "INSERT INTO history (time, name, location) VALUES ({}, \"{}\", \"{}\") ",
            Local::now().format("%Y%m%d%H%M%S"),
            name,
            location
        );
        self.db.execute(&query)?;
        Ok(())
    }

    pub fn cd(&self, name: &str) -> Option<(String, String)> {
        read_dir(current_dir().unwrap())
            .expect("[error] Unable to search contents of current directory.")
            .filter(|f| f.is_ok())
            .map(|f| f.unwrap().path())
            .map(|f| (f.as_path().display().to_string(), extract_path_ending(f.to_path_buf())))
            .find(|(_, path_end)| path_end == name)
    }

    pub fn list_hops(&self) -> anyhow::Result<String> {
        let query = format!("SELECT name, location FROM named_hops");
        let mut query_result = self.db.prepare(&query)?;
        let mut hops: Vec<(String, String)> = Vec::new();
        while let Ok(sqlite::State::Row) = query_result.next() {
            let name = query_result.read::<String, _>("name")?;
            let location = query_result.read::<String, _>("location")?;
            hops.push((name, location));
        }
        let max_name_size = hops.iter().map(|(name, _)| name.len()).max().unwrap_or(0);
        let mut formatted_hops: Vec<String> = hops
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
            if (self.config.ls_display_block != 0) && (idx % self.config.ls_display_block == 0) {
                press_btn_continue::wait("Press any key to continue...")
                    .expect("[error] User input failed.");
            }
        }
        Ok("".to_string())
    }

    pub fn hop_names(&self) -> anyhow::Result<Vec<String>> {
        let query = format!("SELECT name FROM named_hops");
        let mut query_result = self.db.prepare(&query)?;
        let mut hops: Vec<String> = Vec::new();
        while let Ok(sqlite::State::Row) = query_result.next() {
            let name = query_result.read::<String, _>("name")?;
            hops.push(name);
        }
        Ok(hops)
    }

    pub fn brb<T: AsRef<Path>>(&mut self, path: T) -> anyhow::Result<String> {
        self.add_hop(path.as_ref(), "back")?;
        Ok("".to_string())
    }

    pub fn execute(&mut self, cmd: args::Cmd) {
        let output = match cmd {
            args::Cmd::AddHop(name, loc) => self.add_hop(loc, &name),
            args::Cmd::AddHopAndMove(name, loc) => {
                match self.add_hop(loc, &name) {
                    Ok(_) => self.groove(&name),
                    Err(e) => Err(e),
                }
            },
            args::Cmd::Move(name) => self.groove(&name),
            args::Cmd::PrintMsg(msg) => Ok(msg),
            args::Cmd::SetBrb(loc) => self.brb(loc),
            args::Cmd::BrbHop => self.hop("back"),
            args::Cmd::ListHops => self.list_hops(),
        };
        match output {
            Ok(statement) => if statement.len() > 0 { println!("{}", statement) },
            Err(e) => println!("[error] Unable to execute command: {}", e),
        };
    }
}
