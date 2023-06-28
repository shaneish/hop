extern crate anyhow;
extern crate sqlite;

#[path = "runners/add_runners.rs"]
mod add_runners;
use add_runners::{
    Runners,
    Shell::{Bash, Nushell, Powershell, Zsh},
};
#[path = "src/metadata.rs"]
mod metadata;
use metadata::Environment;

fn main() {
    let env = Environment::new();
    let script_dir = env
        .config_path
        .parent()
        .expect("Unable to find config dir")
        .join("scripts");

    println!("Building Bunnyhop version {}", env!("CARGO_PKG_VERSION"));
    for script in [
        "src/add_runners.rs",
        "src/runners/runner.ps1",
        "src/runners/runner.sh",
        "src/runners/runner.nu",
    ]
    .iter()
    {
        println!("cargo:rerun-if-changed=runners/{}", script);
    }
    for env_var in [
        "BHOP_ZSH_CONFIG_DIR",
        "BHOP_BASH_CONFIG_DIR",
        "BHOP_NUSHELL_CONFIG_DIR",
        "BHOP_POWERSHELL_CONFIG_DIR",
    ]
    .iter()
    {
        println!("cargo:rerun-if-env-changed={}", env_var);
    }

    // Any new shells added in the future must be added in the following vector to be properly
    // configured with their respective runner script when `Bunnyhop` is built.
    let supported_shells = vec![Zsh, Bash, Nushell, Powershell];
    let runners = Runners::new(supported_shells, script_dir);
    runners.add_runners();
}
