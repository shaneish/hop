__bhop_function__() {
    out=$(__SHELL_CALLABLE__ -c "__HOPPERCMD__ ${1} ${2} ${3}")
    IFS = "|" read -ra arr <<< "$out"
    cd ${arr[0]}
    __SHELL_CALLABLE__ -c "${arr[1]}"
}

alias __FUNCTION_ALIAS__="__bhop_execute__"
