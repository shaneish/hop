
# function that serves are runner for `hopper` to allow program to change directory of current terminal
def-env hop [cmd: string, p2: string = "", p3: string = ""] {
    let command = (nu -c ($"HOPPERCMD ($cmd) ($p2) ($p3)" | str trim))
    let new_loc = if ($command | str starts-with 'cd') {
        ($command | parse "{cmd} {dir}" | get dir | first)
    } else {
        nu -c $command
        $env.PWD
    }
    cd $new_loc
}
