# Script to easily build `hp` command, even on systems without solid Bash support like Windows

from pathlib import Path
from sys import argv
import os
from subprocess import run, PIPE


def add_runner(config, shell, alias="hp", ext="sh", source="source"):
    # Didn't add type hints to keep script to as many Python versions as possible
    # Type hints are as follows:
    #     config: Optional[str]
    #     shell: str
    #     ext: str
    #     source: str
    if config is None:
        print(f"Unable to locate config directory for shell {shell}.")
        return
    print("Adding hp runner to ", shell)
    root_dir = Path(os.path.realpath(__file__)).parent.absolute()
    exe_dir = Path(root_dir).parent.absolute() / "target" / "release" / "bunnyhop"
    conf_file = config.parent.absolute() / f".hop.{ext}"
    source_cmd = f"{source} {str(conf_file).replace(os.sep, '/')}"
    with open(root_dir / f"runner.{ext}", "r") as f:
        script = f.read().replace("__HOPPERCMD__", str(exe_dir).replace(os.sep, "/")).replace("__FUNCTION_ALIAS__", alias)
    with open(conf_file, "w") as f:
        f.write(script)
    in_shell_conf = True
    with open(config, "r+") as f:
        current_shell_conf = f.read()
        if source_cmd not in current_shell_conf:
            in_shell_conf = False
    if not in_shell_conf:
        with open(config, "a") as f:
            f.write("\n")
            f.write(source_cmd)
            f.write("\n")


if __name__ == "__main__":
    home_dir = Path(os.path.expanduser("~"))
    zsh_dir = Path(os.environ.get("BUNNYHOP_ZSH_CONFIG_DIR", home_dir / ".zshrc"))
    sh_dir = Path(os.environ.get("BUNNYHOP_BASH_CONFIG_DIR", home_dir / ".bashrc"))
    nu_dir = os.environ.get("BUNNYHOP_NUSHELL_CONFIG_DIR", None)
    ps_dir = os.environ.get("BUNNYHOP_POWERSHELL_CONFIG_DIR", None)
    alias = os.environ.get("BUNNYHOP_SHELL_ALIAS", "hp")
    if ("zsh" in argv) or (len(argv) < 2):
        add_runner(zsh_dir, "zsh", alias)
    if ("sh" in argv) or (len(argv) < 2):
        add_runner(sh_dir, "bash/dash", alias)
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
        add_runner(nu_dir_correct, "nushell", alias, "nu")
    if ("ps" in argv) or (len(argv) < 2):
        if ps_dir is not None:
            ps_dir = Path(ps_dir) / "profile.ps1"
        elif os.path.isdir(home_dir / "Documents" / "WindowsPowerShell"):
            ps_dir = home_dir / "Documents" / "WindowsPowerShell" / "profile.ps1"
        elif os.path.isdir(home_dir / "OneDrive" / "Documents" / "WindowsPowerShell"):
            ps_dir = (
                home_dir
                / "OneDrive"
                / "Documents"
                / "WindowsPowerShell"
                / "profile.ps1"
            )
        add_runner(ps_dir, "powershell", alias, "ps1", ".")
