__FUNCTION_ALIAS__() {
    out=$(__SHELL_CALLABLE__ -c "__HOPPERCMD__ ${1} ${2} ${3}")
    IFS="__CMD_SEPARATOR__" read -ra arr <<< "$out"
    cd ${arr[0]}
    __SHELL_CALLABLE__ -c "${arr[1]}"
}

