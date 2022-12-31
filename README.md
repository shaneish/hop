# hop

I have a bash/zsh/nushell function named `short` that lets users jump to predefined directories easily.

The basic zsh function is defined as:
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

However, as these are separate scripts for the various shells I use (nushell for personal, bash and zsh for work), I've found it very annoying to have to update the same script three times every time I want to add a new feature.  `hop` is supposed to replicate the behavior of `short`, but in a single language so it's easily updated between various shells.
