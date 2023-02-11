__bunnyhop__() {
    export out=$(sh -c "__HOPPERCMD__ ${1} ${2} ${3}")
    if [[ "$out" == "__cd__"* ]]; then
        export dir=$(echo ${out#*" "})
        cd $dir
    elif [[ "$out" == "__cmd__"* ]]; then
        export edit_cmd=$(echo ${out#*" "})
        $edit_cmd
    else
        echo "$out"
    fi
}

alias __FUNCTION_ALIAS__="__bunnyhop__"
