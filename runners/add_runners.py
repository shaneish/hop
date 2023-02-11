# Script to easily build `hp` command, even on systems without solid Bash support like Windows

from pathlib import Path
from sys import argv
import os
from subprocess import run, PIPE


_HP_FUNC_START = "\n## hop runner begin ##"
_HP_FUNC_END = "\n## hop runner end ##"


def add_runner(config: Path, shell: str):
    print("Adding hp runner to ", shell)
    if shell == "nushell":
        ext = "nu"
    else:
        ext = "sh"
    root_dir = Path(os.path.realpath(__file__)).parent.absolute()
    exe_dir = Path(root_dir).parent.absolute() / "target" / "release" / "bhop"
    with open(root_dir / f"runner.{ext}", "r") as f:
        script = f.read().replace("__HOPPERCMD__", str(exe_dir).replace(os.sep, "/"))
    with open(config, "r+") as f:
        current_shell_conf = f.read()
        if _HP_FUNC_START in current_shell_conf:
            new_script = (
                current_shell_conf.split(_HP_FUNC_START)[0]
                + current_shell_conf.split(_HP_FUNC_END)[-1]
            )
            f.write(new_script)
    with open(config, "a") as f:
        f.write(_HP_FUNC_START)
        f.write(
            "\n# Below function that serves as a runner for `bhop`, allows program to change directory of current terminal location.\n"
        )
        f.write(script)
        f.write(_HP_FUNC_END)


if __name__ == "__main__":
    home_dir = Path(os.path.expanduser("~"))
    zsh_dir = Path(os.environ.get('HOP_ZSH_CONFIG_DIRECTORY', home_dir / ".zshrc"))
    sh_dir = Path(os.environ.get('HOP_BASH_CONFIG_DIRECTORY', home_dir / ".bashrc"))
    nu_dir = os.environ.get('HOP_ZSH_CONFIG_DIRECTORY', None)
    if ("zsh" in argv) or (len(argv) < 2):
        add_runner(home_dir / ".zshrc", "zsh")
    if ("sh" in argv) or (len(argv) < 2):
        add_runner(home_dir / ".bashrc", "bash/dash")
    if ("nu" in argv) or (len(argv) < 2):
        nu_dir_default = home_dir / ".config" / "nushell" / "env.nu"
        if nu_dir is None:
            nu_check = run(["nu", "-c", "$nu.env-path"], stdout=PIPE)
            if nu_check.returncode == 0:
                nu_dir_correct = Path(nu_check.stdout.decode("utf-8").strip())
            else:
                nu_dir_correct = nu_dir_default
        else:
            nu_dir_correct = Path(nu_dir)
        add_runner(nu_dir_correct, "nushell")
