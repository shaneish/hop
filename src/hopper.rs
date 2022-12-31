use symlink;
use std::{
    io,
    env,
    convert::AsRef,
    path::Path,
};

pub fn add_hop<T: AsRef<Path>>(path: T, name: &str) -> io::Result<()> {
    symlink::symlink_dir(path, format!("~/.config/hop/{}", name))
}

pub fn hop(name: &str) -> io::Result<()> {
    env::set_current_dir(&format!("~/.config/hop/{}", name))
}
