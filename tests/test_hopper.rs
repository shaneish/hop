use bhop::{Hopper, Settings};
use serial_test::serial;
use std::{collections::HashMap, env, fs, io::Write, path::PathBuf};
use tempdir::TempDir;

fn get_test_hopper(config_dir: &PathBuf) -> bhop::Hopper {
    env::set_var(
        "HOP_CONFIG_DIRECTORY",
        config_dir.as_path().display().to_string(),
    );
    let hopper = Hopper::new().unwrap();
    hopper
}

fn get_default_editor() -> String {
    match env::consts::OS {
        "windows" => "notepad".to_string(),
        _ => "vi".to_string(),
    }
}

#[test]
#[serial]
fn test_initializing_resources() {
    let temp_dir =
        TempDir::new("tmp_test_init_resources").expect("Unable to create temp directory for test.");
    let config_dir = PathBuf::from(&temp_dir.path());
    let _ = get_test_hopper(&config_dir);
    let mut new_toml = config_dir.clone();
    new_toml.push("bunnyhop.toml");
    println!("{}", fs::read_to_string(&new_toml).unwrap());
    assert!(new_toml.exists(), "TOML wasn't created.");
    assert!(new_toml.is_file(), "TOML isn't a file.");

    let mut new_db = config_dir.clone();
    new_db.push("db");
    assert!(new_db.exists(), "DB directory wasn't created.");
    assert!(new_db.is_dir(), "DB directory isn't a directory.");

    new_db.push("bunnyhop.db");
    assert!(new_db.exists(), "DB file wasn't created.");
    assert!(new_db.is_file(), "DB file isn't a file.");
}

#[test]
#[serial]
fn test_read_default_configs() {
    let temp_dir = TempDir::new("tmp_test_default_configs")
        .expect("Unable to create temp directory for test.");
    let config_dir = PathBuf::from(&temp_dir.path());
    let hopper = get_test_hopper(&config_dir);
    println!("{:?}", hopper.config);
    let default_config = bhop::Config {
        settings: Settings {
            default_editor: get_default_editor(),
            max_history: 300,
            ls_display_block: 0,
        },
        editors: Some(HashMap::new()),
    };

    assert_eq!(hopper.config, default_config);
}

#[test]
#[serial]
fn test_read_configs_with_alt_editors() {
    let temp_dir =
        TempDir::new("tmp_test_alt_editors").expect("Unable to create temp directory for test.");
    let config_dir = PathBuf::from(&temp_dir.path());
    let mut alt_toml = config_dir.clone();
    alt_toml.push("bunnyhop.toml");
    let mut alt_toml_file =
        fs::File::create(&alt_toml).expect("Unable to create alternate bunnyhop.toml.");
    alt_toml_file
        .write_all(
            b"[settings]\ndefault_editor=\"vi\"\nmax_history=100\nls_display_block=10\n
                            [editors]\npy=\"nano\"\nipynb=\"code\"\nrust=\"nvim\"",
        )
        .expect("Unable to generate alternate hop.toml.");
    let hopper = get_test_hopper(&config_dir);
    let expected_editors = HashMap::from_iter(
        [("py", "nano"), ("ipynb", "code"), ("rust", "nvim")]
            .iter()
            .map(|(a, b)| (a.to_string(), b.to_string())),
    );
    let default_config = bhop::Config {
        settings: Settings {
            default_editor: "vi".to_string(),
            max_history: 100,
            ls_display_block: 10,
        },
        editors: Some(expected_editors),
    };

    assert_eq!(hopper.config, default_config);
}
