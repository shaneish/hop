__bunnyhop__() {
    export out=$(sh -c "__HOPPERCMD__ ${1} ${2} ${3}")
    if [[ "$out" == "__cd__"* ]]; then
        if [[ "$out" == *"__cmd__"* ]]; then
            cd_cmd=$(echo ${out%%"__cmd__ "*})
            cd_dir=$(echo ${cd_cmd#*" "})
            cd "$cd_dir"
            last_cmd=$(echo ${out##*"__cmd__ "})
            sh -c "$last_cmd"
        else
            export dir=$(echo ${out#*" "})
            echo "2nd"
            echo $dir
            cd $dir
        fi
    elif [[ "$out" == "__cmd__"* ]]; then
        export edit_cmd=$(echo ${out#*" "})
        $edit_cmd
    else
        echo "$out"
    fi
}

alias __FUNCTION_ALIAS__="__bunnyhop__"
