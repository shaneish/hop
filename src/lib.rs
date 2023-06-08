pub mod args;
pub mod configs;
use args::Rabbit;
use colored::Colorize;

pub struct Hopper {
    pub config: configs::Config,
    pub db: sqlite::Connection,
}

impl Hopper {
    pub fn new() -> anyhow::Result<Self> {
        let env = configs::Environment::new();
        let config = configs::Config::new(&env.config_file)?;
        let conn = sqlite::open(&env.database_file)?;
        Ok(Hopper {
            config: configs,
            db: conn,
        })
    }

    fn info(&self, msg: &str) {
        if self.config.settings.verbose.unwrap_or(false) {
            println!("[info] {}", msg);
        }
    }

    fn add_hop<T: AsRef<Path>>(&mut self, path: T, name: &str) -> anyhow::Result<()> {
        let path_as_string = Self::sanitize(path.as_ref())?;
        let query = format!(
            "INSERT OR REPLACE INTO named_hops (name, location) VALUES (\"{}\", \"{}\")",
            name, &path_as_string
        );
        self.db.execute(query)?;
        self.info(format!("Added shortcut: {} -> {}", name, path_as_string).as_str());
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
                    Ok(_) => self.info(format!("Removed shortcut: {}", name).as_str()),
                    Err(e) => print!(
                        "[error] Failed to remove shortcut: {} with error {}",
                        name, e
                    ),
                },
                None => print!("[error] Unable to find shortcut to remove."),
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

    fn format_editor<T: AsRef<str>>(&self, editor: T, path: T, move_to: Option<T>) {
        if let Some(m) = move_to {
            let loc = if !m.as_ref().is_empty() {
                m.as_ref()
            } else {
                "."
            };
            print!("__cd__ {}", loc)
        };
        if editor.as_ref().contains("{}") {
            let imputed = editor.as_ref().replace("{}", path.as_ref());
            print!("__cmd__ {}", imputed);
        } else {
            print!("__cmd__ {} {}", editor.as_ref(), path.as_ref());
        }
    }

    fn print_hop(&self, shortcut_name: String) -> anyhow::Result<()> {
        match self.find_hop(shortcut_name) {
            Some(name) => print!("{}", name),
            None => print!("[error] Unable to find shortcut."),
        }
        Ok(())
    }

