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

daemon: $(daemon)
.PHONY: daemon



####################################################################################################
# Running
####################################################################################################

export LC_LOG ?= info,lunacam_daemon=debug
export DATABASE_URL ?= $(build)/run/daemon.db

run: daemon
	@$(PAL_CREATE_DIR) $(dir $(DATABASE_URL))
	@diesel migration run
	@cargo run -q
.PHONY: run



####################################################################################################
# Cleaning
####################################################################################################

clean:
	@cargo clean
	@$(call PAL_RM,$(DATABASE_URL))
.PHONY: clean
