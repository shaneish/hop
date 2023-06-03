extern crate sqlite;
extern crate anyhow;

use std::{fs, io, path::Path};
#[path = "src/add_runners.rs"]
mod add_runners;
use add_runners::{
    Runners,
    Shell::{Bash, Nushell, Powershell, Zsh},
};
#[path = "src/configs.rs"]
mod configs;
use configs::Environment;

fn copy_dir_all(src: impl AsRef<Path>, dst: impl AsRef<Path>) -> anyhow::Result<()> {
    fs::create_dir_all(&dst)?;
    for entry in fs::read_dir(src)? {
        let entry = entry?;
        let ty = entry.file_type()?;
        if ty.is_dir() {
            copy_dir_all(entry.path(), dst.as_ref().join(entry.file_name()))?;
        } else {
            fs::copy(entry.path(), dst.as_ref().join(entry.file_name()))?;
        }
    }
    Ok(())
}

fn create_database(db_path: impl AsRef<Path>) -> anyhow::Result<()> {
    let db_path = db_path.as_ref();
    if !db_path.exists() {
        let conn = sqlite::open(db_path)?;
        conn.execute(
            "CREATE TABLE IF NOT EXISTS named_hops (
            name TEXT PRIMARY KEY,
            location TEXT NOT NULL
            )",
        )?;
        conn.execute(
            "CREATE TABLE IF NOT EXISTS history (
            name TEXT NOT NULL,
            location TEXT NOT NULL,
            usage INTEGER NOT NULL
            )",
        )?;
    }
    Ok(())
}

fn main() {
    let env = Environment::new();

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
        "BUNNYHOP_ZSH_CONFIG_DIR",
        "BUNNYHOP_BASH_CONFIG_DIR",
        "BUNNYHOP_NUSHELL_CONFIG_DIR",
        "BUNNYHOP_POWERSHELL_CONFIG_DIR",
    ]
    .iter()
    {
        println!("cargo:rerun-if-env-changed={}", env_var);
    }

    // Move runner scripts to script config directory
    copy_dir_all("src/runners", &env.script_dir)
        .expect("Failed to copy runner scripts to script directory");

    // Add default hop.toml to config directory
    let default_toml = if cfg!(windows) {
        "src/defaults/windows_defaults.toml"
    } else {
        "src/defaults/unix_defaults.toml"
    };
    fs::copy(default_toml, &env.config_path).expect("Failed to copy hop.toml to config directory");

    // Create database if it doesn't exist
    create_database(&env.db_path).expect("Failed to create database");

    // Any new shells added in the future must be added in the following vector to be properly
    // configured with their respective runner script when `Bunnyhop` is built.
    let supported_shells = vec![Zsh, Bash, Nushell, Powershell];
    let runners = Runners::new(supported_shells);
    runners.add_runners();
}
