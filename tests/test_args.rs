use bhop::args::Rabbit;
use std::{env::current_dir, path::PathBuf};

#[test]
fn test_file_parse() {
    let mut test_file = PathBuf::from(current_dir().expect("Unable to get current directory."));
    test_file.push("README.md");
    let no_nickname = Rabbit::from(test_file.clone(), None);
    match no_nickname {
        Rabbit::File(name, loc) => {
            assert_eq!(name, "README.md".to_string());
            assert_eq!(loc, test_file);
        }
        _ => assert!(false, "Rabbit::from() did not derive Rabbit::File variant."),
    };

    let nickname = "short".to_string();
    let with_nickname = Rabbit::from(test_file.clone(), Some(nickname.clone()));
    match with_nickname {
        Rabbit::File(name, loc) => {
            assert_eq!(nickname, name);
            assert_eq!(test_file, loc);
        }
        _ => assert!(false, "Rabbit::from() did not derive Rabbit::File variant."),
    };
}

#[test]
fn test_dir_parse() {
    let mut test_dir = PathBuf::from(current_dir().expect("Unable to get current directory."));
    test_dir.push("tests");
    let no_nickname = Rabbit::from(test_dir.clone(), None);
    match no_nickname {
        Rabbit::Dir(name, loc) => {
            assert_eq!(name, "tests".to_string());
            assert_eq!(loc, test_dir);
        }
        _ => assert!(false, "Rabbit::from() did not derive Rabbit::Dir variant."),
    };

    let nickname = "short".to_string();
    let with_nickname = Rabbit::from(test_dir.clone(), Some(nickname.clone()));
    match with_nickname {
        Rabbit::Dir(name, loc) => {
            assert_eq!(nickname, name);
            assert_eq!(test_dir, loc);
        }
        _ => assert!(false, "Rabbit::from() did not derive Rabbit::File variant."),
    };
}
