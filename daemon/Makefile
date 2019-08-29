repo := $(abspath $(dir $(lastword $(MAKEFILE_LIST)))/..)
build := $(repo)/build



####################################################################################################
# Daemon binary
####################################################################################################

profile ?= debug
ifeq "$(profile)" "release"
profile_arg := --release
endif

ifdef target
target_dir := $(build)/target/$(target)
target_arg := --target $(target)
else
target_dir := $(build)/target
endif

daemon := $(target_dir)/$(profile)/lunacam-daemon

$(daemon): Cargo.toml ../Cargo.lock $(shell find src -type f)
	@cargo build $(target_arg) $(profile_arg)

daemon: $(daemon)
.PHONY: daemon



####################################################################################################
# Running
####################################################################################################

export LC_LOG ?= info,lunacam_daemon=debug
export DATABASE_URL := $(build)/run/daemon.db

run: daemon
	@mkdir -p $(dir $(DATABASE_URL))
	@diesel migration run
	@cargo run -q
.PHONY: run



####################################################################################################
# Cleaning
####################################################################################################

clean:
	@cargo clean
	@rm -f $(DATABASE_URL)
.PHONY: clean



all: daemon
.PHONY: all
