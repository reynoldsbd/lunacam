repo := $(abspath $(dir $(lastword $(MAKEFILE_LIST))))


####################################################################################################
# Files in .targets are used to represent non-file dependencies, like Docker objects. This allows
# make to correctly track such dependencies and run their recipes only when needed.
####################################################################################################

target_dir := $(repo)/.targets

$(target_dir):
	@mkdir -p $(target_dir)


####################################################################################################
# Docker image used to cross-compile ("xc") binaries for the Raspberry Pi
#
# TODO: store CARGO_HOME in a persisted volume (otherwise, Cargo's cache is always empty)
####################################################################################################

xc_dir := $(repo)/build/xc
xc_img_name := lunacam-xc
xc_img_target := $(target_dir)/xc_img

$(xc_img_target): $(shell find $(xc_dir) -type f) $(target_dir)
	@docker build -t $(xc_img_name) $(xc_dir)
	@touch $(xc_img_target)

####################################################################################################
# Cross-compiling the LunaCam control server binary
####################################################################################################

srv_manifest := $(repo)/Cargo.toml
srv := $(repo)/target/arm-unknown-linux-gnueabihf/debug/lunacam

$(srv): $(shell find $(repo)/src -type f) $(srv_manifest) $(xc_img_target)
	@docker run -it --rm -v $(repo):/source -w /source $(xc_img_name) \
		cargo build --target arm-unknown-linux-gnueabihf
	@touch $(srv)


####################################################################################################
# Static files (like CSS stylesheets)
####################################################################################################

static := $(repo)/.static
static_target := $(target_dir)/static
style := $(repo)/style

$(static_target): $(shell find $(style) -type f)
	@sass $(style):$(static)
	@touch $(static_target)


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
	@cp -R $(repo)/system/* $(stg)/
	@mkdir -p $(stg)/root/usr/local/bin
	@cp $(srv) $(stg)/root/usr/local/bin/lunacam
	@mkdir -p $(stg)/root/usr/local/share/lunacam/static
	@cp -R $(static)/* $(stg)/root/usr/local/share/lunacam/static
	@mkdir -p $(stg)/root/usr/local/share/lunacam/templates
	@cp -R $(templates)/* $(stg)/root/usr/local/share/lunacam/templates
	@touch $(stg_target)

stg: $(stg_target)
.PHONY: stg


####################################################################################################
# Deploys complete LunaCam installation to a connected Raspberry Pi
####################################################################################################

pi_host := lunacam.local

PI_CP = scp -r $(1) $(pi_host):~/
PI_CMD := ssh $(pi_host)

deploy: $(stg_target)
	@echo copying staging artifacts to pi
	@$(call PI_CP,$(stg))
	@echo installing LunaCam
	@$(PI_CMD) sudo /home/alarm/staging/install.sh /home/alarm/staging
	@echo resetting services
	@$(PI_CMD) sudo systemctl daemon-reload
	@$(PI_CMD) sudo systemctl restart lunacam
.PHONY: deploy


####################################################################################################
# Building the LunaCam SD card image
# TODO: this section needs a rewrite
#
# This is a 3-step process:
# 1. "build-image" produces a Docker image with an environment suitable for preparing the SD card.
#    This approach makes it easy to consistently build images on various platforms.
# 2. "build-volume" prepares a Docker volume with artifacts that will end up on the SD card.
# 3. "image" runs the Docker image from step 1 with the volume prepared in step 2 to build and
#    initialize the final image.
####################################################################################################

build-image: $(shell find build-image -type f)
	@docker build -f ./build-image/Dockerfile -t lunacam-build ./build-image

build-volume: server $(shell find system -type f)
	@docker volume rm lunacam-build 2> /dev/null || true
	@docker volume create --name lunacam-build > /dev/null
	@docker rm copier 2> /dev/null || true
	@docker run -v lunacam-build:/data --name copier busybox true > /dev/null
	@docker cp ./system/. copier:/data
	@docker cp $(server) copier:/data/root/usr/local/bin/lunacam
	@docker cp ./templates copier:/data/root/usr/local/share/lunacam

image: build-image build-volume
	@docker rm lunacam-build 2> /dev/null || true
	@docker run -it --privileged --name lunacam-build \
		-v lunacam-build:/artifacts \
		lunacam-build
	@docker cp lunacam-build:/alarm.img ./lunacam.img
