// Build script that will detect shells on your system and add runner functions for those shells to
// their configuration files.  These runners are needed because it's otherwise impossible to change
// directories in the terminal without executing `cd` as a shell command.  The runner functions
// basically just check to see if it's a `cd` command, some other type of command, or just text and
// it will run all necessary commands and print the text output.
//
// To add functionality for a new shell, the following need to be added:
//  1) New runner in the language of the new shell in the `runners` folder that matches the pattern
//     of `runner.{ext}`, where `ext` is the file format extension of shell scripts for the new
//     shell being added.
//  2) New shell added to the `Shell` enum.
//  3) `Shell` enum implementations must be updated for each of the following methods to provide
//     the appropriate metadata for the new shell being added (if it isn't the same as the default
//     method output):
//          a) call_cmd
//          b) source_cmd
//          c) name
//          d) env_var
//          e) ext
//          f) script
//          g) config_dir
//          h) config_name
//          i) find_default
//  4) New `Shell` method for deriving most probable default shell configuration file for the shell
//     being added will need to be created if the default shell configuration file is not in the
//     user's home directory.
//  5) New `Shell` enum variant needs to be added to `build.rs` in the vector of shells where being
//     fed into the `Runner`.
//
// With all these updates in place, the current build system should start configuring the new shell
// for use.
use dirs::home_dir;
use std::{
    env::var,
    fs::{read_to_string, OpenOptions},
    io::Write,
    path::PathBuf,
    process::Command,
};

pub enum Shell {
    Zsh,
    Bash,
    Nushell,
    Powershell,
    Elvish
}

impl Shell {
    // These are the foundational methods that need to be implemented for each new shell runner
    // support is being added for.
    fn call_cmd(&self) -> &str {
        // This method returns the command used to call shell commands in this from any other shell.
        match self {
            Shell::Zsh => "zsh",
            Shell::Bash => "sh",
            Shell::Nushell => "nu",
            Shell::Powershell => "pwsh",
            Shell::Elvish => "elvish",
        }
    }

    fn source_cmd(&self) -> &str {
        // This method returns the command used to source another file in a shell's config file.
        match self {
            Shell::Nushell => "source",
            _ => ".",
        }
    }

    fn name(&self) -> &str {
        // This method returns the proper name of the shell.
        match self {
            Shell::Zsh => "ZSH",
            Shell::Bash => "BASH",
            Shell::Nushell => "NUSHELL",
            Shell::Powershell => "POWERSHELL",
            Shell::Elvish => "ELVISH",
        }
    }

    fn env_var(&self) -> &str {
        // This method returns the environment variable that can be set to specify a non-standard
        // shell configuration file location.
        match self {
            Shell::Zsh => "BHOP_ZSH_CONFIG_DIR",
            Shell::Bash => "BHOP_BASH_CONFIG_DIR",
            Shell::Nushell => "BHOP_NUSHELL_CONFIG_DIR",
            Shell::Powershell => "BHOP_POWERSHELL_CONFIG_DIR",
            Shell::Elvish => "BHOP_ELVISH_CONFIG_DIR",
        }
    }

    fn ext(&self) -> &str {
        // This method returns the file format extension of saved shell scripts for each shell.
        match self {
            Shell::Nushell => "nu",
            Shell::Powershell => "ps1",
            Shell::Zsh => "zsh",
            Shell::Elvish => "elv",
            _ => "sh",
        }
    }

    fn script(&self) -> &str {
        // This method returns the specific implementation script for the runners in their
        // respective shells.  Any new shells added will need an appropriate runner implementation
        // added.
        match self {
            Shell::Nushell => include_str!("scripts/runner.nu"),
            Shell::Powershell => include_str!("scripts/runner.ps1"),
            Shell::Zsh => include_str!("scripts/runner.zsh"),
            Shell::Elvish => include_str!("scripts/runner.elv"),
            _ => include_str!("scripts/runner.sh"),
        }
    }

    fn config_dir(&self) -> &str {
        // This method returns the the configuration file or the directory the configuration file
        // is in when called as variables within their respective shells.
        //
        // For example, in Zsh if
        // you call `zsh -c "echo $ZDOTDIR"`, it will return the current configuration directory
        // that .zshrc is in or it will return an empty string.
        //
        // Similarly, `nu -c "echo $nu.env-path"` and `powershell -c "echo $profile"` will both
        // return the path to their direct configuration files that need to be updated.
        match self {
            Shell::Zsh => "ZDOTDIR",
            Shell::Bash => "HOME",
            Shell::Nushell => "nu.env-path",
            Shell::Powershell => "profile",
            Shell::Elvish => "$E:HOME/.config/elvish",
        }
    }

