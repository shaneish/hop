# bhop (bunnyhop)
[![Test Status](https://github.com/UnsafeOats/hop/actions/workflows/tests.yml/badge.svg)](https::/github.com/UnsafeOats/hop/actions)
[![Crates.io](https://img.shields.io/crates/v/bhop.svg)](https://crates.io/crates/bhop)
[![License](https://img.shields.io/github/license/UnsafeOats/hop)](LICENSE)

### what even is this tho??

**Tl;dr: a tool to quickly work with your filesystem via saved shortcuts and historical movements. Allows user to both jump to other locations or open files in other locations in an editor of your choice with a single command.**

I have a bash/zsh/nushell function named `short` that lets users jump to predefined directories easily.

The basic zsh function that I originally used was defined as:

```bash
short() {
    if [[ "$1" == "add" ]]; then
        if [ ! -f  ~/.config/.shorts/${2} ]; then
            echo "[cmd] \`short ${2}\` -> ${PWD}"
            ln -sf ./ ~/.config/.shorts/${2}
        fi
    else
        cd ~/.config/.shorts/${1}
    fi
}
```
However, as this required maintaining separate scripts for the various shells I use (nushell for personal, bash and zsh for work), I've found it very annoying to have to update the same script multiple times every time I want to add a new feature.

`bhop` (short for `bunnyhop`) is supposed to replicate the behavior of `short`, but in a single language so it's easily updated between various shells.

This iteration also includes many improvements over the very simple shell function used before (and doesn't clutter you're system with unnecessary symlinks).

### install
Install requires `cargo` (obvi, since it's a tool written in Rust).

If you currently don't have `rust` or `cargo` set up on your system, just run the following command before installing `bunnyhop`
```bash
curl https://sh.rustup.rs -sSf | sh
```
Once `cargo` is installed, simply run the following to install `bunnyhop`:
```bash
cargo install bhop
```
The default runner alias set is `hp`, use this to call `bunnyhop` from the command line (unless you set a custom alias).

If you'd like to use a different alias to call `bunnyhop` from your terminal, simple set the environment variable "**BHOP_SHELL_ALIAS**" prior to running the `cargo install bhop` command.

If you'd like to change to a different alias after installion, again you can just set the environment variable "**BHOP_SHELL_ALIAS**" and rerun the appropriate `cargo install bhop` command.

Once everything is installed and the shell hooks added, you can open the config file to set your editor and other preferences by simply typing:
```console
foo@bar:~$ hp configure # full command
```
The current build supports four different shells: nushell, zsh, powershell, and bash/dash/anything else that use ~/.bashrc.

To see where all your configuration resources were provisioned, use:
```console
foo@bar:~$ hp locate
/home/you/.config/bhop
```

### use
```console
foo@bar:~$ hp version # show version
bunnyhop 🐇 v.0.8.6
foo@bar:~$ hp v # alternate command
bunnyhop 🐇 v.0.8.6
```
To add a shortcut to your directory with the shortcut name `example`:
```console
foo@bar:~$ hp add . example
foo@bar:~$ hp + . example # alternate command
```
To add a shortcut to the `/home/you/.config` directory with the shortcut name `configs`:
```console
foo@bar:~$ hp add /home/you/.config configs
foo@bar:~$ hp + /home/you/.config configs # alternate command
```
Of course, if you want to move to your `/home/you/.config` director and rename it to `configs` at the same time:
```console
foo@bar:~$ hp /home/you/.config configs
```
To add a shortcut to a file that can be opened up in the set editor, use:
```console
# will create a shortcut to init.vim named `init.vim`
foo@bar:~$ hp add /home/you/.configs/nvim/init.vim

# will create a shortcut to init.vim named `nvim-confs`
foo@bar:~$ hp add /home/you/.configs/nvim/init.vim nvim-confs
```
If a shortcut maps to a file and not a directory, open that file in your editor by "jumping" to it:
```console
foo@bar:~$ hp init.vim # full command consistent with opening a directory for editing
```
To delete a shortcut with name `example`:
```console
foo@bar:~$ hp remove example
foo@bar:~$ hp rm example # alternate command
foo@bar:~$ hp - example # alternate command
```
To jump to the `example` named directory:
```console
foo@bar:~$ hp example
```
To jump to the `example` named directory and open your default configuration for that directory:
```console
foo@bar:~$ hp group example
foo@bar:~$ hp grp example # alternate command
foo@bar:~$ hp ! example # alternate command
```
To jump to the `example` named directory and open your configuration for that directory named `tests`:
```console
foo@bar:~$ hp group example tests
foo@bar:~$ hp grp example tests # alternate command
foo@bar:~$ hp ! example tests # alternate command
```
To list available hops:
```console
foo@bar:~$ hp list
Shortcuts:
configs  -> /home/you/.config
back     -> /home/you/Documents
hop-conf -> /home/you/.config/bhop/bhop.toml
example  -> /home/you/project/example_directory
History:
src -> /home/you/projects/hop/src
hop -> /home/you/projects/hop
foo@bar:~$ hp ls # alternate command
foo@bar:~$ hp .. # alternate command
```
To list available hops that have `hop` in them:
```console
foo@bar:~$ hp list *hop*
Shortcuts:
hop-conf -> /home/you/.config/bhop/bhop.toml
History:
src -> /home/you/projects/hop/src
hop -> /home/you/projects/hop
```
To grab the output path of the hop with shortcut name `example`:
```console
foo@bar:~$ hp find example
/home/you/project/example_directory
foo@bar:~$ hp f example # alternate command
foo@bar:~$ hp ? example # alternate command
```
You can use `hp` like `cd` to move into directories or edit files in your current directory.
This will then add that directory to the stored history and allow you to jump to it in the future without adding a shortcut directly.
```console
foo@bar:~$ echo $PWD
/home/you/projects/hop

foo@bar:~$ ls
.gitignore
Cargo.toml
README.md
src
target

foo@bar:~$ hp src

foo@bar:~$ echo $PWD
/home/you/projects/hop/src
```
If you place a file named `.bhop` in a directory with a shortcut, you can set different "windows" or "functions" for that shortcut directory.  An example `.bhop` file would look like:
```
foo@bar:~$ cat .bhop
test = "cargo test -- --nocapture"

[default]
files = ["src/*.rs"]

[tests]
files = ["tests/*.rs"]

[runners]
files = ["runners/*.rs", "runners/scripts/*"]

[notebooks]
editor = "jupyter-notebook"
files = ["examples/*.ipynb"]
```
Given the above `.bhop` file in a directory with shortcut name `example_shortcut`, you can run unit tests with the following commands:
```
foo@bar:~$ hp group example_shortcut test
...
foo@bar:~$ hp ! example_shortcut test # alternate command
```
Given the same `.bhop` file and the same shortcut name `example_shortcut`, you can open all the files and scripts in the `runners` folder in your default editor with:
```
foo@bar:~$ hp group example_shortcut runners
...
foo@bar:~$ hp ! example_shortcut runners # alternate command
```
### general flow for resolving `HP` commands
Calling a `hp` command with a shortcut name or path will attempt to do three things to resolve where it should jump you to:
1) Check if it is a valid location within the file system.
2) Check if it is within the saved list of shortcuts manually added by the user.
3) Check if it is within the history list of previous `hp` commands used by the user.

The order between 1) and 2) can be switched in your `bhop.toml` configuration file.

### custom configuration
By default, you can find the configuration file for `bhop` at `~/.config/bunnyhop/bunnyhop.toml`.

Check out the config file to see the current options available and to set your personal editors (default is `vi` for Unix and `notepad` for Windows).

Additionally, if you'd to use a location other than the default for your system to store the configuration files and SQLite database, you can set the following environment variables before running `bhop`.
1) `BHOP_CONFIG_DIRECTORY` - Sets the directory the configuration files will be provisioned in. Defaults to `~/.config/bhop`.

If your shell configuration file is set to a non-default location, you can set the following environment variables manually before building `bhop` and it will configure the runners in the location you set:
1) `BHOP_ZSH_CONFIG_DIR` - Directory your `.zshrc` file is located.
2) `BHOP_BASH_CONFIG_DIR` - Directory your `.bashrc` file is located.
3) `BHOP_NUSHELL_CONFIG_DIR` - Directory your nushell `env.nu` file is located.
4) `BHOP_POWERSHELL_CONFIG_DIR` - Directory your powershell `profile.ps1` or `Microsoft.PowerShell_profile.ps1` files are located.

### todo
1) Write a more comprehensive suite of unit tests.
2) Add customized editor launch commands (ie allow flags when calling an editor to open a file).
3) Make a Neovim plugin so I can easily navigate without opening a new terminal panel or closing current terminal panel.

## license
MIT.

If you have issues or would like some update/improvement, feel free to reach out and file an issue.

(fin.)
