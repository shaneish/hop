// Enum used to parse input arguments.  Ended up rolling my own arg parser instead of using an
// existing crate because I wanted `hp` commands to be more natural language-like and use dynamic
// positional commands
//
// Valid first argument commands are:
//    1) `add`: command to add a shortcut to the current directory.
//        - If a second argument is given, that argument is the name that
//            will be used to refer to the shortcut for future use
//        - If no second argument is given, the high level name of the current
//            directory will be added as the shortcut name.  As an example, if
//            the current directory is "~/.config/hop" and `hp add` is called,
//            it will create a shortcut to "~/.config/hop" named "hop"
//    2) `ls`: command to list the current shortcuts and their names.
//    3) `version` and `v`: both commands to show current version info.
//    4) `brb`: command to create a temporary shortcut to the current directory
//        that can be jumped back to using the `hp back` command.
use std::{env, path::{Path, PathBuf}};
use colored::Colorize;

pub enum Rabbit {
    Dir(String, PathBuf),
    File(String, PathBuf),
    Request(String),
}

impl Rabbit {
    pub fn from<T: AsRef<Path>>(input: T, name: Option<String>) -> Self {
        let current_name = match name {
            Some(given_name) => given_name,
            None => input.as_ref().file_stem().expect("[error] Unable to disambiguate file/directory.").to_str().expect("[error] Unable to convert file/directory name to UTF-8.").to_string(),
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
    PrintMsg(String),
    SetBrb(PathBuf),
    BrbHop,
    ListHops,
}

impl Cmd {
    pub fn parse() -> Self {
        match env::args().nth(1) {
            Some(primary) => match primary.as_str() {
                "add" => match env::args().nth(2) {
                    Some(f_or_d) => {
                        let current_dir = env::current_dir().expect("[error] Unable to locate current working directory.");
                        let mut f_or_d_path = PathBuf::from(&current_dir);
                        f_or_d_path.push(&f_or_d);
                        if f_or_d_path.is_file() {
                            Cmd::Use(Rabbit::from(f_or_d_path, env::args().nth(3)))
                        } else {
                            Cmd::Use(Rabbit::from(&current_dir, Some(f_or_d)))
                        }
                    },
                    None => Cmd::Use(Rabbit::from(env::current_dir().expect("[error] Unable to locate current working directory."), None)),
                },
                "ls" => Cmd::ListHops,
                "version" | "v" => Cmd::PrintMsg(format!("🐇 {} 🐇 {}{}", "Hop".cyan().bold(), "v.".bold(), env!("CARGO_PKG_VERSION").bright_white().bold())),
                "brb" => Cmd::SetBrb(env::current_dir().expect("[error] Unable to add brb location.")),
                "back" => Cmd::BrbHop,
                whatevs => Cmd::Use(Rabbit::Request(whatevs.to_string())),
            },
            None => Cmd::PrintMsg("[error] Unable to parse current arguments.".to_string()),
        }
    }
}
