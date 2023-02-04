use hopper;
use std::{env, io};

fn main() {
    let big_hopper = hopper::Hopper::read();
    let big_command = hopper::args::Cmd::parse();
    big_hopper.execute(big_command);
}
