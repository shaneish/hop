#!/usr/bin/env python3

# Script to easily build `hp` command, even on systems without solid Bash support like Windows
from pathlib import Path
import os
from subprocess import run, PIPE


with open("runner.nu", "r") as f:
    nu_script = f.read()

with open("runner.sh", "r") as f:
    sh_script = f.read()

root_dir = Path(os.path.realpath(__file__)).parent.absolute()
exec_path = root_dir / "target" / "release" / "bhop"
nu_script = nu_script.replace("__HOPPERCMD__", str(exec_path))
sh_script = nu_script.replace("__HOPPERCMD__", str(exec_path))

if os.path.isfile("~/.zsh"):
    with open("~/.zsh", "ra") as f:
        if "hp()" in f.read():
            print("Unable to add for zsh as a function hp() is already defined in .zsh.")
        else:
            f.write(sh_script)

if os.path.isfile("~/.bashrc"):
    with open("~/.bashrc", "ra") as f:
        if "hp()" in f.read():
            print("Unable to add for bash as a function hp() is already defined in .bashrc.")
        else:
            f.write(sh_script)

nu_check = run(['nu', '-c', '$nu.env-path'], stdout=PIPE)
if nu_check.returncode == 0:
    nu_env_path = nu_check.stdout.decode("utf-8").strip()
    with open(nu_env_path, "ra") as f:
        if "def-env hp" in f.read():
            print("Unable to add for nushell as a function hp() is already defined.")
        else:
            f.write(nu_script)
