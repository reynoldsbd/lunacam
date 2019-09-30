# TODO: remove non-LC_* vars
export repo := $(abspath $(dir $(lastword $(MAKEFILE_LIST))))
export build := $(repo)/build
export LC_SOURCE := $(abspath $(dir $(lastword $(MAKEFILE_LIST))))
export LC_BUILD := $(LC_SOURCE)/build
export LC_TOOLS := $(LC_SOURCE)/tools

include $(LC_TOOLS)/make/pal.mk



FORCE:



####################################################################################################
# Files under $(pseudo) represent non-file dependencies (e.g. Docker images)
####################################################################################################

pseudo := $(build)/pseudo
$(pseudo):
	@$(PAL_CREATE_DIR) $(pseudo)



####################################################################################################
# The crossbuild Docker image is used to cross-compile binaries for the Raspberry Pi
####################################################################################################

crossbuild := $(pseudo)/crossbuild
crossbuild_dir := $(repo)/tools/crossbuild
crossbuild_img := lc-crossbuild
crossbuild_cache := lc-crossbuild-cache

$(crossbuild): $(call PAL_ENUM_DIR,$(crossbuild_dir)) | $(pseudo)
	@docker build -t $(crossbuild_img) $(crossbuild_dir)
	@$(call PAL_TOUCH_FILE,$(crossbuild))

pi_triple := arm-unknown-linux-gnueabihf

crossbuild_cmd := docker run
crossbuild_cmd += -it
crossbuild_cmd += --rm
crossbuild_cmd += -v $(repo):/source
crossbuild_cmd += -v $(crossbuild_cache):/root/.cargo
crossbuild_cmd += -w /source
crossbuild_cmd += $(crossbuild_img)
crossbuild_cmd += make --no-print-directory --directory /source
crossbuild_cmd += RUST_TARGET=$(pi_triple) RUST_PROFILE=release

crossbuild_out_dir := $(build)/target/$(pi_triple)/release
crossbuild_daemon := $(crossbuild_out_dir)/lunacam-daemon
crossbuild_portal := $(crossbuild_out_dir)/lunacam-portal

$(crossbuild_daemon): $(crossbuild) FORCE
	@$(crossbuild_cmd) daemon
	@$(call PAL_TOUCH_FILE,$(crossbuild_daemon))

$(crossbuild_portal): $(crossbuild) FORCE
	@$(crossbuild_cmd) portal
	@$(call PAL_TOUCH_FILE,$(crossbuild_portal))

crossbuild: $(crossbuild_daemon) $(crossbuild_portal)

clean-crossbuild:
	@$(call PAL_RM,$(crossbuild_out_dir))
	@docker image rm -f $(crossbuild_img)
	@docker volume rm -f $(crossbuild_cache)
	@$(call PAL_RM,$(crossbuild))

.PHONY: crossbuild clean-crossbuild



####################################################################################################
# Daemon
####################################################################################################

daemon: | $(pseudo)
	@$(MAKE) --no-print-directory -C daemon

run-daemon: | $(pseudo)
	@$(MAKE) --no-print-directory -C daemon run

clean-daemon: | $(pseudo)
	@$(MAKE) --no-print-directory -C daemon clean

install-daemon: | $(pseudo)
	@$(MAKE) --no-print-directory -C daemon install

deploy-daemon: $(crossbuild_daemon) | $(pseudo)
	@$(MAKE) --no-print-directory -C daemon deploy RUST_TARGET=$(pi_triple) RUST_PROFILE=release

.PHONY: daemon run-daemon clean-daemon deploy-daemon



####################################################################################################
# Portal
####################################################################################################

portal: | $(pseudo)
	@$(MAKE) --no-print-directory -C portal

run-portal: | $(pseudo)
	@$(MAKE) --no-print-directory -C portal run

clean-portal: | $(pseudo)
	@$(MAKE) --no-print-directory -C portal clean

install-portal: | $(pseudo)
	@$(MAKE) --no-print-directory -C portal install

deploy-portal: $(crossbuild_portal) | $(pseudo)
	@$(MAKE) --no-print-directory -C portal deploy RUST_TARGET=$(pi_triple) RUST_PROFILE=release

.PHONY: portal run-portal clean-portal deploy-portal



####################################################################################################
# Custom Raspbian image with LunaCam preconfigured
####################################################################################################

pigen := $(pseudo)/pi-gen
pigen_dir := $(LC_BUILD)/pi-gen

# TODO: checkout specific commit
$(pigen): | $(pseudo)
	@git clone --depth 1 https://github.com/RPi-Distro/pi-gen $(pigen_dir)
	@touch $(pigen_dir)/stage2/SKIP_IMAGES
	@$(call PAL_TOUCH_FILE,$(pigen))



stg_common := $(pseudo)/stg-common
stg_common_dir := $(pigen_dir)/common
cfg_common := $(pigen_dir)/config

