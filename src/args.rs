use crate::extract_path_ending;
use std::{env, path::Path};

pub enum Cmd {
    JumpHop(String),
    AddHop(String, Path),
    AddHopAndMove(String, String),
    Move(String),
    PrintMsg(String),
    SetBrb(Path),
    Disambiguate(String),
    BrbHop,
    ListHops,
}

impl Cmd {
    pub fn parse() -> Self {
        match env::args().nth(1) {
            Some(primary) => match primary.as_str() {
                "add" => match &env::args().nth(2) {
                    Some(name) => Cmd::AddHop(
                        name,
                        env::current_dir().expect("[error] Unable to locate current directory."),
                    ),
                    None => {
                        let current_path = env::current_dir()
                            .expect("[error] Unable to locate current directory.");
                        let curr_path_end = extract_path_ending(current_path);
                        Cmd::AddHop(
                            curr_path_end,
                            env::current_dir()
                                .expect("[error] Unable to locate current directory."),
                        )
                    }
                },
                "ls" => Cmd::ListHops,
                "version" => Cmd::PrintMsg(format!("Hop version - {}", env!("CARGO_PKG_VERSION"))),
                "--version" => {
                    Cmd::PrintMsg(format!("Hop version - {}", env!("CARGO_PKG_VERSION")))
                }
                "-v" => Cmd::PrintMsg(format!("Hop version - {}", env!("CARGO_PKG_VERSION"))),
                "brb" => {
                    Cmd::SetBrb(env::current_dir().expect("[error] Unable to add brb location."))
                }
                "back" => Cmd::BrbHop,
                whatevs => match &env::args().nth(2) {
                    Some(name) => Cmd::AddHopAndMove(whatevs.to_string(), name),
                    None => Cmd::Move(whatevs.to_string()),
                },
            },
            None => {
                Cmd::PrintMsg("[error] Unable to ascertain or disambiguate command.".to_string())
            }
        }
    }
}