    fn grab(&self, shortcut_name: String) -> anyhow::Result<()> {
        match self.find_hop(shortcut_name.clone()) {
            Some(name) => {
                print!("{}", name);
                Ok(())
            }
            None => {
                let history = self.retrieve_history()?;
                let associated_pair = history.into_iter().find(|(n, _)| n == &shortcut_name);
                match associated_pair {
                    Some((_, associated_path)) => {
                        print!("{}", associated_path);
                        Ok(())
                    }
                    None => {
                        print!("[error] Unable to find matching reference.");
                        Ok(())
                    }
                }
            }
        }
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

    fn find_any(&self, shortcut_name: String) -> Option<String> {
        match self.find_hop(shortcut_name.clone()) {
            Some(name) => Some(name),
            None => {
                let history = self.retrieve_history().unwrap_or_default();
                let associated_pair = history.into_iter().find(|(n, _)| n == &shortcut_name);
                associated_pair.map(|(_, associated_path)| associated_path)
            }
        }
    }

    fn output_ambiguous<T: AsRef<Path>>(&self, location: T) {
        let location_path = location.as_ref();
        let location_string =
            Self::sanitize(location.as_ref()).unwrap_or(location.as_ref().display().to_string());
        if location_path.is_file() {
            let editor = self.map_editor(&location);
            let dir = location_path
                .parent()
                .unwrap_or(Path::new("."))
                .display()
                .to_string();
            self.format_editor(editor, location_string, Some(dir));
        } else if location_path.is_dir() {
            print!("__cd__ {}", location_string);
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
            self.log_history(&location_path, shortcut_name)?;
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
        print!("__cmd__ {}", self.config.settings.default_editor);
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

    fn add_and_just_do_it(&mut self, bunny: Rabbit) -> anyhow::Result<()> {
        match bunny {
            Rabbit::File(hop_name, hop_path) | Rabbit::Dir(hop_name, hop_path) => {
                self.add_hop(hop_path, &hop_name)?;
                self.use_hop(hop_name)
            }
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
        let mut count_result = self
            .db
            .prepare("SELECT COUNT(*) AS hist_count FROM history")?;
        if let Ok(sqlite::State::Row) = count_result.next() {
            let count = count_result.read::<i64, _>("hist_count")?;
            if (count >= self.config.settings.max_history as i64)
                || (self.config.settings.max_history as i64 == 0)
            {
                let retrieve_query = format!(
                    "SELECT location, name, usage FROM history WHERE name=\"{}\" AND location=\"{}\"",
                    name,
                    location,
                );
                let mut retrieve_result = self.db.prepare(retrieve_query)?;
                if let Ok(sqlite::State::Row) = retrieve_result.next() {
                    let usage = retrieve_result.read::<i64, _>("usage")?;
                    let update_query = format!(
                        "UPDATE history SET usage={} WHERE name=\"{}\" AND location=\"{}\"",
                        usage + 1,
                        name,
                        location
                    );
                    self.db.execute(update_query)?;
                    return Ok(());
                } else {
                    let insert_query = format!(
                        "INSERT INTO history (name, location, usage) VALUES (\"{}\", \"{}\", 1)",
                        name, location
                    );
                    self.db.execute(insert_query)?;
                };
            }
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
            .map(|(n, l)| {
                (
                    n,
                    if PathBuf::from(&l).is_file() {
                        "file".to_string()
                    } else {
                        "dir".to_string()
                    },
                    l,
                )
            })
            .filter(|(n, t, l)| {
                n.contains(&filter_condition)
                    || l.contains(&filter_condition)
                    || t.contains(&filter_condition)
            })
            .collect();
        let max_name_size = filtered_hops
            .iter()
            .map(|(name, _, _)| name.len())
            .max()
            .unwrap_or(0);
        let first_col = self
            .config
            .settings
            .print_color_primary
            .unwrap_or([51, 255, 255]);
        let sec_col = self
            .config
            .settings
            .print_color_secondary
            .unwrap_or([51, 255, 153]);
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
        self.format_editor(bhop_exe, cmd, None);
        Ok(())
    }

    fn configure(&self) -> anyhow::Result<()> {
        let editor = self.map_editor(&self.env.config_file);
        self.format_editor(editor, self.env.config_file.display().to_string(), None);
        Ok(())
    }

    fn hop_to_and_open_dir(&mut self, shortcut_name: String) -> anyhow::Result<()> {
        let hop_loc_string = self.find_any(shortcut_name.clone());
        match hop_loc_string {
            Some(loc) => {
                let hop_loc = PathBuf::from(&loc);
                if hop_loc.is_dir() {
                    self.use_hop(shortcut_name)?;
                    print!("__cmd__ {}", self.config.settings.default_editor);
                } else if hop_loc.is_file() {
                    self.log_history(&hop_loc, shortcut_name)?;
                    let dir = hop_loc
                        .parent()
                        .unwrap_or(Path::new("."))
                        .display()
                        .to_string();
                    self.format_editor(
                        self.map_editor(&hop_loc),
                        hop_loc.as_path().display().to_string(),
                        Some(dir),
                    );
                }
            }
            None => {
                match self.check_dir(&shortcut_name) {
                    Some((dir, short)) => {
                        self.log_history(&dir, short)?;
                        if dir.is_file() {
                            let editor = self.map_editor(&dir);
                            let file_dir =
                                dir.parent().unwrap_or(Path::new(".")).display().to_string();
                            self.format_editor(
                                editor,
                                dir.as_path().display().to_string(),
                                Some(file_dir),
                            );
                        };
                    }
                    None => {
                        print!("[error] Unable to find referenced file or directory.");
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
        let query = "SELECT name, location
            FROM history
            ORDER by usage DESC";
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
                        print!("{}", associated_path);
                        Ok(())
                    }
                    None => {
                        print!(
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
                        print!("{}", associated_name);
                        Ok(())
                    }
                    None => self
                        .search_history(Rabbit::RequestName(loc.as_path().display().to_string())),
                }
            }
            _ => {
                print!("[error] Unable to find matching reference in current history mapping.");
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
