
# function that serves as a runner for `hopper`, allows program to change directory of current terminal
hp() {
    export out=$(sh -c "__HOPPERCMD__ ${1} ${2} ${3}")
    if [[ "$out" == "cd"* ]]; then
        export dir=$(echo ${out##*" "})
        cd $dir
    else
        sh -c $out
    fi
}
