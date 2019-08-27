repo := $(abspath $(dir $(dir $(lastword $(MAKEFILE_LIST)))))
curdir := $(abspath $(dir $(lastword $(MAKEFILE_LIST))))


####################################################################################################
# Daemon binary
####################################################################################################

daemon_bin := $(repo)/target/debug/lunacam-daemon
$(daemon_bin):
	@cargo build -q

compile: $(daemon_bin)
.PHONY: compile


####################################################################################################
# Running
####################################################################################################

export LC_LOG ?= info,lunacam_daemon=debug
export DATABASE_URL ?= $(curdir)/lunacam.db
run: compile
	@cargo run -q
.PHONY: run
