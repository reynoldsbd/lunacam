repo := $(abspath $(dir $(lastword $(MAKEFILE_LIST))))
build := $(repo)/build



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
# Files under $(pseudo) represent non-file dependencies (e.g. Docker images)
####################################################################################################

pseudo := $(build)/pseudo
$(pseudo):
	@mkdir -p $(pseudo)



####################################################################################################
# The crossbuild Docker image is used to cross-compile binaries for the Raspberry Pi
####################################################################################################

crossbuild := $(pseudo)/crossbuild
crossbuild_dir := $(repo)/tools/crossbuild
crossbuild_img := lc-crossbuild
crossbuild_cache := lc-crossbuild-cache
crossbuild_cmd := docker run -it --rm   \
	-v $(repo):/source                  \
	-v $(crossbuild_cache):/root/.cargo \
	-w /source                          \
	$(crossbuild_img)

$(crossbuild): $(shell find $(crossbuild_dir) -type f) $(pseudo)
	@docker build -t $(crossbuild_img) $(crossbuild_dir)
	@touch $(crossbuild)

pi_triple := arm-unknown-linux-gnueabihf
crossbuild_out_dir := $(build)/target/$(pi_triple)/release
crossbuild_daemon := $(crossbuild_out_dir)/lunacam-daemon

$(crossbuild_daemon): $(crossbuild)
	@$(crossbuild_cmd) make  \
		--no-print-directory \
		--directory /source  \
		target=$(pi_triple)  \
		profile=release      \
		daemon

crossbuild: $(crossbuild_daemon)

clean-crossbuild:
	@$(crossbuild_cmd) cargo clean --target $(pi_triple) --release
	@docker image rm -f $(crossbuild_img) 2> /dev/null
	@docker volume rm $(crossbuild_cache)
	@rm -rf $(crossbuild)

.PHONY: crossbuild clean-crossbuild



####################################################################################################
# Cross-compiling the LunaCam control server binary
####################################################################################################

srv_manifest := $(repo)/Cargo.toml
srv := $(repo)/target/arm-unknown-linux-gnueabihf/release/lunacam
xc_cache_vol_name := lunacam-xc-cache

$(srv): $(shell find $(repo)/src -type f) $(srv_manifest) $(xc_img_target)
	@echo building server binary
	@docker run -it --rm -v $(repo):/source -v $(xc_cache_vol_name):/root/.cargo -w /source \
		$(xc_img_name) cargo build --target arm-unknown-linux-gnueabihf --release
	@touch $(srv)

srv: $(srv)
.PHONY: srv

clean-srv:
	@echo cleaning server binary
	@cargo clean
.PHONY: clean-srv
clean_targets += clean-srv


####################################################################################################
# Static files (like CSS stylesheets)
####################################################################################################

static := $(repo)/.static
static_target := $(target_dir)/static
style := $(repo)/style
js := $(repo)/js

$(static_target): $(shell find $(style) -type f) $(shell find $(js) -type f) $(target_dir)
	@echo building static resources
	@sass --source-map-urls=absolute $(style):$(static)
	@cp -R $(js) $(static)
	@touch $(static_target)

static: $(static_target)
.PHONY: static

clean-static:
	@echo cleaning static resources
	@rm -rf $(static)
	@rm -rf $(static_target)
.PHONY: clean-static
clean_targets += clean-static


####################################################################################################
# The staging directory contains everything required to install LunaCam onto an Arch ARM system.
#
# Primarily, this consists of an overlay of the root filesystem. The filesystem structure under
# $(stg)/root will be installed into the Pi's root filesystem.
#
# $(stg) also includes an install script responsible for installing the root overlay and performing
# any necessary followup operations (like enabling services).
####################################################################################################

stg := $(repo)/.staging
stg_target := $(target_dir)/stg
templates := $(repo)/templates

$(stg_target): \
		$(shell find $(repo)/system -type f) \
		$(srv) \
		$(static_target) \
		$(shell find $(templates) -type f)
	@echo building staging directory
	@mkdir -p $(stg)
	@cp -Rp $(repo)/system/* $(stg)/
	@mkdir -p $(stg)/root/usr/local/bin
	@cp $(srv) $(stg)/root/usr/local/bin/lunacam
	@mkdir -p $(stg)/root/usr/local/share/lunacam/static
	@cp -R $(static)/* $(stg)/root/usr/local/share/lunacam/static
	@mkdir -p $(stg)/root/usr/local/share/lunacam/templates
	@cp -R $(templates)/* $(stg)/root/usr/local/share/lunacam/templates
	@touch $(stg_target)

stg: $(stg_target)
.PHONY: stg

clean-stg:
	@echo cleaning staging directory
	@rm -rf $(stg)
	@rm -rf $(stg_target)
.PHONY: clean-stg
clean_targets += clean-stg


####################################################################################################
# Docker image used to prepare the SD card image containing LunaCam
####################################################################################################

sd_dir := $(repo)/build/sd
sd_img_name := lunacam-sd
sd_img_target := $(target_dir)/sd_img

$(sd_img_target): $(shell find $(sd_dir) -type f) $(target_dir)
	@echo building SD card builder image
	@docker build -t $(sd_img_name) $(sd_dir)
	@touch $(sd_img_target)

sd-img: $(sd_img_target)
.PHONY: sd-img

clean-sd-img:
	@echo cleaning SD card builder image
	@docker image rm -f $(sd_img_name) 2> /dev/null
	@rm -rf $(sd_img_target)
.PHONY: clean-sd-img
clean_targets += clean-sd-img


####################################################################################################
# Builds the LunaCam SD card image
####################################################################################################

sd := $(repo)/lunacam.img
sd_ctr_name := lunacam-sd

$(sd): $(stg_target) $(sd_img_target)
	@docker run -it --rm --privileged --tmpfs /tmp -v $(stg):/mnt -v $(repo):/out \
		--name $(sd_ctr_name) $(sd_img_name)

sd: $(sd)
.PHONY: sd

clean-sd:
	@echo cleaning SD card image
	@rm -rf $(sd)
.PHONY: clean-sd
clean_targets += clean-sd


####################################################################################################
# Deploys complete LunaCam installation to a connected Raspberry Pi
#
# For this to work, you need to (1) configure a user account on the Pi using the same username as on
# your workstation, (2) setup SSH keys and ssh-agent, and (3) setup passwordless sudo on the Pi.
####################################################################################################

pi_host ?= lunacam-dev

PI_CP = scp -r $(1) $(pi_host):
PI_CMD := ssh $(pi_host)

deploy: $(stg_target)
	@echo copying staging artifacts to pi
	@$(call PI_CP,$(stg))
	@echo installing LunaCam
	@$(PI_CMD) sudo ./$(notdir $(stg))/install.sh ./$(notdir $(stg))
	@echo resetting services
	@$(PI_CMD) sudo systemctl daemon-reload
	@$(PI_CMD) sudo systemctl restart lunacam
.PHONY: deploy


####################################################################################################
# Runs LunaCam on the local machine
####################################################################################################

run: $(shell find $(repo)/src -type f) $(srv_manifest) $(static_target)
	@cargo run -- config.json
.PHONY: run


####################################################################################################
# Cleanup
####################################################################################################

clean: $(clean_targets)
.PHONY: clean
