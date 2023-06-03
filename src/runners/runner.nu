def-env __bunnyhop__ [cmd: string, p2: string = "", p3: string = ""] {
    let command = (__SHELL_CALLABLE__ -c ($"__HOPPERCMD__ ($cmd) ($p2) ($p3)" | str trim))
    let new_loc = if ($command | str starts-with '__cd__') {
        ($command | split row "__cmd__ " | first | parse "__cd__ {dir}" | get dir | first)
    } else if ($command | str starts-with '__cmd__') {
        __SHELL_CALLABLE__ -c ($command | parse "__cmd__ {first_cmd}" | get first_cmd | first)
        $env.PWD
    } else {
        echo $command
        $env.PWD
    }
    cd $new_loc
    if ($command | str starts-with '__cd__') {
        if ($command | str contains '__cmd__') {
            __SHELL_CALLABLE__ -c ($command | split row "__cmd__ " | last)
        }
    }
}

alias hp = __bunnyhop__
