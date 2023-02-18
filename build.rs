#[path = "runners/add_runners.rs"]
mod add_runners;
use add_runners::{
    Runners,
    Shell::{Bash, Nushell, Powershell, Zsh},
};

fn main() {
    println!("Building Bunnyhop version {}", env!("CARGO_PKG_VERSION"));
    for script in ["add_runners.rs", "runner.ps1", "runner.sh", "runner.nu"].iter() {
        println!("cargo:rerun-if-changed=runners/{}", script);
    }
    for env_var in [
        "BUNNYHOP_ZSH_CONFIG_DIR",
        "BUNNYHOP_BASH_CONFIG_DIR",
        "BUNNYHOP_NUSHELL_CONFIG_DIR",
        "BUNNYHOP_POWERSHELL_CONFIG_DIR",
    ]
    .iter()
    {
        println!("cargo:rerun-if-env-changed={}", env_var);
    }

    // Any new shells added in the future must be added in the following vector to be properly
    // configured with their respective runner script when `Bunnyhop` is built.
    let supported_shells = vec![Zsh, Bash, Nushell, Powershell];
    let runners = Runners::new(supported_shells);
    runners.add_runners();
}
