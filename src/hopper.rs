use symlink;
use std::{
    io,
    env,
    convert::AsRef,
    path::Path,
};
use dirs::home_dir;

pub fn add_hop<T: AsRef<Path>>(path: T, name: &str) -> io::Result<()> {
    let hop_config_dir = format!("{}/.config/hop", home_dir().unwrap().into_os_string().into_string().unwrap());
    let sym_result = symlink::symlink_dir(path.as_ref(), format!("{}/{}", hop_config_dir, name));
    match sym_result {
        Ok(_) => println!("[hop] {} -> {}", name, path.as_ref().display().to_string()),
        Err(_) => println!("[error] unable to add hop {} -> {}", name, path.as_ref().display().to_string()),
    };
    sym_result
}

pub fn hop(name: &str) -> io::Result<()> {
    let hop_config_dir = format!("{}/.config/hop", home_dir().unwrap().into_os_string().into_string().unwrap());
    env::set_current_dir(&format!("{}/{}", hop_config_dir, name))
}
