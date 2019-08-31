# TODO: repo -> LC_SOURCE_ROOT and build -> LC_BUILD_ROOT
export repo := $(abspath $(dir $(lastword $(MAKEFILE_LIST))))
export build := $(repo)/build

include $(repo)/tools/make/pal.mk



####################################################################################################
# Daemon
####################################################################################################

daemon:
	@$(MAKE) --no-print-directory -C daemon

run-daemon:
	@$(MAKE) --no-print-directory -C daemon run

clean-daemon:
	@$(MAKE) --no-print-directory -C daemon clean

.PHONY: daemon run-daemon clean-daemon



####################################################################################################
# Portal
####################################################################################################

portal:
	@$(MAKE) --no-print-directory -C portal

run-portal:
	@$(MAKE) --no-print-directory -C portal run

clean-portal:
	@$(MAKE) --no-print-directory -C portal clean

.PHONY: portal run-portal clean-portal



####################################################################################################
# Files under $(pseudo) represent non-file dependencies (e.g. Docker images)
####################################################################################################

# TODO: pseudo -> LC_PSEUDO_DIR
export pseudo := $(build)/pseudo
$(pseudo):
	@$(PAL_CREATE_DIR) $(pseudo)



####################################################################################################
# The crossbuild Docker image is used to cross-compile binaries for the Raspberry Pi
####################################################################################################

crossbuild := $(pseudo)/crossbuild
crossbuild_dir := $(repo)/tools/crossbuild
crossbuild_img := lc-crossbuild
crossbuild_cache := lc-crossbuild-cache

$(crossbuild): $(call PAL_ENUM_DIR,$(crossbuild_dir)) $(pseudo)
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

$(crossbuild_daemon): $(crossbuild)
	@$(crossbuild_cmd) daemon

$(crossbuild_portal): $(crossbuild)
	@$(crossbuild_cmd) portal

crossbuild: $(crossbuild_daemon)

clean-crossbuild:
	@$(call PAL_RM,$(crossbuild_out_dir))
	@docker image rm -f $(crossbuild_img)
	@docker volume rm -f $(crossbuild_cache)
	@$(call PAL_RM,$(crossbuild))

.PHONY: crossbuild clean-crossbuild


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


####################################################################################################
# Docker image used to prepare the SD card image containing LunaCam
####################################################################################################

# sd_dir := $(repo)/build/sd
# sd_img_name := lunacam-sd
# sd_img_target := $(target_dir)/sd_img

# $(sd_img_target): $(shell find $(sd_dir) -type f) $(target_dir)
# 	@echo building SD card builder image
# 	@docker build -t $(sd_img_name) $(sd_dir)
# 	@touch $(sd_img_target)

# sd-img: $(sd_img_target)
# .PHONY: sd-img

# clean-sd-img:
# 	@echo cleaning SD card builder image
# 	@docker image rm -f $(sd_img_name) 2> /dev/null
# 	@rm -rf $(sd_img_target)
# .PHONY: clean-sd-img
# clean_targets += clean-sd-img


####################################################################################################
# Builds the LunaCam SD card image
####################################################################################################

# sd := $(repo)/lunacam.img
# sd_ctr_name := lunacam-sd

# $(sd): $(stg_target) $(sd_img_target)
# 	@docker run -it --rm --privileged --tmpfs /tmp -v $(stg):/mnt -v $(repo):/out \
# 		--name $(sd_ctr_name) $(sd_img_name)

# sd: $(sd)
# .PHONY: sd

# clean-sd:
# 	@echo cleaning SD card image
# 	@rm -rf $(sd)
# .PHONY: clean-sd
# clean_targets += clean-sd


####################################################################################################
# Deploys complete LunaCam installation to a connected Raspberry Pi
#
# For this to work, you need to (1) configure a user account on the Pi using the same username as on
# your workstation, (2) setup SSH keys and ssh-agent, and (3) setup passwordless sudo on the Pi.
####################################################################################################

# pi_host ?= lunacam-dev

# PI_CP = scp -r $(1) $(pi_host):
# PI_CMD := ssh $(pi_host)

# deploy: $(stg_target)
# 	@echo copying staging artifacts to pi
# 	@$(call PI_CP,$(stg))
# 	@echo installing LunaCam
# 	@$(PI_CMD) sudo ./$(notdir $(stg))/install.sh ./$(notdir $(stg))
# 	@echo resetting services
# 	@$(PI_CMD) sudo systemctl daemon-reload
# 	@$(PI_CMD) sudo systemctl restart lunacam
# .PHONY: deploy


####################################################################################################
# Runs LunaCam on the local machine
####################################################################################################

# run: $(shell find $(repo)/src -type f) $(srv_manifest) $(static_target)
# 	@cargo run -- config.json
# .PHONY: run


####################################################################################################
# Cleanup
####################################################################################################

# clean: $(clean_targets)
# .PHONY: clean
