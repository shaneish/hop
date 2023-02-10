
# function that serves as a runner for `bhop`, allows program to change directory of current terminal
def-env hp [cmd: string, p2: string = "", p3: string = ""] {
    let command = (nu -c ($"__HOPPERCMD__ ($cmd) ($p2) ($p3)" | str trim))
    let new_loc = if ($command | str starts-with '__cd__') {
        ($command | parse "__cd__ {dir}" | get dir | first)
    } else if ($command | str starts-with '__cmd__') {
        nu -c ($command | parse "__cmd__ {shell_cmd}" | get shell_cmd | first)
        $env.PWD
    } else {
        nu -c ($"__HOPPERCMD__ ($cmd) ($p2) ($p3)" | str trim)
        $env.PWD
    }
    cd $new_loc
}
