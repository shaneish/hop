// Enum used to parse input arguments.  Ended up rolling my own arg parser instead of using an
// existing crate because I wanted `hp` commands to be more natural language-like and use dynamic
// positional commands

use crate::Hopper;
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
    RequestAmbiguous(String, PathBuf),
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

    pub fn request(input: String) -> Self {
        let input_path = PathBuf::from(&input);
        if input_path.exists()
            && (input == input.replace('/', ""))
            && (input == input.replace('\\', ""))
        {
            Rabbit::RequestAmbiguous(input, input_path)
        } else if input_path.exists() {
            Rabbit::RequestPath(PathBuf::from(&input))
        } else {
            Rabbit::RequestName(input)
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
    PullHistory(Rabbit),
    Search(Option<String>),
    Grab(String),
}

impl Cmd {
    pub fn parse() -> Self {
        let current_dir =
            env::current_dir().expect("[error] Unable to locate current working directory.");
        match env::args().nth(1) {
            Some(primary) => match primary.as_str() {
                "a" | "ad" | "add" => match env::args().nth(2) {
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
                "r" | "rm" | "remove" => match env::args().nth(2) {
                    Some(name) => Cmd::Remove(Rabbit::request(name)),
                    None => Cmd::Remove(Rabbit::request(current_dir.display().to_string())),
                },
                "l" | "ls" | "list" => match env::args().nth(2) {
                    Some(name) => Cmd::LocateShortcut(name),
                    None => Cmd::Passthrough("_ls".to_string()),
                },
                "_ls" => Cmd::ListHops,
                "v" | "vs" | "version" => Cmd::Passthrough("_version".to_string()),
                "_version" => Cmd::PrintMsg(format!(
                    "{} 🐇 {}{}",
                    "BunnyHop".cyan().bold(),
                    "v.".bold(),
                    env!("CARGO_PKG_VERSION").bright_white().bold()
                )),
                "brb" => Cmd::SetBrb(current_dir),
                "back" => Cmd::BrbHop,
                "help" => Cmd::Passthrough("_help".to_string()),
                "_help" => Cmd::PrintHelp,
                "c" | "cf" | "config" | "configure" => Cmd::Configure,
                "e" | "ed" | "edit" => match env::args().nth(2) {
                    Some(name) => Cmd::HopDirAndEdit(name),
                    None => Cmd::EditDir(Rabbit::from(current_dir, None)),
                },
                "g" | "gb" | "grab" => match env::args().nth(2) {
                    Some(shortcut) => Cmd::Grab(shortcut),
                    None => Cmd::PrintMsg("[error] No shortcut provided.".to_string()),
                },
                "loc" | "locate" => Cmd::Passthrough("_locate_bunnyhop_resources".to_string()),
                "_locate_bunnyhop_resources" => Cmd::LocateBunnyhop,
                "h" | "hs" | "history" | "hist" => match env::args().nth(2) {
                    Some(arg) => Cmd::Passthrough(format!("_history {}", arg)),
                    None => Cmd::Passthrough("_history".to_string()),
                },
                "_history" => match env::args().nth(2) {
                    Some(name) => Cmd::PullHistory(Rabbit::request(name)),
                    None => Cmd::ShowHistory,
                },
                "s" | "sr" | "search" => match env::args().nth(2) {
                    Some(term) => Cmd::Passthrough(format!("_search {}", term)),
                    None => Cmd::Passthrough("_search".to_string()),
                },
                "_search" => Cmd::Search(env::args().nth(2)),
                whatevs => Cmd::Use(Rabbit::RequestName(whatevs.to_string())),
            },
            None => Cmd::PrintMsg("[error] Unable to parse current arguments.".to_string()),
        }
    }
}

impl Hopper {
    pub fn execute(&mut self, cmd: Cmd) -> anyhow::Result<()> {
        match cmd {
            Cmd::Passthrough(cmd) => self.runner(cmd),
            Cmd::Use(bunny) => self.just_do_it(bunny),
            Cmd::SetBrb(loc) => self.brb(loc),
            Cmd::BrbHop => self.use_hop("back".to_string()),
            Cmd::ListHops => self.list_hops(None),
            Cmd::PrintHelp => Self::print_help(),
            Cmd::Remove(bunny) => self.remove_hop(bunny),
            Cmd::Configure => self.configure(),
            Cmd::LocateBunnyhop => self.show_locations(),
            Cmd::LocateShortcut(name) => self.print_hop(name),
            Cmd::HopDirAndEdit(name) => self.hop_to_and_open_dir(name),
            Cmd::EditDir(bunny) => self.edit_dir(bunny),
            Cmd::ShowHistory => self.show_history(None),
            Cmd::PullHistory(bunny) => self.search_history(bunny),
            Cmd::Search(filter_condition) => self.search_all(filter_condition),
            Cmd::Grab(shortcut) => self.grab(shortcut),
            Cmd::PrintMsg(msg) => {
                println!("{}", msg);
                Ok(())
            }
        }
    }
}
