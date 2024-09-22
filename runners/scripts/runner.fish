function hp
    set output (__SHELL_CALLABLE__ -c "__HOPPERCMD__ $argv")
    if not string match -q "*__CMD_SEPARATOR__*" $output
        echo $output
    else
        set cmds (string split "__CMD_SEPARATOR__" $output)
        cd $cmds[1]
        __SHELL_CALLABLE__ -c "$cmds[2]"
    end
end
