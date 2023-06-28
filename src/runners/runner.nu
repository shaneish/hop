def-env __bhop_function__ [cmd: string, a: string = "", b: string = "", c: string = ""] {
    let command = (__SHELL_CALLABLE__ -c ($"__HOPPERCMD__ ($cmd) ($a) ($b) ($c)" | str trim))
    echo $command | split column "|" to_move to_exec
    __SHELL_CALLABLE__ -c $"($to_exec)"
    cd to_move
}

alias hp = __bhop_function__
