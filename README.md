# hop

### what even is this?
i have a bash/zsh/nushell function named `short` that lets users jump to predefined directories easily.

the basic zsh function that i originally used was defined as:

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
however, as this required maintaining separate scripts for the various shells i use (nushell for personal, bash and zsh for work), i've found it very annoying to have to update the same script multiple times every time i want to add a new feature.

`hop` is supposed to replicate the behavior of `short`, but in a single language so it's easily updated between various shells.

this iteration also includes many improvements over the very simple shell function used before (and doesn't clutter you're system with unnecessary symlinks).

### how to install
simply clone this repo and run `make` from the root repo directory.

current install script that works on the most systems with the most shells requires a system install of `python3` and `cargo`.

if you currently don't have `rust` or `cargo` set up on your system, just run the following command before installing `hop`
```bash
curl https://sh.rustup.rs -sSf | sh
```
once `cargo` is installed, simply run the following to install `hop`:
```bash
git clone https://github.com/gnoat/hop.github
cd hop
make all
```
once everything is installed and the shell hooks added, you can open the config file to set your editor and other preferences by simply typing:
```bash
hp configure
```
the current build supports four different shells: nushell, zsh, powershell, and bash/dash/anything else that use ~/.bashrc.

to see where all your configuration resources were provisioned, use:
```bash
hp locate
```

### how to use
for general usage help:
```bash
hp help # show basic commands
hp version # show version
```
to add a shortcut to your directory with the shortcut name `example`:
```bash
hp add example
```
to add a shortcut to your current directory and use the current directory high level name as the shortcut name:
```bash
hp add
```
to add a shortcut to a file that can be opened up in the set editor, use:
```bash
hp add init.vim # will create a shortcut to init.vim named `init.vim`
hp add init.vim vi # will create a shortcut to init.vim named `vi`
```
to open a shortcut file in your configured editor of choice, use either of the following:
```bash
hp edit init.vim # full command consistent with opening a directory for editing
hp init.vim # shortened command that just works for editing files and not directories
```
to delete a shortcut with name `example`:
```bash
hp rm example # can be used from any location
hp rm # can be used within the "example" directory
hp remove example # long form of rm command
```
to jump to the `example` named directory:
```bash
hp example
```
to jump to the `example` named directory and open your default editor in that directory:
```bash
hp edit example
```
to list all saved shortcuts:
```bash
hp ls # shortened form
hp list # long form
```
to capture just the full path of a shortcut, you can again use the `locate` command:
```bash
hp locate init.vim # show full path to saved file `init.vim`
hp locate example # show full path to saved directory `example`
```
### custom configuration
by default, you can find the configuration file for `hop` at `~/.config/bunnyhop/bunnyhop.toml`.

check out the config file to see the current options available and to set your personal editors (default is `vi` for Unix and `notepad` for Windows).
