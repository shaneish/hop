__bunnyhop__() {
    export out=$(__SHELL_CALLABLE__ -c "__HOPPERCMD__ ${1} ${2} ${3}")
    if [[ "$out" == "__cd__"* ]]; then
        if [[ "$out" == *"__cmd__"* ]]; then
            cd_cmd=$(echo ${out%%"__cmd__ "*})
            cd_dir=$(echo ${cd_cmd#*" "})
            cd "$cd_dir"
            last_cmd=$(echo ${out##*"__cmd__ "})
            __SHELL_CALLABLE__ -c "$last_cmd"
        else
            export dir=$(echo ${out#*" "})
            cd $dir
        fi
    elif [[ "$out" == "__cmd__"* ]]; then
        export edit_cmd=$(echo ${out#*" "})
        __SHELL_CALLABLE__ -c "$edit_cmd"
    else
        echo "$out"
    fi
}

alias __FUNCTION_ALIAS__="__bunnyhop__"
