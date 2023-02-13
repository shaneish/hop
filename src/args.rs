// Enum used to parse input arguments.  Ended up rolling my own arg parser instead of using an
// existing crate because I wanted `hp` commands to be more natural language-like and use dynamic
// positional commands

use colored::Colorize;
use std::{
    env,
    path::{Path, PathBuf},
};

#[derive(Debug)]
pub enum Rabbit {
    Dir(String, PathBuf),
    File(String, PathBuf),
    RequestName(String),
    RequestPath(PathBuf),
}

impl Rabbit {
    pub fn from<T: AsRef<Path>>(input: T, name: Option<String>) -> Self {
        let current_name = match name {
            Some(given_name) => given_name,
            None => input
                .as_ref()
                .file_name()
                .expect("[error] Unable to disambiguate file/directory.")
                .to_str()
                .expect("[error] Unable to convert file/directory name to UTF-8.")
                .to_string(),
        };
        if input.as_ref().is_dir() {
            Rabbit::Dir(current_name, input.as_ref().to_path_buf())
        } else {
            Rabbit::File(current_name, input.as_ref().to_path_buf())
        }
    }
}

pub enum Cmd {
    Use(Rabbit),
    Remove(Rabbit),
    PrintMsg(String),
    SetBrb(PathBuf),
    BrbHop,
    ListHops,
    PrintHelp,
    Passthrough(String),
    LocateBunnyhop,
    LocateShortcut(String),
    Configure,
    HopDirAndEdit(String),
    EditDir(Rabbit),
    ShowHistory,
}

impl Cmd {
    pub fn parse() -> Self {
        let current_dir =
            env::current_dir().expect("[error] Unable to locate current working directory.");
        match env::args().nth(1) {
            Some(primary) => match primary.as_str() {
                "add" => match env::args().nth(2) {
                    Some(f_or_d) => {
                        let mut f_or_d_path = PathBuf::from(&current_dir);
                        f_or_d_path.push(&f_or_d);
                        if f_or_d_path.is_file() {
                            Cmd::Use(Rabbit::from(f_or_d_path, env::args().nth(3)))
                        } else {
                            Cmd::Use(Rabbit::from(&current_dir, Some(f_or_d)))
                        }
                    }
                    None => Cmd::Use(Rabbit::from(
                        env::current_dir()
                            .expect("[error] Unable to locate current working directory."),
                        None,
                    )),
                },
                "rm" | "remove" => match env::args().nth(2) {
                    Some(name) => Cmd::Remove(Rabbit::RequestName(name)),
                    None => Cmd::Remove(Rabbit::RequestPath(current_dir.to_path_buf())),
                },
                "ls" | "list" => Cmd::Passthrough("_ls".to_string()),
                "_ls" => Cmd::ListHops,
                "version" | "v" => Cmd::Passthrough("_version".to_string()),
                "_version" => Cmd::PrintMsg(format!(
                    "{} 🐇 {}{}",
                    "bunnyhop".cyan().bold(),
                    "v.".bold(),
                    env!("CARGO_PKG_VERSION").bright_white().bold()
                )),
                "brb" => Cmd::SetBrb(current_dir),
                "back" => Cmd::BrbHop,
                "help" => Cmd::Passthrough("_help".to_string()),
                "_help" => Cmd::PrintHelp,
                "config" | "configure" => Cmd::Configure,
                "edit" => match env::args().nth(2) {
                    Some(name) => Cmd::HopDirAndEdit(name),
                    None => Cmd::EditDir(Rabbit::from(current_dir, None)),
                },
                "locate" => match env::args().nth(2) {
                    Some(name) => Cmd::LocateShortcut(name),
                    None => Cmd::Passthrough("_locate_bunnyhop".to_string()),
                },
                "_locate_bunnyhop" => Cmd::LocateBunnyhop,
                "history" => Cmd::Passthrough("_history".to_string()),
                "_history" => Cmd::ShowHistory,
                whatevs => Cmd::Use(Rabbit::RequestName(whatevs.to_string())),
            },
            None => Cmd::PrintMsg("[error] Unable to parse current arguments.".to_string()),
        }
    }
}
