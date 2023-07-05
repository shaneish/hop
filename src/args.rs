// Enum used to parse input arguments.  Ended up rolling my own arg parser instead of using an
// existing crate because I wanted `hp` commands to be more natural language-like and use dynamic
use crate::Hopper;
use colored::Colorize;
use std::env;

#[derive(Debug, Eq, PartialEq)]
pub enum Request {
    Add(String, Option<String>),
    Remove(String),
    Use(String, Option<String>),
    Find(String),
    Group(String, Option<String>),
    Passthrough(String),
    Search(Option<String>),
    Notify(String),
    Help,
    Configure,
    Locate,
    Skip,
}

impl Request {
    pub fn parse() -> Self {
        let args: Vec<String> = match env::var("BHOP_TEST_ARGS") {
            Ok(val) => val.split(' ').map(|s| s.to_string()).collect(),
            Err(_) => env::args().collect(),
        };
        match args.get(1).map(|s| s.to_string()) {
            Some(cmd) => match cmd.as_str() {
                "add" | "+" => {
                    if let Some(reference) = args.get(2).map(|s| s.to_string()) {
                        Request::Add(reference, args.get(3).map(|s| s.to_string()))
                    } else {
                        Request::Notify("No shortcut name provided.".to_string())
                    }
                }
                "r" | "rm" | "remove" | "-" => match args.get(2).map(|s| s.to_string()) {
                    Some(reference) => Request::Remove(reference),
                    None => Request::Notify("No shortcut to remove provided.".to_string()),
                },
                "g" | "grp" | "group" | "->" | "!" => match args.get(2).map(|s| s.to_string()) {
                    Some(reference) => match args.get(3).map(|s| s.to_string()) {
                        Some(subgroup) => Request::Group(reference, Some(subgroup)),
                        None => Request::Group(reference, None),
                    },
                    None => Request::Notify("No shortcut to use provided.".to_string()),
                },
                "f" | "find" | "<-" | "?" => match args.get(2) {
                    Some(reference) => Request::Find(reference.to_string()),
                    None => Request::Notify("No reference to grab provided.".to_string()),
                },
                "brb" => Request::Add(".".to_string(), Some("back".to_string())),
                "loc" | "locate" => Request::Locate,
                "v" | "version" => Request::Passthrough("__bhop_version__".to_string()),
                "h" | "help" => Request::Passthrough("__bhop_help__".to_string()),
                "l" | "ls" | "list" | ".." => match args.get(2) {
                    Some(name) => Request::Passthrough(format!("__bhop_list__ {}", name)),
                    None => Request::Passthrough("__bhop_list__".to_string()),
                },
                "c" | "cfg" | "configure" => Request::Configure,
                "__bhop_version__" => {
                    println!(
                        "{} 🐇 {}{}",
                        "Bhop".cyan().bold(),
                        "v.".bold(),
                        env!("CARGO_PKG_VERSION")
                    );
                    Request::Skip
                }
                "__bhop_help__" => Request::Help,
                "__bhop_list__" => Request::Search(args.get(2).map(|s| s.to_string())),
                _ => Request::Use(cmd, args.get(2).map(|s| s.to_string())),
            },
            None => Request::Notify("No command provided.".to_string()),
        }
    }
}

impl Hopper {
    pub fn execute(&mut self, request: Request) -> anyhow::Result<()> {
        let output = match request {
            Request::Add(reference, name) => {
                self.add_shortcut(reference, name).map(|_| "".to_string())
            }
            Request::Remove(reference) => self.remove_shortcut(reference).map(|_| "".to_string()),
            Request::Group(reference, subgroup) => self.use_group(reference, subgroup),
            Request::Find(reference) => {
                let path = self.grab(reference);
                match path {
                    Some(p) => {
                        Ok(crate::sanitize(p).unwrap_or("Unable to sanitize path.".to_string()))
                    }
                    None => Err(anyhow::anyhow!("Unable to grab reference.")),
                }
            }
            Request::Use(reference, name) => match name {
                Some(n) => {
                    self.add_shortcut(&reference, Some(n))?;
                    self.bhop_it(reference, false)
                }
                None => self.bhop_it(reference, false),
            },
            Request::Passthrough(cmd) => self.passthrough(cmd),
            Request::Search(pattern) => self.search(pattern).map(|_| "".to_string()),
            Request::Notify(msg) => Ok(msg),
            Request::Configure => self.configure(),
            Request::Locate => self.locate(),
            Request::Help => Ok(format!(include_str!("defaults/help.txt"),
                "hp".cyan().bold(),
                "[COMMAND]".green().bold(),
                "add, a, +".green().bold(),
                "list, ls, l, ..".green().bold(),
                "remove, rm, r, -".green().bold(),
                "find, f, <-, ?".green().bold(),
                "group, grp, g, !".green().bold(),
                "locate, loc".green().bold(),
                "configure, cfg, c".green().bold(),
                "version, v".green().bold(),
                "help, h".green().bold(),
                "brb".green().bold(),
                "back".green().bold(),
                "[COMMAND]".green().bold())),
            Request::Skip => Ok("".to_string()),
        };
        match output {
            Ok(msg) => {
                if !msg.is_empty() {
                    print!("{}", msg)
                }
            }
            Err(err) => print!("{}", err),
        };
        Ok(())
    }
}
