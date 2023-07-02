function __FUNCTION_ALIAS__($cmd, $p1, $p2, $p3, $p4) {
    $fullCmd = Invoke-Expression "__HOPPERCMD__ $cmd $p1 $p2 $p3 $p4"
    if ($fullCmd -contains "__CMD_SEPARATOR__") {
        $to_move, $to_exec = $fullCmd.trim().Split("__CMD_SEPARATOR__", 2)
        cd $to_move
        Invoke-Expression $to_exec
    } else {
        echo $fullCmd
    }
}

