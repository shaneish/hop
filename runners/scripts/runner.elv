use str

fn __FUNCTION_ALIAS__ { |@a|
    var sep = __CMD_SEPARATOR__
    var bhop_cmd = (echo __HOPPERCMD__ $@a)
    var out = (__SHELL_CALLABLE__ -c $bhop_cmd)
    if (not (str:contains $out $sep)) {
        echo $out
    }
    var arr = [(str:split $sep $out)]
    cd $arr[0]
    __SHELL_CALLABLE__ -c $arr[1]
}

edit:add-var __FUNCTION_ALIAS__~ $__FUNCTION_ALIAS__~
