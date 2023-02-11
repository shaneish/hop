use bunnyhop;

fn main() {
    let big_command = bunnyhop::args::Cmd::parse();
    let big_hopper = bunnyhop::Hopper::new();
    match big_hopper {
        Ok(mut hopper) => match hopper.execute(big_command) {
            Ok(_) => {}
            Err(e) => println!("[error] Unable to execute hop command: {}", e),
        },
        Err(e) => println!("[error] Unable to create hop instance: {}", e),
    };
}
