pub mod args;
use args::Rabbit;
use chrono::Local;
use colored::Colorize;
use dirs::home_dir;
use proceed::any_or_quit_with;
use serde_derive::Deserialize;
use std::{
    collections::HashMap,
    env::{consts, current_exe, var},
    fs, include_str,
    io::Write,
    path::{Path, PathBuf},
};
use toml::from_str;

#[derive(Deserialize, PartialEq, Debug)]
pub struct Config {
    pub settings: Settings,
    pub editors: Option<HashMap<String, String>>,
}

#[derive(Deserialize, PartialEq, Debug)]
pub struct Settings {
    pub default_editor: String,
    pub max_history: usize,
    pub ls_display_block: usize,
    pub print_color_primary: Option<[u8; 3]>,
    pub print_color_secondary: Option<[u8; 3]>,
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
                home_dir.push("bunnyhop");
                home_dir
            }
        };
        let mut config_file = PathBuf::from(&config_dir);
        match var("config_file_NAME") {
            Ok(name) => config_file.push(name),
            Err(_) => config_file.push("bunnyhop.toml"),
        };
        let mut database_file = match var("HOP_DATABASE_DIRECTORY") {
            Ok(loc) => PathBuf::from(&loc),
            Err(_) => {
                let mut db_dir_temp = PathBuf::from(format!("{}", &config_dir.as_path().display()));
                db_dir_temp.push("db");
                db_dir_temp
            }
        };
        if !Path::new(&database_file).exists() {
            match fs::create_dir_all(&database_file) {
                Ok(_) => {}
                Err(e) => println!("[error] Error creating database directory: {}", e),
            };
        };
        match var("HOP_DATABASE_FILE_NAME") {
            Ok(name) => database_file.push(name),
            Err(_) => database_file.push("bunnyhop.db"),
        };

        Env {
            config_file,
            database_file,
        }
    }
}

