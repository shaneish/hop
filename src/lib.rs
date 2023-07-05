pub mod args;
pub mod configs;
pub mod groups;
pub mod metadata;
use colored::Colorize;
use glob::glob;
use std::env::var;
use std::fs;
use std::path::{Path, PathBuf};

pub fn sanitize<T: AsRef<Path>>(p: T) -> anyhow::Result<String> {
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

pub struct Hopper {
    pub config: configs::Configs,
    pub db: sqlite::Connection,
    pub env: metadata::Environment,
}

impl Hopper {
    pub fn new() -> anyhow::Result<Self> {
        let env = metadata::Environment::new();
        let config = configs::Configs::new(&env.config_path);
        let conn = sqlite::open(&env.db_path)?;
        Ok(Hopper {
            config,
            db: conn,
            env,
        })
    }

    fn locate(&self) -> anyhow::Result<String> {
        sanitize(
            self.env
                .config_path
                .clone()
                .parent()
                .unwrap_or(&self.env.config_path),
        )
    }

    fn configure(&mut self) -> anyhow::Result<String> {
        let config_path = format!("{}/bhop.toml", self.locate()?);
        let move_dir = if self.config.always_jump {
            self.locate()?
        } else {
            ".".to_string()
        };
        let cmd = format!(
            "{}{}{}",
            move_dir,
            var("BHOP_CMD_SEPARATOR").unwrap_or("|".to_string()),
            self.map_editor(config_path, None)?
        );
        Ok(cmd)
    }

    fn passthrough(&self, cmd: String) -> anyhow::Result<String> {
        let bhop_exe = sanitize(std::env::current_exe()?)?;
        Ok(format!(
            ".{}{} {}",
            var("BHOP_CMD_SEPARATOR").unwrap_or("|".to_string()),
            bhop_exe,
            cmd
        ))
    }

    fn map_editor(&self, f: String, ext: Option<String>) -> anyhow::Result<String> {
        let editor = match ext {
            None => self.config.default_editor.to_string(),
            Some(ext) => match &self.config.editors.get(&ext) {
                Some(special_editor) => special_editor.to_string(),
                None => self.config.default_editor.to_string(),
            },
        };
        if editor.contains("{}") {
            Ok(editor.replace("{}", &f))
        } else {
            Ok(format!("{} {}", editor, f))
        }
    }

    fn add_shortcut<T: AsRef<Path>>(
        &mut self,
        path: T,
        name: Option<String>,
    ) -> anyhow::Result<()> {
        let name = match name {
            Some(n) => n,
            None => path
                .as_ref()
                .file_name()
                .ok_or(anyhow::anyhow!("Unable to extract file name for shortcut"))?
                .to_str()
                .ok_or(anyhow::anyhow!("Unable to extract file name for shortcut"))?
                .to_string(),
        };
        let path_as_string = sanitize(path)?;
        let query = format!(
            "INSERT OR REPLACE INTO shortcuts (name, location) VALUES (\"{}\", \"{}\")",
            name, &path_as_string
        );
        self.db.execute(query)?;
        Ok(())
    }

    fn remove_shortcut(&mut self, name: String) -> anyhow::Result<()> {
        let query = format!("DELETE FROM shortcuts WHERE name GLOB '{}'", name);
        self.db.execute(query)?;
        Ok(())
    }

    fn find_shortcut(&mut self, name: &str) -> Option<PathBuf> {
        let query = format!(
            "SELECT location FROM shortcuts WHERE name GLOB '{}' ORDER BY name",
            name
        );
        match self.db.prepare(query.as_str()) {
            Ok(mut statement) => {
                if let Ok(sqlite::State::Row) = statement.next() {
                    let location = statement.read::<String, _>("location");
                    match location {
                        Ok(location) => Some(PathBuf::from(location)),
                        Err(_) => None,
                    }
                } else {
                    None
                }
            }
            Err(_) => None,
        }
    }

    fn find_history(&mut self, name: &str) -> Option<(String, i64)> {
        let query = format!(
            "SELECT location, usage FROM history WHERE name GLOB '{}' ORDER BY usage DESC",
            name
        );
        match self.db.prepare(query.as_str()) {
            Ok(mut statement) => {
                if let Ok(sqlite::State::Row) = statement.next() {
                    let location = statement.read::<String, _>("location");
                    let usage = statement.read::<i64, _>("usage").unwrap_or(0);
                    match location {
                        Ok(location) => Some((location, usage)),
                        Err(_) => None,
                    }
                } else {
                    None
                }
            }
            Err(_) => None,
        }
    }

    fn add_history<T: AsRef<Path>>(&mut self, path: T) -> anyhow::Result<()> {
        let path = path.as_ref();
        let file_name = fs::canonicalize(path)?
            .file_name()
            .expect("Unable to extract file name for shortcut")
            .to_str()
            .expect("Unable to extract file name for shortcut")
            .to_string();
        let history = self.find_history(&file_name);
        let usage = match history {
            Some((_, usage)) => usage + 1,
            None => 1,
        };
        let path_as_string = sanitize(path)?;
        let query = format!(
            "INSERT OR REPLACE INTO history (name, location, usage) VALUES (\"{}\", \"{}\", {})",
            file_name, &path_as_string, usage
        );
        self.db.execute(query)?;
        Ok(())
    }

    fn find_local(&self, name: &str) -> Option<PathBuf> {
        match glob(name).expect("Failed to parse glob.").next() {
            Some(path) => path.ok(),
            None => None,
        }
    }

    fn grab(&mut self, name: String) -> Option<PathBuf> {
        let shortcut = self.find_shortcut(&name);
        let history = self
            .find_history(&name)
            .map(|(path, _)| PathBuf::from(path));
        let local = self.find_local(&name);

        let mut order = Vec::new();
        if self.config.prioritize_shortcuts {
            order.push(shortcut);
            order.push(local);
        } else {
            order.push(local);
            order.push(shortcut);
        }
        order.push(history);

        match order.into_iter().find(|x| x.is_some()) {
            Some(Some(path)) => Some(path),
            _ => None,
        }
    }

    fn bhop_it(&mut self, name: String, edit_dir: bool) -> anyhow::Result<String> {
        let path_opt = self.grab(name);
        match path_opt {
            Some(path) => {
                self.add_history(&path)?;
                if path.is_dir() && !edit_dir {
                    Ok(format!(
                        "{}{}",
                        sanitize(path)?,
                        var("BHOP_CMD_SEPARATOR").unwrap_or("|".to_string())
                    ))
                } else {
                    let sanitized = sanitize(&path)?;
                    let ext = path.extension().map(|s| s.to_str().unwrap().to_string());
                    let move_dir = if self.config.always_jump {
                        sanitize(path.parent().unwrap_or(&path))?
                    } else {
                        ".".to_string()
                    };
                    Ok(format!(
                        "{}{}{}",
                        move_dir,
                        var("BHOP_CMD_SEPARATOR").unwrap_or("|".to_string()),
                        self.map_editor(sanitized, ext)?
                    ))
                }
            }
            None => Err(anyhow::Error::msg("No matching options found.")),
        }
    }

    fn pull_maps(&self, query: &str) -> anyhow::Result<()> {
        let mut statement = self.db.prepare(query)?;
        let mut results = Vec::new();
        while let Ok(sqlite::State::Row) = statement.next() {
            let name = statement.read::<String, _>("name");
            let location = statement.read::<String, _>("location");
            if let Ok(n) = name {
                if let Ok(l) = location {
                    results.push([n, l])
                }
            }
        }
        self.format_map(results);
        Ok(())
    }

    fn search_shortcuts(&self, filter: &Option<String>) -> anyhow::Result<()> {
        let filter = match filter {
            Some(f) => format!(
                "{}{}{}",
                self.config.search_match_prefix, f, self.config.search_match_suffix
            ),
            None => "*".to_string(),
        };
        let query = format!(
            "SELECT name, location FROM shortcuts WHERE name GLOB '{}' OR location GLOB '{}' ORDER BY name",
            &filter,
            &filter,
        );
        self.pull_maps(&query)?;
        Ok(())
    }

    fn search_history(&self, filter: &Option<String>) -> anyhow::Result<()> {
        let filter = match filter {
            Some(f) => format!(
                "{}{}{}",
                self.config.search_match_prefix, f, self.config.search_match_suffix
            ),
            None => "*".to_string(),
        };
        let query = format!(
            r#"SELECT h1.name, h1.location FROM history AS h1 
                INNER JOIN 
                    (SELECT name, MIN(usage) AS min_usage, MIN(location) AS min_loc FROM history 
                     WHERE name GLOB '{}' OR location GLOB '{}' GROUP BY name) AS h2 
                ON (h1.name = h2.name AND h1.usage = h2.min_usage AND h1.location = h2.min_loc) 
                GROUP BY h1.name, h1.location 
                ORDER BY h1.name"#,
            &filter, &filter
        );
        self.pull_maps(&query)?;
        Ok(())
    }

    fn search(&self, filter: Option<String>) -> anyhow::Result<()> {
        println!("{}", "Shortcut:".bright_white().bold());
        self.search_shortcuts(&filter)?;
        println!("{}", "History:".bright_white().bold());
        self.search_history(&filter)?;
        Ok(())
    }

    fn format_map(&self, hops: Vec<[String; 2]>) {
        let max_name_size = hops.iter().map(|[name, _]| name.len()).max().unwrap_or(0);
        let first_col = self.config.print_color_primary;
        let sec_col = self.config.print_color_secondary;
        let formatted_hops: Vec<String> = hops
            .into_iter()
            .map(|[name, location]| {
                (
                    String::from_utf8(vec![b' '; max_name_size - name.len() + 1])
                        .unwrap_or(" ".to_string()),
                    name,
                    location,
                )
            })
            .map(|(ws, name, location)| {
                format!(
                    "{}{}{} {}",
                    name.truecolor(first_col[0], first_col[1], first_col[2])
                        .bold(),
                    ws,
                    "->".bright_white().bold(),
                    &location
                        .truecolor(sec_col[0], sec_col[1], sec_col[2])
                        .bold(),
                )
            })
            .collect();
        println!("{}", formatted_hops.join("\n"));
    }

    fn use_group(&mut self, group: String, subgroup: Option<String>) -> anyhow::Result<String> {
        let subgroup = subgroup.unwrap_or("default".to_string());
        let path = self.grab(group.clone()).unwrap_or(PathBuf::from("."));
        let group_path = path.join(var("BHOP_PROJECT_CONFIGS").unwrap_or(".bhop".to_string()));
        match groups::BhopGroup::from(&subgroup, group_path) {
            Some(options) => match options.cmd {
                Some(cmd) => Ok(format!(
                    "{}{}{}",
                    sanitize(&path)?,
                    var("BHOP_CMD_SEPARATOR").unwrap_or("|".to_string()),
                    cmd
                )),
                None => match options.files {
                    Some(files) => {
                        let mut files = files.into_iter();
                        match files.next() {
                            Some(first) => {
                                // the method to find the extension below is a bit hacky, but
                                // works.  Originally I was going to bring in the external glob
                                // crate, expand the glob, and then find the extension of the
                                // first file, but didn't want to add another dependency just for
                                // this.
                                let ext = PathBuf::from(first.replace('*', "x"))
                                    .extension()
                                    .map(|s| s.to_str().unwrap().to_string());
                                let editor_cmd =
                                    options.editor.unwrap_or(self.map_editor(first, ext)?);
                                let rest = files.collect::<Vec<String>>().join(" ");
                                let rest = if rest.is_empty() {
                                    "".to_string()
                                } else {
                                    format!(" {}", rest)
                                };
                                Ok(format!(
                                    "{}{}{}{}",
                                    sanitize(&path)?,
                                    var("BHOP_CMD_SEPARATOR").unwrap_or("|".to_string()),
                                    editor_cmd,
                                    rest
                                ))
                            }
                            None => self.bhop_it(group, true),
                        }
                    }
                    None => Err(anyhow::Error::msg("No matching options found.")),
                },
            },
            None => Ok("Unable to find group or subgroup.".to_string()),
        }
    }
}
