use::bhop::configs::{Configs, ReadConfig, ReadSettings};
use std::fs::File;
use std::io::Write;
use std::env;
use std::collections::HashMap;
use tempfile::tempdir;

#[test]
fn test_read_config_from_file() {
    let dir = tempdir().unwrap();
    let config_path = dir.path().join("config.toml");
    let mut file = File::create(&config_path).unwrap();
    write!(
        file,
        r#"
        [settings]
        default_editor = "nano"
        verbose = true
        [editors]
        python = "python3"
        "#,
    )
    .unwrap();

    let read_config = ReadConfig::new(&config_path);

    let expected_settings = ReadSettings {
        default_editor: Some("nano".to_string()),
        verbose: Some(true),
        ..ReadSettings::default()
    };
    let mut expected_editors = HashMap::new();
    expected_editors.insert("python".to_string(), "python3".to_string());

    assert_eq!(
        read_config,
        ReadConfig {
            settings: Some(expected_settings),
            editors: Some(expected_editors),
        }
    );
}

#[test]
fn test_configs_uses_defaults() {
    let dir = tempdir().unwrap();
    let config_path = dir.path().join("config.toml");
    File::create(&config_path).unwrap();

    let configs = Configs::new(&config_path);

    assert_eq!(configs.ls_display_block, 0);
    assert_eq!(configs.print_color_primary, [51, 255, 255]);
    assert_eq!(configs.verbose, false);
}

#[test]
fn test_configs_uses_values_from_file() {
    let dir = tempdir().unwrap();
    let config_path = dir.path().join("config.toml");
    let mut file = File::create(&config_path).unwrap();
    write!(
        file,
        r#"
        [settings]
        ls_display_block = 10
        verbose = true
        [editors]
        python = "python3"
        "#,
    )
    .unwrap();

    let configs = Configs::new(&config_path);

    assert_eq!(configs.ls_display_block, 10);
    assert_eq!(configs.verbose, true);
    assert_eq!(
        configs.editors,
        [("python".to_string(), "python3".to_string())]
            .iter()
            .cloned()
            .collect()
    );
}

#[test]
fn test_configs_uses_system_editor() {
    env::set_var("EDITOR", "nano");
    let dir = tempdir().unwrap();
    let config_path = dir.path().join("config.toml");
    File::create(&config_path).unwrap();

    let configs = Configs::new(&config_path);

    assert_eq!(configs.default_editor, "nano");
}

