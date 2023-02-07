use bhop;

fn main() {
    let big_command = bhop::args::Cmd::parse();
    let big_hopper = bhop::Hopper::new();
    match big_hopper {
        Ok(mut hopper) => hopper.execute(big_command),
        Err(e) => println!("[error] Unable to create hop instance: {}", e),
    };
}
