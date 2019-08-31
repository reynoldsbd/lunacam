# Rust Helpers

ifndef RUST_MK
RUST_MK :=

include $(repo)/tools/make/pal.mk


RUST_PROFILE ?= debug
ifeq "$(RUST_PROFILE)" "release"
RUST_PROFILE_ARG = --release
endif


ifdef RUST_TARGET
RUST_OUT_DIR = $(build)/target/$(RUST_TARGET)/$(RUST_PROFILE)
RUST_TARGET_ARG = --target $(RUST_TARGET)
else
RUST_OUT_DIR = $(build)/target/$(RUST_PROFILE)
endif


ifdef PAL_WINDOWS
RUST_BIN_SUFFIX = .exe
endif


RUST_BIN_PATH = $(RUST_OUT_DIR)/$(1)$(RUST_BIN_SUFFIX)


RUST_DEPS = $(repo)/Cargo.lock Cargo.toml $(call PAL_ENUM_DIR,src)
RUST_BUILD_CMD = cargo build $(RUST_TARGET_ARG) $(RUST_PROFILE_ARG)


endif
