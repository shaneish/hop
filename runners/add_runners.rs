// Build script that will detect shells on your system and add runner functions for those shells to
// their configuration files.  These runners are needed because it's otherwise impossible to change
// directories in the terminal without executing `cd` as a shell command.  The runner functions
// basically just check to see if it's a `cd` command, some other type of command, or just text and
// it will run all necessary commands and print the text output.
//
// To add functionality for a new shell, the following need to be added:
//  1) New runner in the language of the new shell in the `runners` folder.
//  2) New shell added to the `Shell` enum.
//  3) New `ShellMetadata` entry for new enum variant.
//  4) New function within `ShellMetadata` to locate the default configuration file for the new
//     shell if the default configuration file for the new shell is not the user's home directory.
//  5) New enum variant needs to be added to `build.rs` where the `Runners` struct is created.
//
// With all these updates in place, the current build system should start configuring the new shell
// for use.
use dirs::home_dir;
use std::{
    env::var,
    fs::{read_to_string, OpenOptions},
    io::Write,
    path::{Path, PathBuf},
    process::Command,
};

pub enum Shell {
    Zsh,
    Bash,
    Nushell,
    Powershell,
}

#[derive(Clone)]
pub struct ShellMetadata<'a, 'b> {
    shell: &'b Shell,
    call_cmd: &'a str,
    source_cmd: &'a str,
    name: &'a str,
    env_var: &'a str,
    ext: &'a str,
    script: &'a str,
    config_dir: &'a str,
    config_name: &'a str,
}

impl<'a, 'b> ShellMetadata<'a, 'b> {
    pub fn new(shell: &'b Shell) -> Self {
        match *shell {
            Shell::Zsh => ShellMetadata {
                shell,
                call_cmd: "zsh",
                source_cmd: "source",
                name: "ZSH",
                env_var: "BUNNYHOP_ZSH_CONFIG_DIR",
                ext: "sh",
                script: include_str!("runner.sh"),
                config_dir: "ZDOTDIR",
                config_name: ".zshrc",
            },
            Shell::Bash => ShellMetadata {
                shell,
                call_cmd: "sh",
                source_cmd: "source",
                name: "BASH",
                env_var: "BUNNYHOP_BASH_CONFIG_DIR",
                ext: "sh",
                script: include_str!("runner.sh"),
                config_dir: "HOME",
                config_name: ".bashrc",
            },
            Shell::Nushell => ShellMetadata {
                shell,
                call_cmd: "nu",
                source_cmd: "source",
                name: "NUSHELL",
                env_var: "BUNNYHOP_NUSHELL_CONFIG_DIR",
                ext: "nu",
                script: include_str!("runner.nu"),
                config_dir: "nu.env-path",
                config_name: "env.nu",
            },
            Shell::Powershell => ShellMetadata {
                shell,
                call_cmd: "powershell",
                source_cmd: ".",
                name: "POWERSHELL",
                env_var: "BUNNYHOP_POWERSHELL_CONFIG_DIR",
                ext: "ps1",
                script: include_str!("runner.ps1"),
                config_dir: "profile",
                config_name: "Microsoft.Powershell_profile.ps1",
            },
        }
    }

    fn find_default(&self) -> Option<PathBuf> {
        match self.shell {
            Shell::Nushell => self.nushell_default(),
            Shell::Powershell => self.powershell_default(),
            _ => self.home_default(),
        }
    }

    pub fn derive_config_path(&self) -> Option<PathBuf> {
        let from_env = var(self.env_var);
        match from_env {
            Ok(p) => Some(PathBuf::from(&p)),
            Err(_) => match Command::new(self.call_cmd)
                .arg("-c")
                .arg(format!("echo ${}", &self.config_dir))
                .output()
            {
                Ok(out) => match String::from_utf8(out.stdout) {
                    Ok(p) => {
                        let derived = p.trim();
                        if !derived.is_empty() {
                            // the below is slightly more convoluted that I think it should be, but
                            // it's this way because calling `bash -c "echo $HOME"` and `sh -c "echo $HOME"`
                            // on Windows sucks and returns path strings unparsable by PathBuf
                            // depending on which implementation you're using (the two that I
                            // primarily use, Ubuntu WSL and Git-Bash both return completely
                            // different bad paths).
                            let derived_path = PathBuf::from(&derived);
                            if derived_path.is_file() {
                                return Some(derived_path);
                            } else if derived_path.is_dir() {
                                return Some(derived_path.as_path().join(self.config_name));
                            }
                        }
                        self.find_default()
                    }
                    Err(_) => self.find_default(),
                },
                Err(_) => self.find_default(),
            },
        }
    }

