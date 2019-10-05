repo := $(abspath $(dir $(lastword $(MAKEFILE_LIST))))
build ?= $(repo)/build



####################################################################################################
# Files under $(pseudo) represent non-file dependencies (e.g. Docker images)
####################################################################################################

pseudo := $(build)/pseudo
$(pseudo):
	@mkdir -p $(pseudo)



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
cargo_deps := Cargo.toml Cargo.lock $(shell find src -type f)

agent := $(rust_dir)/lcagent

$(agent): $(cargo_deps)
	@$(cargo_cmd) --bin lcagent
	@touch $(agent)

agent: $(agent)

portal := $(rust_dir)/lcportal

$(portal): $(cargo_deps)
	@$(cargo_cmd) --bin lcportal
	@touch $(portal)

portal: $(portal)

.PHONY: agent portal



####################################################################################################
# NPM packages
####################################################################################################

npm := $(pseudo)/npm
npm_dir := $(build)/node_modules

$(npm): client/package.json client/yarn.lock
	@mkdir -p $(npm_dir)
	@yarn install --cwd client --modules-folder $(npm_dir) --silent
	@touch $(npm)

npm: $(npm)

.PHONY: npm



####################################################################################################
# Static files
####################################################################################################

static := $(build)/static



sass_cmd := sass
sass_cmd += -I $(npm_dir)/bulma
sass_cmd += -I $(npm_dir)/bulma-switch/dist/css
sass_cmd += -I $(npm_dir)/@fortawesome/fontawesome-free/scss

stylesheets := $(static)/css/style.css

$(static)/css/%.css: client/style/%.scss $(npm)
	@$(sass_cmd) $< $@



jsfiles += $(static)/js/base.js
jsfiles += $(static)/js/camera.js
jsfiles += $(static)/js/admin/cameras.js

$(static)/js/%: client/js/%
	@mkdir -p $(dir $@)
	@cp $< $@



webfonts := $(pseudo)/webfonts

$(webfonts): $(npm)
	@mkdir -p $(static)/css
	@rsync -r --delete $(npm_dir)/@fortawesome/fontawesome-free/webfonts $(static)/css
	@touch $(webfonts)



####################################################################################################
# Running
####################################################################################################

export LC_LOG ?= info,lunacam=debug
export LC_STATIC := $(static)
export LC_TEMPLATES := templates
# TODO: agent and portal should not share state directory. not sure how to express this in makefile
export STATE_DIRECTORY := $(build)/run

run-agent: $(agent)
	@mkdir -p $(STATE_DIRECTORY)
	@cargo run -q --bin lcagent

run-portal: $(portal) $(stylesheets) $(jsfiles) $(webfonts)
	@mkdir -p $(STATE_DIRECTORY)
	@cargo run -q --bin lcportal

.PHONY: run-agent run-portal




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

FORCE:

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
# Custom Raspbian image with LunaCam preconfigured
####################################################################################################

pigen := $(pseudo)/pi-gen
pigen_dir := tools/pi-gen
pigen_build_dir := $(build)/pi-gen

# TODO: checkout specific commit
$(pigen): | $(pseudo)
	@git clone --depth 1 https://github.com/RPi-Distro/pi-gen $(pigen_build_dir)
	@touch $(pigen_build_dir)/stage2/SKIP_IMAGES
	@touch $(pigen)



stg_common := $(pseudo)/stg-common
stg_common_dir := $(pigen_build_dir)/common
cfg_common := $(pigen_build_dir)/config

$(stg_common): $(pigen) $(pigen_dir)/prerun.sh $(shell find $(pigen_dir)/common -type f)
	@mkdir -p $(stg_common_dir)
	@rsync -r --delete $(pigen_dir)/common/ $(stg_common_dir)
	@cp $(pigen_dir)/prerun.sh $(stg_common_dir)/prerun.sh
	@touch $(stg_common)

$(cfg_common): $(pigen_dir)/config.sh $(stg_common)
	@cp $(pigen_dir)/config.sh $(cfg_common)



stg_agent := $(pseudo)/stg-agent
stg_agent_dir := $(pigen_build_dir)/agent
cfg_agent := $(pigen_build_dir)/config-agent
agent_image := $(pigen_build_dir)/deploy/image_$(shell date -uI)-lunacam-agent.img

$(stg_agent): $(pigen) $(pigen_dir)/prerun.sh $(shell find $(pigen_dir)/agent -type f) $(crossbuild_agent)
	@mkdir -p $(stg_agent_dir)
	@rsync -r --delete $(pigen_dir)/agent/ $(stg_agent_dir)
	@cp $(pigen_dir)/prerun.sh $(stg_agent_dir)/prerun.sh
	@cp $(crossbuild_agent) $(stg_agent_dir)/02-agent/files/lcagent
	@touch $(stg_agent)

$(cfg_agent): $(pigen_dir)/config-agent.sh $(stg_agent) $(cfg_common)
	@cp $(pigen_dir)/config-agent.sh $(cfg_agent)

$(agent_image): $(cfg_agent)
	@docker rm -v pigen_work || true
	@cd $(pigen_build_dir) && ./build-docker.sh -c config-agent

agent-image: $(agent_image)



stg_portal := $(pseudo)/stg-portal
stg_portal_dir := $(pigen_build_dir)/portal
cfg_portal := $(pigen_build_dir)/config-portal
portal_image := $(pigen_build_dir)/deploy/image_$(shell date -uI)-lunacam-portal.img

$(stg_portal): $(pigen) $(pigen_dir)/prerun.sh $(shell find $(pigen_dir)/portal -type f) $(crossbuild_portal) $(stylesheets) $(jsfiles) $(webfonts) $(shell find templates -type f)
	@mkdir -p $(stg_portal_dir)
	@rsync -r --delete $(pigen_dir)/portal/ $(stg_portal_dir)
	@cp $(pigen_dir)/prerun.sh $(stg_portal_dir)/prerun.sh
	@cp $(crossbuild_portal) $(stg_portal_dir)/01-portal/files/lcportal
	@rsync -r $(static)/ $(stg_portal_dir)/01-portal/files/static
	@rsync -r templates/ $(stg_portal_dir)/01-portal/files/templates
	@touch $(stg_portal)

$(cfg_portal): $(pigen_dir)/config-portal.sh $(stg_portal) $(cfg_common)
	@cp $(pigen_dir)/config-portal.sh $(cfg_portal)

$(portal_image): $(cfg_portal)
	@docker rm -v pigen_work || true
	@cd $(pigen_build_dir) && ./build-docker.sh -c config-portal

portal-image: $(portal_image)
