use bhop::metadata::Environment;
use serial_test::serial;
use std::env;
use std::path::PathBuf;
use tempfile::tempdir;

#[test]
#[serial]
fn test_environment_default() {
    env::remove_var("BHOP_CONFIG_DIRECTORY");
    let environment = Environment::new();

    let mut expected_config_dir = dirs::home_dir().unwrap_or(PathBuf::from("~/"));
    expected_config_dir.push(".config");
    expected_config_dir.push("bhop");

    assert_eq!(
        environment.config_path,
        expected_config_dir.join("bhop.toml")
    );
    assert_eq!(
        environment.db_path,
        expected_config_dir.join("db").join("bhop.db")
    );
}

#[test]
#[serial]
fn test_environment_from_env_var() {
    let temp_dir = tempdir().unwrap();
    env::set_var("BHOP_CONFIG_DIRECTORY", temp_dir.path());

    let environment = Environment::new();
    assert_eq!(environment.config_path, temp_dir.path().join("bhop.toml"));
    assert_eq!(
        environment.db_path,
        temp_dir.path().join("db").join("bhop.db")
    );
}

#[test]
#[serial]
fn test_environment_creates_dirs_and_files() {
    let temp_dir = tempdir().unwrap();
    env::set_var("BHOP_CONFIG_DIRECTORY", temp_dir.path());

    let _environment = Environment::new();

    assert!(temp_dir.path().join("bhop.toml").exists());
    assert!(temp_dir.path().join("scripts").exists());
    assert!(temp_dir.path().join("db").join("bhop.db").exists());
}

#[test]
#[serial]
fn test_environment_creates_database() {
    let temp_dir = tempdir().unwrap();
    env::set_var("BHOP_CONFIG_DIRECTORY", temp_dir.path());

    let _environment = Environment::new();
    let conn = sqlite::open(temp_dir.path().join("db").join("bhop.db")).unwrap();

    let mut shortcuts_exists = conn
        .prepare("SELECT name FROM sqlite_master WHERE type='table' AND name='shortcuts';")
        .unwrap();
    let mut history_exists = conn
        .prepare("SELECT name FROM sqlite_master WHERE type='table' AND name='history';")
        .unwrap();
    let mut shortcuts = Vec::new();
    let mut history = Vec::new();
    while let Ok(sqlite::State::Row) = shortcuts_exists.next() {
        shortcuts.push(shortcuts_exists.read::<String, _>("name").unwrap());
    }
    while let Ok(sqlite::State::Row) = history_exists.next() {
        history.push(history_exists.read::<String, _>("name").unwrap());
    }
    assert!(!shortcuts.is_empty());
    assert!(!history.is_empty());
}
