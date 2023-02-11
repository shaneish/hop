function __bunnyhop__($cmd, $p1, $p2) {
    $fullCmd = Invoke-Expression "__HOPPERCMD__ $cmd $p1 $p2"
        if($fullCmd -like '__cd__ *') {
            cd $fullCmd.split(' ')[1]
        } elseif($fullCmd -like '__cmd__ *') {
            Invoke-Expression $fullCmd.substring(8)
        } else {
            echo $fullCmd
        }
}

Set-Alias -Name __FUNCTION_ALIAS__ -Value __bunnyhop__
