test_hopper:
	cargo test

build_hopper:
	cargo build --release

add_shell_runners:
	./runners/add_runners.sh

add_gitbash_runners:
	./runners/add_runners_windows_gitbash.sh

system_agnostic_python_runners:
	./runners/add_runners.py

unix: test_hopper build_hopper add_shell_runners

windows-git-bash: test_hopper build_hopper add_gitbash_runners

install: test_hopper build_hopper system_agnostic_python_runners

help:
	@echo "usage: make install"
