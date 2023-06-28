def-env __FUNCTION_ALIAS__ [cmd: string, p1: string = "", p2: string = "", p3: string = ""] {
    let command = (__SHELL_CALLABLE__ -c ($"__HOPPERCMD__ ($cmd) ($p1) ($p2) ($p3)" | str trim))
    let arr = ($command | split column "__CMD_SEPARATOR__" to_move to_exec)
    cd ($arr.to_move | get 0)
    __SHELL_CALLABLE__ -c $"($arr.to_exec | get 0)"
}

