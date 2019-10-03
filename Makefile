# TODO: remove non-LC_* vars
export repo := $(abspath $(dir $(lastword $(MAKEFILE_LIST))))
export build := $(repo)/build
export LC_SOURCE := $(abspath $(dir $(lastword $(MAKEFILE_LIST))))
export LC_BUILD := $(LC_SOURCE)/build
export LC_TOOLS := $(LC_SOURCE)/tools



FORCE:



####################################################################################################
# Files under $(pseudo) represent non-file dependencies (e.g. Docker images)
####################################################################################################

pseudo := $(build)/pseudo
$(pseudo):
	@mkdir -p $(pseudo)



####################################################################################################
# The crossbuild Docker image is used to cross-compile binaries for the Raspberry Pi
####################################################################################################

ifndef CROSSBUILD



crossbuild := $(pseudo)/crossbuild
crossbuild_dir := $(repo)/tools/crossbuild
crossbuild_img := lc-crossbuild
crossbuild_cache := lc-crossbuild-cache

$(crossbuild): $(shell find $(crossbuild_dir) -type f) | $(pseudo)
	@docker build -t $(crossbuild_img) $(crossbuild_dir)
	@touch $(crossbuild)

pi_triple := arm-unknown-linux-gnueabihf

crossbuild_cmd := docker run

# Don't try to use -it when running headless (i.e. during CI build)
ifeq "$(shell tty -s; echo $$?)" "0"
crossbuild_cmd += -it
endif

crossbuild_cmd += --rm
crossbuild_cmd += -v $(repo):/source
crossbuild_cmd += -v $(crossbuild_cache):/root/.cargo
crossbuild_cmd += -w /source
crossbuild_cmd += --env OUT_UID=$(shell id -u)
crossbuild_cmd += --env OUT_GID=$(shell id -g)
crossbuild_cmd += $(crossbuild_img)

crossbuild_out_dir := $(build)/target/$(pi_triple)/release
crossbuild_agent := $(crossbuild_out_dir)/lcagent
crossbuild_portal := $(crossbuild_out_dir)/lcportal

$(crossbuild_agent): $(crossbuild) FORCE
	@$(crossbuild_cmd) agent
	@touch $(crossbuild_agent)

crossbuild-agent: $(crossbuild_agent)

$(crossbuild_portal): $(crossbuild) FORCE
	@$(crossbuild_cmd) portal
	@touch $(crossbuild_portal)

crossbuild-portal: $(crossbuild_portal)

.PHONY: crossbuild crossbuild-agent crossbuild-portal



endif



####################################################################################################
# Rust binaries
####################################################################################################

LC_PROFILE ?= debug
ifeq "$(LC_PROFILE)" "release"
profile_arg := --release
endif

ifdef LC_TARGET
rust_dir := $(build)/target/$(LC_TARGET)/$(LC_PROFILE)
target_arg := --target $(LC_TARGET)
else
rust_dir := $(build)/target/$(LC_PROFILE)
endif

cargo_cmd := cargo build $(target_arg) $(profile_arg)

agent := $(rust_dir)/lcagent

$(agent): Cargo.toml Cargo.lock $(shell find src -type f)
	@$(cargo_cmd) --bin lcagent
	@touch $(agent)

agent: $(agent)

.PHONY: agent



####################################################################################################
# Portal
####################################################################################################

portal: | $(pseudo)
	@$(MAKE) --no-print-directory -C portal

run-portal: | $(pseudo)
	@$(MAKE) --no-print-directory -C portal run

install-portal: | $(pseudo)
	@$(MAKE) --no-print-directory -C portal install

deploy-portal: $(crossbuild_portal) | $(pseudo)
	@$(MAKE) --no-print-directory -C portal deploy RUST_TARGET=$(pi_triple) RUST_PROFILE=release

.PHONY: portal run-portal deploy-portal



####################################################################################################
# Custom Raspbian image with LunaCam preconfigured
####################################################################################################

pigen := $(pseudo)/pi-gen
pigen_dir := $(LC_BUILD)/pi-gen

# TODO: checkout specific commit
$(pigen): | $(pseudo)
	@git clone --depth 1 https://github.com/RPi-Distro/pi-gen $(pigen_dir)
	@touch $(pigen_dir)/stage2/SKIP_IMAGES
	@touch $(pigen)



stg_common := $(pseudo)/stg-common
stg_common_dir := $(pigen_dir)/common
cfg_common := $(pigen_dir)/config

$(stg_common): $(pigen) $(LC_TOOLS)/pi-gen/prerun.sh $(shell find $(LC_TOOLS)/pi-gen/common -type f)
	@mkdir -p $(stg_common_dir)
	@rsync -r --delete $(LC_TOOLS)/pi-gen/common/ $(stg_common_dir)
	@cp $(LC_TOOLS)/pi-gen/prerun.sh $(stg_common_dir)/prerun.sh
	@touch $(stg_common)

$(cfg_common): $(LC_TOOLS)/pi-gen/config.sh $(stg_common)
	@cp $(LC_TOOLS)/pi-gen/config.sh $(cfg_common)



stg_agent := $(pseudo)/stg-agent
stg_agent_dir := $(pigen_dir)/agent
cfg_agent := $(pigen_dir)/config-agent
agent_image := $(pigen_dir)/deploy/image_$(shell date -uI)-lunacam-agent.img

$(stg_agent): $(pigen) $(LC_TOOLS)/pi-gen/prerun.sh $(shell find $(LC_TOOLS)/pi-gen/agent -type f) $(crossbuild_agent)
	@mkdir -p $(stg_agent_dir)
	@rsync -r --delete $(LC_TOOLS)/pi-gen/agent/ $(stg_agent_dir)
	@cp $(LC_TOOLS)/pi-gen/prerun.sh $(stg_agent_dir)/prerun.sh
	@cp $(crossbuild_agent) $(stg_agent_dir)/02-agent/files/lcagent
	@touch $(stg_agent)

$(cfg_agent): $(LC_TOOLS)/pi-gen/config-agent.sh $(stg_agent) $(cfg_common)
	@cp $(LC_TOOLS)/pi-gen/config-agent.sh $(cfg_agent)

$(agent_image): $(cfg_agent)
	@docker rm -v pigen_work || true
	@cd $(pigen_dir) && ./build-docker.sh -c config-agent

agent-image: $(agent_image)



stg_portal := $(pseudo)/stg-portal
stg_portal_dir := $(pigen_dir)/portal
cfg_portal := $(pigen_dir)/config-portal
portal_image := $(pigen_dir)/deploy/image_$(shell date -uI)-lunacam-portal.img

$(stg_portal): $(pigen) $(LC_TOOLS)/pi-gen/prerun.sh $(shell find $(LC_TOOLS)/pi-gen/portal -type f)
	@mkdir -p $(stg_portal_dir)
	@rsync -r --delete $(LC_TOOLS)/pi-gen/portal/ $(stg_portal_dir)
	@cp $(LC_TOOLS)/pi-gen/prerun.sh $(stg_portal_dir)/prerun.sh
	@touch $(stg_portal)

$(cfg_portal): $(LC_TOOLS)/pi-gen/config-portal.sh $(stg_portal) $(cfg_common)
	@cp $(LC_TOOLS)/pi-gen/config-portal.sh $(cfg_portal)

$(portal_image): $(cfg_portal)
	@docker rm -v pigen_work || true
	@cd $(pigen_dir) && ./build-docker.sh -c config-portal

portal-image: $(portal_image)
