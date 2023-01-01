test_hopper:
	cargo test

build_hopper:
	cargo build --release

add_shell_runners:
	./runners/add_runners.sh

install: test_hopper build_hopper add_shell_runners
 
help:
	@echo "usage: make install"
