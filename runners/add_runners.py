# Script to easily build `hp` command, even on systems without solid Bash support like Windows
# Using a Python script to set up all the shell runners is the preferred method right now because
# it was a huge pain-in-the-@ss using shell scripts.
#
# Building and adding each shell runner to each supported shell in Windows, Ubuntu,
# and MacOS was massively inconsistent between the various shells and operating systems and required
# maintaining three separate shell scripts (zsh/bash/dash, nushell, and powershell) that all did about
# the same thing.
#
# Using Python allowed to me abstract and simplify adding the runners significantly.
#
# Ideally in the future, this script will be rewritten in a smaller scripting language written
# in Rust (like Rhai, Gluon, or Rune) so that the Python dependency can be removed leaving just Cargo.
#
# I considered including RustPython as a dependency and using it to execute this script, but that would
# add a massive number of dependencies and increase the crate memory footprint significantly for such
# a minor script.
#
# I'm assuming most people that are installing Rust applications from source have at least some versionE
# of Python 3 installed anyways, so nbd right now.
#
# I left out type hints and other newer Python features
# to make this as widely useable as possible, anything from Python 3.4 and up should work.

from pathlib import Path
from sys import argv
import os
from platform import platform
from subprocess import run, PIPE


def add_runner(config, shell, alias="hp", ext="sh", source="source"):
    # Type hints are as follows:
    #     config: Optional[str]
    #     shell: str
    #     ext: str
    #     source: str
    if config is None:
        print(f"Unable to locate config directory for shell {shell}...")
        print(
            f"If you'd like to install bunnyhop for {shell}, please set the following environment variable and run the associated `make` command:"
        )
        if shell == "zsh":
            print("Set BUNNYHOP_ZSH_CONFIG_DIR and run `make zsh`")
        elif shell == "bash/dash":
            print("Set BUNNYHOP_BASH_CONFIG_DIR and run `make sh`")
        elif shell == "nushell":
            print("Set BUNNYHOP_NUSHELL_CONFIG_DIR and run `make nu`")
        elif shell == "powershell":
            print("Set BUNNYHOP_POWERSHELL_CONFIG_DIR and run `make ps`")
        return

    print("Adding hp runner to ", shell)
    root_dir = Path(os.path.realpath(__file__)).parent.absolute()
    exe_dir = Path(root_dir).parent.absolute() / "target" / "release" / "bhop"
    conf_file = config.parent.absolute() / f".hop.{ext}"
    source_cmd = f"{source} \"{str(conf_file).replace(os.sep, '/')}\""
    in_shell_conf = True

    with open(root_dir / f"runner.{ext}", "r") as f:
        script = (
            f.read()
            .replace("__HOPPERCMD__", str(exe_dir).replace(os.sep, "/"))
            .replace("__FUNCTION_ALIAS__", alias)
        )

    with open(conf_file, "w") as f:
        f.write(script)

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
    # Locate shell configuration paths.  As the location of the nushell and powershell configurations
    # can vary depending on operating system, they are treated a bit differently and
    # their configuration directory is found dynamically.
    home_dir = Path(os.path.expanduser("~"))
    zsh_dir = Path(os.environ.get("BUNNYHOP_ZSH_CONFIG_DIR", home_dir / ".zshrc"))
    sh_dir = Path(os.environ.get("BUNNYHOP_BASH_CONFIG_DIR", home_dir / ".bashrc"))
    nu_dir = os.environ.get("BUNNYHOP_NUSHELL_CONFIG_DIR", None)
    ps_dir = os.environ.get("BUNNYHOP_POWERSHELL_CONFIG_DIR", None)
    alias = os.environ.get("BUNNYHOP_SHELL_ALIAS", "hp")

    # Configure for zsh
    if ("zsh" in argv) or (len(argv) < 2):
        add_runner(zsh_dir, "zsh", alias)

    # Configure for bash/dash
    if ("sh" in argv) or (len(argv) < 2):
        add_runner(sh_dir, "bash/dash", alias)

    # Configure for nushell
    if ("nu" in argv) or (len(argv) < 2):
        nu_dir_default_unix = home_dir / ".config" / "nushell" / "env.nu"
        nu_dir_default_windows = home_dir / "AppData" / "Roaming" / "nushell"
        nu_dir_correct = None

        nu_check = run(["nu", "-c", "$nu.env-path"], stdout=PIPE)
        if nu_dir is not None:
            nu_dir_correct = Path(nu_dir)
        elif nu_check.returncode == 0:
            if nu_check.returncode == 0:
                nu_dir_correct = Path(nu_check.stdout.decode("utf-8").strip())
        elif "windows" in platform().lower():
            if os.path.isdir(nu_dir_default_windows):
                nu_dir_correct = nu_dir_default_windows
        add_runner(nu_dir_correct, "nushell", alias, "nu")

    # Configure for pwershell
    if ("ps" in argv) or (len(argv) < 2):
        ps_check = run(["powershell", "echo $profile"], stdout=PIPE)
        ps_default_unix = (
            home_dir / ".config" / "powershell" / "Microsoft.PowerShell_profile.ps1"
        )
        ps_dir = None

        if ps_dir is not None:
            ps_dir = Path(ps_dir) / "profile.ps1"
        if ps_check.returncode == 0:
            ps_dir = Path(ps_check.stdout.decode("utf-8").strip())
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
        elif os.path.isdir(home_dir / ".config"):
            ps_dir = ps_default_unix
        add_runner(ps_dir, "powershell", alias, "ps1", ".")
