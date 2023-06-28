fn main() {
    let command = bhop::args::Request::parse();
    let hopper = bhop::Hopper::new();
    match hopper {
        Ok(mut hopper) => match hopper.execute(command) {
            Ok(_) => {}
            Err(e) => println!("[error] Unable to execute hop command: {}", e),
        },
        Err(e) => println!("[error] Unable to create hop instance: {}", e),
    };
}
