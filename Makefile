help:
	@echo "required: Cargo & Python 3"
	@echo "usage: make all - install for all available shells."
	@echo "usage: make powershell - install for powershell."
	@echo "usage: make nushell - install for nushell."
	@echo "usage: make zsh - install for zsh."
	@echo "usage: make bash - install for bash/dash"

test_hopper:
	cargo test

build_hopper:
	cargo build --release

py_runners_all:
	python3 runners/add_runners.py

py_runners_powershell:
	python3 runners/add_runners.py ps

py_runners_bash:
	python3 runners/add_runners.py sh

py_runners_zsh:
	python3 runners/add_runners.py zsh

py_runners_nushell:
	python3 runners/add_runners.py nu

nushell: test_hopper build_hopper py_runners_nushell

powershell: test_hopper build_hopper py_runner_powershell

zsh: test_hopper build_hopper py_runners_zsh

bash: test_hopper build_hopper py_runners_bash

all: test_hopper build_hopper py_runners_all

install: test_hopper build_hopper py_runners_all
