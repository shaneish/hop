use bhop::groups::BhopGroup;
use std::{fs, path::Path};

#[test]
fn test_from_str_cmd_string() {
    let toml = r#"
    test = "command"
    "#;
    let bhop_group = BhopGroup::from_str("test", toml).unwrap();
    assert_eq!(bhop_group.cmd, Some("command".to_string()));
    assert_eq!(bhop_group.editor, None);
    assert_eq!(bhop_group.files, None);
}

#[test]
fn test_from_str_with_table() {
    let toml = r#"
    [test_group]
    editor = "vim"
    files = ["file1.rs", "file2.rs"]
    "#;
    let bhop_group = BhopGroup::from_str("test_group", toml).unwrap();
    assert_eq!(bhop_group.cmd, None);
    assert_eq!(bhop_group.editor, Some("vim".to_string()));
    assert_eq!(bhop_group.files, Some(vec!["file1.rs".to_string(), "file2.rs".to_string()]));
}

#[test]
fn test_from_str_no_group() {
    let toml = r#"
    [other_group]
    editor = "vim"
    files = ["file1.rs", "file2.rs"]
    "#;
    let bhop_group = BhopGroup::from_str("test_group", toml);
    assert_eq!(bhop_group, None);
}

#[test]
fn test_from_file() {
    let path = Path::new("test.toml");
    fs::write(&path, r#"
    [test_group]
    editor = "vim"
    files = ["file1.rs", "file2.rs"]
    "#).unwrap();

    let bhop_group = BhopGroup::from("test_group", &path).unwrap();
    assert_eq!(bhop_group.cmd, None);
    assert_eq!(bhop_group.editor, Some("vim".to_string()));
    assert_eq!(bhop_group.files, Some(vec!["file1.rs".to_string(), "file2.rs".to_string()]));
}
