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

daemon: $(pseudo)
	@$(MAKE) --no-print-directory -C daemon

run-daemon: $(pseudo)
	@$(MAKE) --no-print-directory -C daemon run

clean-daemon: $(pseudo)
	@$(MAKE) --no-print-directory -C daemon clean

install-daemon: $(pseudo)
	@$(MAKE) --no-print-directory -C daemon install

.PHONY: daemon run-daemon clean-daemon deploy-daemon



####################################################################################################
# Portal
####################################################################################################

portal: $(pseudo)
	@$(MAKE) --no-print-directory -C portal

run-portal: $(pseudo)
	@$(MAKE) --no-print-directory -C portal run

clean-portal: $(pseudo)
	@$(MAKE) --no-print-directory -C portal clean

install-portal: $(pseudo)
	@$(MAKE) --no-print-directory -C portal install

.PHONY: portal run-portal clean-portal deploy-portal



####################################################################################################
# The imagebuild Docker image is used to build a bootable SD card image
####################################################################################################

imagebuild_ctx := $(pseudo)/imagebuild-ctx
imagebuild_ctx_dir := $(LC_BUILD)/imagebuild

$(imagebuild_ctx): $(LC_TOOLS)/imagebuild/aur-install.sh $(pseudo)
	@$(PAL_CREATE_DIR) $(imagebuild_ctx_dir)
	@cp $(LC_TOOLS)/imagebuild/aur-install.sh $(imagebuild_ctx_dir)/aur-install.sh
	@$(call PAL_TOUCH_FILE,$(imagebuild_ctx))

imagebuild := $(pseudo)/imagebuild
imagebuild_dir := $(repo)/tools/imagebuild
imagebuild_img := lc-imagebuild

$(imagebuild): $(LC_TOOLS)/imagebuild/Dockerfile $(imagebuild_ctx) $(pseudo)
	@docker build -t $(imagebuild_img) -f $(LC_TOOLS)/imagebuild/Dockerfile $(imagebuild_ctx_dir)
	@$(call PAL_TOUCH_FILE,$(imagebuild))

imagebuild: $(imagebuild)

imagebuild_cmd := docker run
imagebuild_cmd += -it --rm --privileged
imagebuild_cmd += --tmpfs /tmp
imagebuild_cmd += -v $(repo):/source
imagebuild_cmd += -w /source
imagebuild_cmd += $(imagebuild_img)
imagebuild_cmd += /source/tools/imagebuild/build.sh

daemon_image := $(build)/images/lc-daemon.img
portal_image := $(build)/images/lc-portal.img

$(daemon_image): $(crossbuild_daemon) $(imagebuild)
	@$(imagebuild_cmd) daemon

daemon-image: $(daemon_image)

$(portal_image): $(crossbuild_portal) $(imagebuild)
	@$(imagebuild_cmd) portal

portal-image: $(portal_image)

clean-imagebuild:
	@docker image rm -f $(imagebuild_img)
	@$(call PAL_RM,$(imagebuild))

.PHONY: imagebuild daemon-image portal-image clean-imagebuild


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