pub struct Hopper {
    pub config: Config,
    pub env: Env,
    pub db: sqlite::Connection,
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
            let mut new_conf =
                fs::File::create(&env.config_file).expect("[error] Unable to create config file.");
            let default_configs: &str = match consts::OS {
                "windows" => include_str!("defaults/windows_defaults.toml"),
                _ => include_str!("defaults/unix_defaults.toml"),
            };
            new_conf
                .write_all(default_configs.as_bytes())
                .expect("[error] Unable to generate default config file.");
        };
        let toml_str: String = fs::read_to_string(env.config_file.clone())
            .expect("[error] Unable to read config file location.");
        let configs: Config =
            from_str(&toml_str).expect("[error] Unable to parse configuration TOML.");
        let conn = sqlite::open(&env.database_file)?;
        conn.execute(
            "CREATE TABLE IF NOT EXISTS named_hops (
            name TEXT PRIMARY KEY,
            location TEXT NOT NULL
            )",
        )?;
        conn.execute(
            "CREATE TABLE IF NOT EXISTS history (
            time TEXT,
            name TEXT NOT NULL,
            location TEXT NOT NULL
            )",
        )?;
        Ok(Hopper {
            config: configs,
            env,
            db: conn,
        })
    }

    fn add_hop<T: AsRef<Path>>(&mut self, path: T, name: &str) -> anyhow::Result<()> {
        let path_as_string = Self::sanitize(path.as_ref())?;
        let query = format!(
            "INSERT OR REPLACE INTO named_hops (name, location) VALUES (\"{}\", \"{}\")",
            name, &path_as_string
        );
        self.db.execute(query)?;
        println!("[info] Added shortcut: {} -> {}", name, path_as_string);
        Ok(())
    }

    fn remove_hop(&mut self, rabbit: Rabbit) -> anyhow::Result<()> {
        let mut is_passthrough = false;
        let statement_check = match rabbit {
            Rabbit::RequestName(name) => Some((
                self.db
                    .execute(format!("DELETE FROM named_hops WHERE name=\"{}\"", &name)),
                name,
            )),
            Rabbit::RequestPath(loc) => Some((
                self.db.execute(format!(
                    "DELETE FROM named_hops WHERE location=\"{}\"",
                    Self::sanitize(loc.as_path())?
                )),
                loc.as_path().display().to_string(),
            )),
            Rabbit::RequestAmbiguous(name, loc) => {
                is_passthrough = true;
                match self.find_hop(name.clone()) {
                    Some(_) => {
                        self.remove_hop(Rabbit::RequestName(name))?;
                        None
                    }
                    None => {
                        self.remove_hop(Rabbit::RequestPath(loc))?;
                        None
                    }
                }
            }
            _ => None,
        };
        if !is_passthrough {
            match statement_check {
                Some((statement, name)) => match statement {
                    Ok(_) => println!("[info] Removed shortcut: {}", name),
                    Err(e) => println!(
                        "[error] Failed to remove shortcut: {} with error {}",
                        name, e
                    ),
                },
                None => println!("[error] Unable to find shortcut to remove."),
            };
        };
        Ok(())
    }

    fn map_editor<T: AsRef<Path>>(&self, f: T) -> String {
        let ext_option = f.as_ref().extension();
        match &self.config.editors {
            Some(editor_map) => match ext_option {
                Some(ext) => match editor_map.get(
                    &(ext
                        .to_str()
                        .expect("[error] Cannot extract extension.")
                        .to_string()),
                ) {
                    Some(special_editor) => special_editor.to_string(),
                    None => self.config.settings.default_editor.to_string(),
                },
                None => self.config.settings.default_editor.to_string(),
            },
            None => self.config.settings.default_editor.to_string(),
        }
    }

    fn format_editor<T: AsRef<str>>(&self, editor: T, path: T) {
        if editor.as_ref().contains("{}") {
            let imputed = editor.as_ref().replace("{}", path.as_ref());
            println!("__cmd__ {}", imputed);
        } else {
            println!("__cmd__ {} {}", editor.as_ref(), path.as_ref());
        }
    }

    fn print_hop(&self, shortcut_name: String) -> anyhow::Result<()> {
        match self.find_hop(shortcut_name) {
            Some(name) => println!("{}", name),
            None => println!("[error] Unable to find shortcut."),
        }
        Ok(())
    }

    fn find_hop(&self, shortcut_name: String) -> Option<String> {
        let query = format!(
            "SELECT location FROM named_hops WHERE name=\"{}\"",
            &shortcut_name
        );
        let statement_result = self.db.prepare(query);
        match statement_result {
            Ok(mut statement) => {
                if let Ok(sqlite::State::Row) = statement.next() {
                    let location_result = statement.read::<String, _>("location");
                    match location_result {
                        Ok(location) => Some(location),
                        Err(_) => None,
                    }
                } else {
                    None
                }
            }
            Err(_) => None,
        }
    }

    fn output_ambiguous<T: AsRef<Path>>(&self, location: T) {
        let location_path = location.as_ref();
        let location_string =
            Self::sanitize(location.as_ref()).unwrap_or(location.as_ref().display().to_string());
        if location_path.is_file() {
            let editor = self.map_editor(&location);
            self.format_editor(editor, location_string);
        } else if location_path.is_dir() {
            println!("__cd__ {}", location_string);
        };
    }

    fn use_hop(&mut self, shortcut_name: String) -> anyhow::Result<()> {
        let query = format!(
            "SELECT location FROM named_hops WHERE name=\"{}\"",
            &shortcut_name
        );
        let mut statement = self.db.prepare(query)?;
        if let Ok(sqlite::State::Row) = statement.next() {
            let location = statement.read::<String, _>("location")?;
            let location_path = PathBuf::from(&location);
            self.output_ambiguous(location_path);
            return Ok(());
        }

        match self.check_dir(&shortcut_name) {
            Some((dir, short)) => {
                self.log_history(&dir, short)?;
                self.output_ambiguous(dir);
                Ok(())
            }
            None => {
                let history = self.retrieve_history()?;
                match history.iter().find(|(n, _)| n == &shortcut_name) {
                    Some((short, dir)) => {
                        self.log_history(dir, short.to_string())?;
                        self.output_ambiguous(dir);
                        Ok(())
                    }
                    None => Err(anyhow::anyhow!("Unable to find referenced shortcut.")),
                }
            }
        }
    }

    fn edit_dir(&mut self, bunny: Rabbit) -> anyhow::Result<()> {
        if let Rabbit::Dir(hop_name, hop_path) = bunny {
            self.log_history(hop_path, hop_name)?;
        };
        println!("__cmd__ {}", self.config.settings.default_editor);
        Ok(())
    }

    fn just_do_it(&mut self, bunny: Rabbit) -> anyhow::Result<()> {
        match bunny {
            Rabbit::File(hop_name, hop_path) => self.add_hop(hop_path, &hop_name),
            Rabbit::Dir(hop_name, hop_path) => self.add_hop(hop_path, &hop_name),
            Rabbit::RequestName(shortcut_name) => self.use_hop(shortcut_name),
            _ => Ok(()),
        }
    }

    fn sanitize<T: AsRef<Path>>(p: T) -> anyhow::Result<String> {
        // Back slashes in Windows paths create so many headaches.  Since Windows accepts forward
        // slashes in place of back slashes anyways, this will ensure that all paths are absolute
        // with consistent forward slashes
        let location = if p.as_ref().is_absolute() {
            p.as_ref().display().to_string()
        } else {
            fs::canonicalize(p.as_ref())?.display().to_string()
        };
        Ok(location.replace('\\', "/").replace("//?/", ""))
    }

    fn log_history<T: AsRef<Path>>(&self, loc: T, name: String) -> anyhow::Result<()> {
        let location = Self::sanitize(loc.as_ref())?;
        if self.config.settings.max_history > 0 {
            let query = format!(
                "INSERT INTO history (time, name, location) VALUES ({}, \"{}\", \"{}\") ",
                Local::now().format("%Y%m%d%H%M%S"),
                name,
                location
            );
            self.db.execute(query)?;
            let mut count_result = self
                .db
                .prepare("SELECT COUNT(*) AS hist_count, MIN(time) AS hist_min FROM history")?;
            if let Ok(sqlite::State::Row) = count_result.next() {
                let history_count = count_result.read::<i64, _>("hist_count")?;
                let history_min = count_result.read::<String, _>("hist_min")?;
                if history_count > self.config.settings.max_history as i64 {
                    self.db.execute(format!(
                        "DELETE FROM history WHERE time=\"{}\"",
                        history_min
                    ))?;
                };
            };
        };
        Ok(())
    }

    fn check_dir(&self, name: &str) -> Option<(PathBuf, String)> {
        let potential_path = PathBuf::from(&name);
        if potential_path.exists() {
            let shortcut_name = match &potential_path.file_name() {
                Some(n) => match n.to_str() {
                    Some(m) => m.to_string(),
                    None => name.to_string(),
                },
                None => name.to_string(),
            };
            Some((potential_path, shortcut_name))
        } else {
            None
        }
    }

    fn search_all(&self, filter_condition: Option<String>) -> anyhow::Result<()> {
        println!("{}", "Saved Hops:".bold());
        self.list_hops(filter_condition.clone())?;
        println!("\n{}", "Historical Hops:".bold());
        self.show_history(filter_condition)?;
        Ok(())
    }

    fn print_formatted_maps(&self, hops: Vec<(String, String)>, filter_string: Option<String>) {
        let filter_condition = filter_string.unwrap_or("".to_string());
        let filtered_hops: Vec<(String, String, String)> = hops
            .into_iter()
            .map(|(n, l)| (n, if PathBuf::from(&l).is_file() { "file".to_string() } else { "dir".to_string() }, l))
            .filter(|(n, t, l)| n.contains(&filter_condition) || l.contains(&filter_condition) || t.contains(&filter_condition))
            .collect();
        let max_name_size = filtered_hops
            .iter()
            .map(|(name, _, _)| name.len())
            .max()
            .unwrap_or(0);
        let first_col = self.config.settings.print_color_primary.unwrap_or([51, 255, 255]);
        let sec_col = self.config.settings.print_color_secondary.unwrap_or([51, 255, 153]);
        let mut formatted_hops: Vec<String> = filtered_hops
            .into_iter()
            .map(|(name, type_loc, location)| {
                (
                    String::from_utf8(vec![b' '; max_name_size - name.len() + 1])
                        .unwrap_or(" ".to_string()),
                    name,
                    location,
                    type_loc,
                )
            })
            .map(|(ws, name, location, type_loc)| {
                format!(
                    "{}{}{} {} [{}]",
                    name.truecolor(first_col[0], first_col[1], first_col[2])
                        .bold(),
                    ws,
                    "->".bright_white().bold(),
                    &location
                        .truecolor(sec_col[0], sec_col[1], sec_col[2])
                        .bold(),
                    type_loc.bold(),
                )
            })
            .collect();
        formatted_hops.sort();
        for (idx, hop) in formatted_hops.into_iter().enumerate() {
            println!("{}", hop);
            if (self.config.settings.ls_display_block != 0)
                && ((idx + 1) % self.config.settings.ls_display_block == 0)
            {
                println!("{}", "Press 'Enter' to continue or 'q' to quit...".dimmed());
                if !any_or_quit_with('q') {
                    return;
                }
            }
        }
    }

    fn list_hops(&self, filter_string: Option<String>) -> anyhow::Result<()> {
        let query = "SELECT name, location FROM named_hops";
        let mut query_result = self.db.prepare(query)?;
        let mut hops: Vec<(String, String)> = Vec::new();
        while let Ok(sqlite::State::Row) = query_result.next() {
            let name = query_result.read::<String, _>("name")?;
            let location = query_result.read::<String, _>("location")?;
            hops.push((name, location));
        }
        self.print_formatted_maps(hops, filter_string);
        Ok(())
    }

    fn brb<T: AsRef<Path>>(&mut self, path: T) -> anyhow::Result<()> {
        self.add_hop(path.as_ref(), "back")?;
        Ok(())
    }

    fn print_help() -> anyhow::Result<()> {
        println!(
            include!("defaults/help.txt"),
            "hp".bold(),
            "arg1".bright_red(),
            "arg2".bright_red(),
            "add".cyan().bold(),
            "ls".cyan().bold(),
            "list".cyan().bold(),
            "v".cyan().bold(),
            "version".cyan().bold(),
            "brb".cyan().bold(),
            "hp".bold(),
            "back".bright_red(),
            "rm".cyan().bold(),
            "remove".cyan().bold(),
            "arg2".bright_red(),
            "edit".cyan().bold(),
            "configure".cyan().bold(),
            "config".cyan().bold(),
            "locate".cyan().bold(),
            "history".cyan().bold(),
            "hist".cyan().bold(),
            "search".cyan().bold(),
            "arg2".bright_red(),
            "...".cyan().bold(),
            "hp".bold()
        );
        Ok(())
    }

    fn runner(&self, cmd: String) -> anyhow::Result<()> {
        let bhop_exe = current_exe()
            .expect("[error] Unable to extract current bunnyhop executable name.")
            .into_os_string()
            .to_str()
            .expect("[error] Unable to convert current bunnyhop executable path to UTF-8.")
            .to_string()
            .replace('\\', "/");
        self.format_editor(bhop_exe, cmd);
        Ok(())
    }

    fn configure(&self) -> anyhow::Result<()> {
        let editor = self.map_editor(&self.env.config_file);
        self.format_editor(editor, self.env.config_file.display().to_string());
        Ok(())
    }

    fn hop_to_and_open_dir(&mut self, shortcut_name: String) -> anyhow::Result<()> {
        let hop_loc_string = self.find_hop(shortcut_name.clone());
        match hop_loc_string {
            Some(loc) => {
                let hop_loc = PathBuf::from(&loc);
                if hop_loc.is_dir() {
                    self.use_hop(shortcut_name)?;
                    println!("__cmd__ {}", self.config.settings.default_editor);
                } else if hop_loc.is_file() {
                    self.format_editor(
                        self.map_editor(&hop_loc),
                        hop_loc.as_path().display().to_string()
                    );
                }
            }
            None => {
                match self.check_dir(&shortcut_name) {
                    Some((dir, short)) => {
                        self.log_history(&dir, short)?;
                        if dir.is_file() {
                            let editor = self.map_editor(&dir);
                            self.format_editor(editor, dir.as_path().display().to_string());
                        };
                    }
                    None => {
                        println!("[error] Unable to find referenced file or directory.");
                    }
                };
            }
        };
        Ok(())
    }

    fn show_history(&self, filter_condition: Option<String>) -> anyhow::Result<()> {
        let hops = self.retrieve_history()?;
        self.print_formatted_maps(hops, filter_condition);
        Ok(())
    }

    fn retrieve_history(&self) -> anyhow::Result<Vec<(String, String)>> {
        let query = "SELECT name, location, COUNT(location) AS cntl
            FROM history GROUP BY name, location
            ORDER by cntl DESC";
        let mut query_result = self.db.prepare(query)?;
        let mut hops: Vec<(String, String)> = Vec::new();
        let mut names: Vec<String> = Vec::new();
        while let Ok(sqlite::State::Row) = query_result.next() {
            let name = query_result.read::<String, _>("name")?;
            let location = query_result.read::<String, _>("location")?;
            if !names.contains(&name) {
                names.push(name.clone());
                hops.push((name, location));
            }
        }
        Ok(hops)
    }

    fn search_history(&self, bunny: Rabbit) -> anyhow::Result<()> {
        let history = self.retrieve_history()?;
        match bunny {
            Rabbit::RequestName(name) => {
                let associated_pair = history.into_iter().find(|(n, _)| n == &name);
                match associated_pair {
                    Some((_, associated_path)) => {
                        println!("{}", associated_path);
                        Ok(())
                    }
                    None => {
                        println!(
                            "[error] Unable to find matching reference in current history mapping."
                        );
                        Ok(())
                    }
                }
            }
            Rabbit::RequestPath(loc) => {
                let associated_pair = history.into_iter().find(|(_, p)| loc == PathBuf::from(&p));
                match associated_pair {
                    Some((associated_name, _)) => {
                        println!("{}", associated_name);
                        Ok(())
                    }
                    None => self
                        .search_history(Rabbit::RequestName(loc.as_path().display().to_string())),
                }
            }
            _ => {
                println!("[error] Unable to find matching reference in current history mapping.");
                Ok(())
            }
        }
    }

    fn show_locations(&self) -> anyhow::Result<()> {
        let loc_vec = vec![
            (
                "Config Directory".to_string(),
                self.env
                    .config_file
                    .parent()
                    .expect("[error] Unable to locate current config directory.")
                    .display()
                    .to_string(),
            ),
            (
                "Database Directory".to_string(),
                self.env
                    .database_file
                    .parent()
                    .expect("[error] Unable to locate current database directory.")
                    .display()
                    .to_string(),
            ),
            (
                "Bunnyhop Executable".to_string(),
                current_exe()
                    .expect("[error] Unable to locate current executable file.")
                    .display()
                    .to_string(),
            ),
        ];
        self.print_formatted_maps(loc_vec, None);
        Ok(())
    }
}

impl Default for Hopper {
    fn default() -> Self {
        Self::new().expect("[error] Unable to create a hopper.")
    }
}
