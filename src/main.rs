use std::{
    io,
    env,
};
use hop;

fn main() -> io::Result<()>{
    let _configs = hop::config::read_configs().unwrap();
    match env::args().nth(1) {
        Some(cmd) => { match cmd.as_str() {
            "add" => { hop::hopper::add_hop(env::current_dir().unwrap(), &env::args().nth(2).expect("Need to specify name to add hop."))?; },
            _ => { hop::hopper::hop(&cmd)?; }, 
            };
        },
        None => {},
    };
    Ok(())
}