    fn config_name(&self) -> &str {
        // This method returns the default name of the configuration file for each shell that will
        // need to be updated.  This is used to parse the shell configuration file when the above
        // used commands fail or return nothing.
        match self {
            Shell::Zsh => ".zshrc",
            Shell::Bash => ".bashrc",
            Shell::Nushell => "env.nu",
            Shell::Powershell => "Microsoft.Powershell_profile.ps1",
            Shell::Elvish => "rc.elv",
        }
    }

    fn find_default(&self) -> Option<PathBuf> {
        // This method points to the specific implementations for determining the shell
        // configuration file for each shell.
        //
        // Zsh and Bash are relatively simple because they are
        // in the same location on almost every operating system.
        //
        // Nushell's default location varies between operating systems
        // and Powershell's varies not only between operating
        // systems but between different configurations of Windows itself.
        //
        // If the configuration file for any new shell being added doesn't default to a user's home
        // directory, a new method will have to be implemented and to derive the default
        // shell configuration path and it will have to be pointed to in this method.
        match self {
            Shell::Nushell => self.nushell_default(),
            Shell::Powershell => self.powershell_default(),
            _ => self.home_default(),
        }
    }
}

impl Shell {
    // These implementations are collections of methods for determining the default path for each
    // shell's configuration file.
    fn home_default(&self) -> Option<PathBuf> {
        home_dir().map(|home| home.join(self.config_name()))
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
                                .join(self.config_name()),
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

impl Shell {
    // These methods combine all other methods together to determine (where possible) the best
    // guess shell configuration path if a specific configuration path isn't specified through
    // environment variables.
    pub fn derive_config_path(&self) -> Option<PathBuf> {
        let from_env = var(self.env_var());
        match from_env {
            Ok(p) => Some(PathBuf::from(&p)),
            Err(_) => match Command::new(self.call_cmd())
                .arg("-c")
                .arg(format!("echo ${}", &self.config_dir()))
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
                                return Some(derived_path.as_path().join(self.config_name()));
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
}

pub struct Runners {
    alias: String,
    shells: Vec<Shell>,
    script_dir: PathBuf,
}

impl Runners {
    pub fn new(shells: Vec<Shell>, script_dir: PathBuf) -> Self {
        let alias = var("BHOP_DEFAULT_ALIAS").unwrap_or("hp".to_string());
        Runners {
            alias,
            shells,
            script_dir,
        }
    }

    pub fn add_runners(&self) {
        for shell in self.shells.iter() {
            match self.add_runner(shell) {
                Ok(_) => println!("Runner successfully added for {}.", shell.name()),
                Err(e) => println!("Failed to add runner for {}, error: {}", shell.name(), e),
            }
        }
    }

    fn add_runner(&self, shell: &Shell) -> anyhow::Result<()> {
        // This is the meat-and-potatoes method that actually does all the necessary imputation to
        // create a working runner function for a shell and add it to the configuration file for
        // the shell.
        println!(
            "[info] Adding runner `{}` to shell {}",
            &self.alias,
            shell.name()
        );
        match shell.derive_config_path() {
            Some(config_path) => {
                println!("[info] Located config file: {}", &config_path.display());
                let script_file_path = self.script_dir.join(format!("runner.{}", shell.ext()));
                {
                    let mut hop_script_file = OpenOptions::new()
                        .write(true)
                        .create(true)
                        .truncate(true)
                        .open(&script_file_path)?;
                    let mut conf_file = OpenOptions::new()
                        .append(true)
                        .read(true)
                        .create(true)
                        .open(&config_path)?;
                    let exe_name = env!("CARGO_PKG_NAME");
                    let source_cmd = format!(
                        "{} \"{}\"",
                        shell.source_cmd(),
                        &script_file_path.as_path().display()
                    )
                    .replace('\\', "/");
                    let script = shell
                        .script()
                        .replace("__HOPPERCMD__", exe_name)
                        .replace("__SHELL_CALLABLE__", shell.call_cmd())
                        .replace("__FUNCTION_ALIAS__", &self.alias)
                        .replace(
                            "__CMD_SEPARATOR__",
                            var("BHOP_CMD_SEPARATOR")
                                .unwrap_or("|".to_string())
                                .as_str(),
                        );
                    hop_script_file.write_all(script.as_bytes())?;
                    let config_file_contents = read_to_string(config_path)?;
                    if !config_file_contents.contains(&source_cmd) {
                        conf_file.write_all(format!("\n{}", source_cmd).as_bytes())?;
                    }
                }
                if !cfg!(windows) {
                    dos2unix::Dos2Unix::convert(&script_file_path.display().to_string(), true);
                }
                Ok(())
            }
            None => anyhow::bail!(format!("Unable to add runner for {}.", &shell.name())),
        }
    }
}
