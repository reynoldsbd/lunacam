include $(repo)/tools/make/pal.mk
include $(repo)/tools/make/rust.mk



all: daemon
.PHONY: all



####################################################################################################
# Daemon binary
####################################################################################################

daemon := $(RUST_OUT_DIR)/lunacam-daemon

$(daemon): $(RUST_DEPS)
	@$(RUST_BUILD_CMD)
	@$(call PAL_TOUCH_FILE,$(daemon))

daemon: $(daemon)
.PHONY: daemon



####################################################################################################
# Running
####################################################################################################

export LC_LOG ?= info,lunacam_daemon=debug
export STATE_DIRECTORY ?= $(build)/run

run: daemon
	@$(PAL_CREATE_DIR) $(STATE_DIRECTORY)
	@cargo run -q
.PHONY: run



####################################################################################################
# Installation
#
# Technically, all is a prereq of install. However, this dependency is intentionally left out to
# prevent accidentally trying to cross-compile during imagebuild.
####################################################################################################

install: install.sh
	@./install.sh



####################################################################################################
# Cleaning
####################################################################################################

clean:
	@cargo clean
	@$(call PAL_RM,$(STATE_DIRECTORY)/daemon.db)
.PHONY: clean
