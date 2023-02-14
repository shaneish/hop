# bunnyhop

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

`bunnyhop` is supposed to replicate the behavior of `short`, but in a single language so it's easily updated between various shells.

this iteration also includes many improvements over the very simple shell function used before (and doesn't clutter you're system with unnecessary symlinks).

### how to install
simply clone this repo and run `make` from the root repo directory.

current install script that works on the most systems with the most shells requires a system install of `python3` and `cargo`.

if you currently don't have `rust` or `cargo` set up on your system, just run the following command before installing `bunnyhop`
```bash
curl https://sh.rustup.rs -sSf | sh
```
once `cargo` is installed, simply run the following to install `bunnyhop`:
```bash
git clone https://github.com/gnoat/hop.github
cd hop
make all
```
the default runner alias set is `hp`, use this to call `bunnyhop` from the command line (unless you set a custom alias).

once everything is installed and the shell hooks added, you can open the config file to set your editor and other preferences by simply typing:
```console
foo@bar:~$ hp configure
```
the current build supports four different shells: nushell, zsh, powershell, and bash/dash/anything else that use ~/.bashrc.

to see where all your configuration resources were provisioned, use:
```console
foo@bar:~$ hp locate
Config Directory    -> C:\Users\steph\.config\bunnyhop
Database Directory  -> C:\Users\steph\.config\bunnyhop\db
Bunnyhop Executable -> C:\Users\steph\Projects\hop\target\release\bunnyhop.exe
```

### how to use
for general usage help:
```console
foo@bar:~$ hp help # show basic commands

hp arg1 arg2
    1) First argument is required.
    2) Second argument is optional.

Valid first argument commands are:
        be used to refer to the shortcut for future use.
        If no second argument is given, the high level name will be used.
    2) ls or list: command to list the current shortcuts and their names.
    3) v or version: both commands to show current hop version info.
    4) brb: command to create a temporary shortcut to the current directory
        that can be jumped back to using the hp back command.
    5) rm or remove: command to remove the shortcut specified by arg2 or remove
        shortcut to current directory if no second argument specified.
    6) edit: open a file or directory within your editor of choice.
    7) configure or config: open your bunnyhop configuration file.
    8) locate: show all relevant installation directories if no second argument
        is given.  If second argument is given, list the full path to the given
        argument.
    9) ...: Any other first arguments given will be checked to see if it
        represents a valid directory/file to hop to.  This input can be a named
        shortcut, a file/directory in the current directory, or a file/directory
        from previous hp commands.

foo@bar:~$ version # show version
bunnyhop 🐇 v.0.2.4
```
to add a shortcut to your directory with the shortcut name `example`:
```console
foo@bar:~$ hp add example
[info] Hop created for example.
```
to add a shortcut to your current directory and use the current directory high level name as the shortcut name:
```console
echo $PWD
/usr/bin/cargo

foo@bar:~$ hp add
[info] Hop created for cargo.
```
to add a shortcut to a file that can be opened up in the set editor, use:
```console
foo@bar:~$ hp add init.vim # will create a shortcut to init.vim named `init.vim`
[info] Hop created for init.vim.

foo@bar:~$ hp add init.vim vi # will create a shortcut to init.vim named `vi`
[info] Hop created for vi.
```
to open a shortcut file in your configured editor of choice, use either of the following:
```console
foo@bar:~$ hp edit init.vim # full command consistent with opening a directory for editing

foo@bar:~$ hp init.vim # shortened command that just works for editing files and not directories
```
to delete a shortcut with name `example`:
```console
foo@bar:~$ rm example # can be used from any location
[info] Hop removed for shortcut: example.

echo $PWD
C:\Users\you\Documents\example

foo@bar:~$ hp rm # can be used within the "example" directory
[info] Hop removed for shortcut: example.

foo@bar:~$ hp remove example # long form of rm command
[info] Hop removed for shortcut: example.
```
to jump to the `example` named directory:
```console
foo@bar:~$ hp example
```
to jump to the `example` named directory and open your default editor in that directory:
```console
foo@bar:~$ hp edit example
```
to list all saved shortcuts:
```console
foo@bar:~$ hp ls # shortened form
appdata  -> C:\Users\steph\AppData\Local
back     -> C:\Users\steph\Projects\hop
hop      -> C:\Users\steph\Projects\hop
hpconf   -> C:\Users\steph\.config\bunnyhop

foo@bar:~$ hp list # long form
appdata  -> C:\Users\steph\AppData\Local
back     -> C:\Users\steph\Projects\hop
hop      -> C:\Users\steph\Projects\hop
hpconf   -> C:\Users\steph\.config\bunnyhop
```
to capture just the full path of a shortcut, you can again use the `locate` command:
```console
foo@bar:~$ hp locate init.vim # show full path to saved file `init.vim`
C:\Users\you\AppData\Local\nvim\init.vim

foo@bar:~$ hp locate example # show full path to saved directory `example`
C:\Users\you\Documents\example
```
you can use `hp` like `cd` to move into directories or edit files in your current directory (so long as the name doesn't conflict with a saved shortcut name).
this will then add that directory to the stored history and allow you to jump to it in the future without adding a shortcut directly.
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
[info] Hop created for cargo.

foo@bar: ~$ hp history
src -> /home/you/projects/hop

foo@bar:~$ cd ...

foo@bar:~$ echo $PWD
/home/you/projects

foo@bar:~$ hp src

foo@bar:~$ echo $PWD
/home/you/projects/hop/src
```

### custom configuration
by default, you can find the configuration file for `bunnyhop` at `~/.config/bunnyhop/bunnyhop.toml`.

check out the config file to see the current options available and to set your personal editors (default is `vi` for Unix and `notepad` for Windows).

### todo
1) write a more comprehensive suite of unit tests
2) add functionality to search stored history for possible locations
3) add customized editor launch commands (ie allow flags when calling an editor to open a file)
