# hop

### what even is this?
i have a bash/zsh/nushell function named `short` that lets users jump to predefined directories easily.

the basic zsh function is defined as:

```zsh
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

and the nushell version is:
```nu
def-env short [p1: string, p2: string = ""] {
    let loc = if $p1 == "add" {
        let loc = $"~/.config/.shorts/($p2)"
        if not ($loc | path exists) {
            echo $"[cmd] `short ($p2)` -> ($env.PWD)"
            ln -s $env.PWD $loc
        }
        $loc
    } else {
        $"~/.config/.shorts/($p1)"
    }

    cd $loc
}
```

however, as these are separate scripts for the various shells i use (nushell for personal, bash and zsh for work), i've found it very annoying to have to update the same script three times every time i want to add a new feature.  `hop` is supposed to replicate the behavior of `short`, but in a single language so it's easily updated between various shells.

### how to install
simply clone this repo and run `make` from the root repo directory.

current install script that works on the most systems with the most shells requires a system install of `python3`.

to install from a bash-type shell, use:
```zsh
git clone https://github.com/gnoat/hop.github
cd hop
make install
```

to install from nushell, use:
```zsh
git clone https://github.com/gnoat/hop.github
cd hop
make all
```

note: if installing on a mac and using the `make unix` option, ensure sure you are using gnu-sed and not apple's horrible frankenstein version of sed.

the current build supports three different shells: nushell, zsh, and bash/dash/anything else that use ~/.bashrc.

### how to use
for general usage help:
```zsh
hp help # show basic commands

hp version # show version
```
to add a shortcut to your directory with the shortcut name `example`:
```zsh
hp add example
```
to add a shortcut to your current directory and use the current directory high level name as the shortcut name:
```zsh
hp add
```
to add a shortcut to a file that can be opened up in the set editor, use:
```zsh
hp add init.vim # will create a shortcut to init.vim named `init.vim`

hp add init.vim vi # will create a shortcut to init.vim named `vi`
```
to delete a shortcut with name `example`:
```zsh
hp rm example # can be used from any location

hp rm # can be used within the "example" directory

hp remove example # long form of rm command
```
to jump to the `example` named directory:
```zsh
hp example
```
to list all saved shortcuts:
```zsh
hp ls # shortened form
hp list # long form
```

