__FUNCTION_ALIAS__() {
    out=$(__SHELL_CALLABLE__ -c "__HOPPERCMD__ ${1} ${2} ${3} ${4}")
    if [[ "$out" != *"__CMD_SEPARATOR__"* ]]; then
        echo $out
        return
    fi
    export arr=(${(@s:__CMD_SEPARATOR__:)out})
    cd ${arr[1]}
    __SHELL_CALLABLE__ -c "${arr[2]}"
}

