use dirs::home_dir;
use hopper::hopper;
use rand::Rng;
use std::fs::{create_dir, remove_dir};
use symlink;

#[test]
fn test_reading_toml() {
    let toml_str = "[defaults]\neditor=\"nvim\"";
    let hopper = hopper::Hopper::from(toml_str, ".config/hop");

    let expected = hopper::Config {
        defaults: hopper::Defaults {
            editor: "nvim".to_string(),
        },
    };

    assert_eq!(hopper.config, expected);
}

#[test]
fn test_list_files() {
    let home_dir = home_dir().unwrap();
    let temp_dir_name = format!("temp_test_dir-{}", rand::thread_rng().gen::<u32>());
    create_dir(home_dir.join(&temp_dir_name)).unwrap();
    println!("{:?}", home_dir.join(&temp_dir_name));
    let new_sym = symlink::symlink_dir(
        format!(
            "{}/{}",
            home_dir
                .join(&temp_dir_name)
                .into_os_string()
                .into_string()
                .unwrap(),
            "test"
        ),
        "/",
    );
    match new_sym {
        Ok(_) => {
            let hopper = hopper::Hopper::new(&temp_dir_name);
            let expected = "echo \"test -> /\"";

            assert_eq!(hopper.list_hops(), expected);
            remove_dir(home_dir.join(&temp_dir_name)).unwrap();
        }
        Err(_) => {
            remove_dir(home_dir.join(&temp_dir_name)).unwrap();
        }
    }
}
