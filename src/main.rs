use bhop;
use std::{env, io};

fn main() -> io::Result<()> {
    let hopper = bhop::hopper::Hopper::new(".config/hop");

    let output: String = match env::args().nth(1) {
        Some(cmd) => match cmd.as_str() {
            "add" => match &env::args().nth(2) {
                Some(name) => hopper.add_hop(env::current_dir().unwrap(), name),
                None => "echo \"[error] Need to specify name to add hop.\"".to_string(),
            },
            "ls" => hopper.list_hops(),
            "version" => format!("echo \"Hop {}\"", env!("CARGO_PKG_VERSION")),
            "brb" => hopper.brb(env::current_dir().unwrap()),
            _ => {
                if hopper.hop_names().contains(&cmd) {
                    hopper.hop(&cmd)
                } else {
                    "echo \"[error] Invalid command or shortcut name.\"".to_string()
                }
            }
        },
        None => "echo \"[error] Missing command.\"".to_string(),
    };
    println!("{}", output);
    Ok(())
}
