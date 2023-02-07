use crate::extract_path_ending;
use std::{env, path::PathBuf};

pub enum Cmd {
    AddHop(String, PathBuf),
    Move(String),
    AddHopAndMove(String, PathBuf),
    PrintMsg(String),
    SetBrb(PathBuf),
    BrbHop,
    ListHops,
}

impl Cmd {
    pub fn parse() -> Self {
        match env::args().nth(1) {
            Some(primary) => match primary.as_str() {
                "add" => match &env::args().nth(2) {
                    Some(name) => Cmd::AddHop(
                        name.to_string(),
                        PathBuf::from(env::current_dir().expect("[error] Unable to locate current directory.")),
                    ),
                    None => {
                        let current_path = env::current_dir()
                            .expect("[error] Unable to locate current directory.");
                        let current_pathbuf = PathBuf::from(&current_path);
                        let curr_path_end = extract_path_ending(current_path);
                        Cmd::AddHop(
                            curr_path_end,
                            current_pathbuf,
                        )
                    }
                },
                "ls" => Cmd::ListHops,
                "version" => Cmd::PrintMsg(format!("🐇 Hop 🐇 v.{}", env!("CARGO_PKG_VERSION"))),
                "-v" => Cmd::PrintMsg(format!("Hop - v.{}", env!("CARGO_PKG_VERSION"))),
                "brb" => {
                    Cmd::SetBrb(env::current_dir().expect("[error] Unable to add brb location."))
                }
                "back" => Cmd::BrbHop,
                whatevs => match &env::args().nth(2) {
                    Some(name) => Cmd::AddHopAndMove(name.to_string(), PathBuf::from(whatevs)),
                    None => Cmd::Move(whatevs.to_string()),
                },
            },
            None => {
                Cmd::PrintMsg("[error] Unable to ascertain or disambiguate command.".to_string())
            }
        }
    }
}
