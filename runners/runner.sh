hp() {
    export out=$(sh -c "__HOPPERCMD__ ${1} ${2} ${3}")
    if [[ "$out" == "__cd__"* ]]; then
        export dir=$(echo ${out##*" "})
        cd $dir
    elif [[ "$out" == "__cmd__"* ]]; then
        export edit_cmd=$(echo ${out##*" "})
        sh -c "$out"
    else
        echo "$out"
    fi
}
