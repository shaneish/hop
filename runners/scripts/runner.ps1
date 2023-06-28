function __FUNCTION_ALIAS__($cmd, $p1, $p2, $p3) {
    $fullCmd = Invoke-Expression "__HOPPERCMD__ $cmd $p1 $p2 $p3"
    $to_move, $to_exec = $fullCmd.trim().Split("__CMD_SEPARATOR__", 2)
    cd $to_move
    Invoke-Expression $to_exec
}

