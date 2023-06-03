function __bunnyhop__($cmd, $p1, $p2) {
    $fullCmd = Invoke-Expression "__HOPPERCMD__ $cmd $p1 $p2"
    if($fullCmd -like '__cd__ *') {
        if($fullCmd -like "*__cmd__*") {
            cd ($fullCmd -split "__cmd__ ")[0].substring(7)
            Invoke-Expression ($fullCmd -split "__cmd__ ")[-1]
        } else {
            cd $fullCmd.substring(7)
        }
    } elseif($fullCmd -like '__cmd__ *') {
        Invoke-Expression $fullCmd.substring(8)
    } else {
        echo $fullCmd
    }
}

Set-Alias -Name __FUNCTION_ALIAS__ -Value __bunnyhop__
