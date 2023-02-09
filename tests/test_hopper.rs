use bhop::{args::Rabbit, Hopper};
use serial_test::serial;
use std::{
    env,
    fs::{create_dir, remove_dir_all},
    path::PathBuf,
};

fn get_test_config() -> (PathBuf, bhop::Hopper) {
    let mut config_dir = PathBuf::from(&env::current_dir().unwrap());
    config_dir.push("tests");
    config_dir.push("test_resources");
    if config_dir.exists() {
        remove_dir_all(&config_dir).expect("Unable to delete temp test resources.")
    };
    create_dir(&config_dir).expect("Unable to create temp testing resource folder.");
    env::set_var(
        "HOP_CONFIG_DIRECTORY",
        &config_dir.as_path().display().to_string(),
    );
    let hopper = bhop::Hopper::new().unwrap();
    (config_dir, hopper)
}

#[test]
#[serial]
fn test_initializing_resources() {
    let (config_dir, _) = get_test_config();
    let curr_dir = config_dir.clone();
    let mut new_toml = curr_dir.clone();
    new_toml.push("hop.toml");
    assert!(new_toml.exists(), "TOML wasn't created.");
    assert!(new_toml.is_file(), "TOML isn't a file.");

    let mut new_db = curr_dir.clone();
    new_db.push("db");
    assert!(new_db.exists(), "DB directory wasn't created.");
    assert!(new_db.is_dir(), "DB directory isn't a directory.");

    new_db.push("hop.sqlite");
    assert!(new_db.exists(), "DB file wasn't created.");
    assert!(new_db.is_file(), "DB file isn't a file.");
}

#[test]
#[serial]
fn test_read_configs() {
    let (_, hopper) = get_test_config();
    let default_config = bhop::Config {
        editor: "nvim".to_string(),
        max_history_entries: 200,
        ls_display_block: 0,
    };

    assert_eq!(hopper.config, default_config);
}