$(stg_common): $(pigen) $(LC_TOOLS)/pi-gen/prerun.sh $(call PAL_ENUM_DIR,$(LC_TOOLS)/pi-gen/common)
	@$(PAL_CREATE_DIR) $(stg_common_dir)
	@rsync -r --delete $(LC_TOOLS)/pi-gen/common/ $(stg_common_dir)
	@cp $(LC_TOOLS)/pi-gen/prerun.sh $(stg_common_dir)/prerun.sh
	@$(call PAL_TOUCH_FILE,$(stg_common))

$(cfg_common): $(LC_TOOLS)/pi-gen/config.sh $(stg_common)
	@cp $(LC_TOOLS)/pi-gen/config.sh $(cfg_common)



stg_agent := $(pseudo)/stg-agent
stg_agent_dir := $(pigen_dir)/agent
cfg_agent := $(pigen_dir)/config-agent
agent_image := $(pigen_dir)/deploy/image_$(shell date -uI)-lunacam-agent.img

$(stg_agent): $(pigen) $(LC_TOOLS)/pi-gen/prerun.sh $(call PAL_ENUM_DIR,$(LC_TOOLS)/pi-gen/agent)
	@$(PAL_CREATE_DIR) $(stg_agent_dir)
	@rsync -r --delete $(LC_TOOLS)/pi-gen/agent/ $(stg_agent_dir)
	@cp $(LC_TOOLS)/pi-gen/prerun.sh $(stg_agent_dir)/prerun.sh
	@$(call PAL_TOUCH_FILE,$(stg_agent))

$(cfg_agent): $(LC_TOOLS)/pi-gen/config-agent.sh $(stg_agent) $(cfg_common)
	@cp $(LC_TOOLS)/pi-gen/config-agent.sh $(cfg_agent)

$(agent_image): $(cfg_agent)
	@cd $(pigen_dir) && ./build-docker.sh -c config-agent

agent-image: $(agent_image)

modules-load=dwc2,g_ether

stg_portal := $(pseudo)/stg-portal
stg_portal_dir := $(pigen_dir)/portal
cfg_portal := $(pigen_dir)/config-portal
portal_image := $(pigen_dir)/deploy/image_$(shell date -uI)-lunacam-portal.img

$(stg_portal): $(pigen) $(LC_TOOLS)/pi-gen/prerun.sh $(call PAL_ENUM_DIR,$(LC_TOOLS)/pi-gen/portal)
	@$(PAL_CREATE_DIR) $(stg_portal_dir)
	@rsync -r --delete $(LC_TOOLS)/pi-gen/portal/ $(stg_portal_dir)
	@cp $(LC_TOOLS)/pi-gen/prerun.sh $(stg_portal_dir)/prerun.sh
	@$(call PAL_TOUCH_FILE,$(stg_portal))

$(cfg_portal): $(LC_TOOLS)/pi-gen/config-portal.sh $(stg_portal) $(cfg_common)
	@cp $(LC_TOOLS)/pi-gen/config-portal.sh $(cfg_portal)

$(portal_image): $(cfg_portal)
	@cd $(pigen_dir) && ./build-docker.sh -c config-portal

portal-image: $(portal_image)



####################################################################################################
# The staging directory contains everything required to install LunaCam onto an Arch ARM system.
#
# Primarily, this consists of an overlay of the root filesystem. The filesystem structure under
# $(stg)/root will be installed into the Pi's root filesystem.
#
# $(stg) also includes an install script responsible for installing the root overlay and performing
# any necessary followup operations (like enabling services).
####################################################################################################

# stg := $(repo)/.staging
# stg_target := $(target_dir)/stg
# templates := $(repo)/templates

# $(stg_target): \
# 		$(shell find $(repo)/system -type f) \
# 		$(srv) \
# 		$(static_target) \
# 		$(shell find $(templates) -type f)
# 	@echo building staging directory
# 	@mkdir -p $(stg)
# 	@cp -Rp $(repo)/system/* $(stg)/
# 	@mkdir -p $(stg)/root/usr/local/bin
# 	@cp $(srv) $(stg)/root/usr/local/bin/lunacam
# 	@mkdir -p $(stg)/root/usr/local/share/lunacam/static
# 	@cp -R $(static)/* $(stg)/root/usr/local/share/lunacam/static
# 	@mkdir -p $(stg)/root/usr/local/share/lunacam/templates
# 	@cp -R $(templates)/* $(stg)/root/usr/local/share/lunacam/templates
# 	@touch $(stg_target)

# stg: $(stg_target)
# .PHONY: stg

# clean-stg:
# 	@echo cleaning staging directory
# 	@rm -rf $(stg)
# 	@rm -rf $(stg_target)
# .PHONY: clean-stg
# clean_targets += clean-stg