    fn home_default(&self) -> Option<PathBuf> {
        home_dir().map(|home| home.join(self.config_name))
    }

    fn nushell_default(&self) -> Option<PathBuf> {
        if cfg!(windows) {
            home_dir().map(|home| {
                home.join("AppData")
                    .join("Roaming")
                    .join("nushell")
                    .join("env.nu")
            })
        } else {
            home_dir().map(|home| home.join("config").join("nushell").join("env.nu"))
        }
    }

    fn powershell_default(&self) -> Option<PathBuf> {
        if cfg!(windows) {
            match home_dir() {
                Some(home) => {
                    if home.join("OneDrive").exists() {
                        Some(
                            home.join("OneDrive")
                                .join("Documents")
                                .join("WindowsPowerShell")
                                .join(self.config_name),
                        )
                    } else {
                        Some(
                            home.join("Documents")
                                .join("WindowsPowerShell")
                                .join("Microsoft.PowerShell_profile.ps1"),
                        )
                    }
                }
                None => None,
            }
        } else {
            home_dir().map(|home| {
                home.join(".config")
                    .join("powershell")
                    .join("Microsoft.Powershell_profile.ps1")
            })
        }
    }
}

pub struct Runners {
    alias: String,
    shells: Vec<Shell>,
}

impl Runners {
    pub fn new(shells: Vec<Shell>) -> Self {
        let alias = match var("BUNNYHOP_SHELL_ALIAS") {
            Ok(n) => n,
            Err(_) => "hp".to_string(),
        };
        Runners { alias, shells }
    }

    pub fn add_runners(&self) {
        for shell in self.shells.iter() {
            let shell_meta = ShellMetadata::new(shell);
            match self.add_runner(&shell_meta) {
                Ok(_) => println!("Runner successfully added for {}.", shell_meta.name),
                Err(e) => println!("Failed to add runner for {}, error: {}", shell_meta.name, e),
            }
        }
    }

    fn add_runner(&self, shell: &ShellMetadata) -> anyhow::Result<()> {
        println!(
            "[info] Adding runner `{}` to shell {}",
            &self.alias, shell.name
        );
        match shell.derive_config_path() {
            Some(config_path) => {
                println!("[info] Located config file: {}", &config_path.display());
                let config_file_path = config_path
                    .parent()
                    .unwrap()
                    .join(format!(".bunnyhop.{}", &shell.ext));
                let mut hop_conf_file = OpenOptions::new()
                    .write(true)
                    .create(true)
                    .open(&config_file_path)?;
                let mut conf_file = OpenOptions::new()
                    .append(true)
                    .read(true)
                    .create(true)
                    .open(&config_path)?;
                let exe_parent_dir = if cfg!(debug_assertions) {
                    "debug"
                } else {
                    "release"
                };
                let exe_path = Path::new(env!("CARGO_MANIFEST_DIR"))
                    .join("target")
                    .join(exe_parent_dir)
                    .join("bhop")
                    .display()
                    .to_string()
                    .replace('\\', "/");
                let source_cmd = format!(
                    "{} \"{}\"",
                    shell.source_cmd,
                    &config_file_path.as_path().display()
                )
                .replace('\\', "/");
                let script = shell
                    .script
                    .replace("__HOPPERCMD__", &exe_path)
                    .replace("__FUNCTION_ALIAS__", &self.alias);
                hop_conf_file.write_all(script.as_bytes())?;
                let config_file_contents = read_to_string(config_path)?;
                if !config_file_contents.contains(&source_cmd) {
                    conf_file.write_all(format!("\n{}", source_cmd).as_bytes())?;
                }
                Ok(())
            }
            None => anyhow::bail!(format!("Unable to add runner for {}.", &shell.name)),
        }
    }
}
