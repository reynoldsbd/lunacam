repo := $(abspath $(dir $(lastword $(MAKEFILE_LIST))))


.PHONY: server staging build-image build-volume image clean


clean:
	@cargo clean
	@rm -rf staging
	@rm -rf lunacam.img


####################################################################################################
# Cross-compiling the LunaCam control server binary
####################################################################################################

toolchain := $(repo)/rpi-tools/arm-bcm2708/arm-linux-gnueabihf/bin
manifest := $(repo)/Cargo.toml
server := $(repo)/target/arm-unknown-linux-gnueabihf/debug/lunacam

$(toolchain):
	@echo Downloading Raspberry Pi toolchain...
	@git clone --depth 1 https://github.com/raspberrypi/tools $(repo)/rpi-tools

export PATH := $(toolchain):$(PATH)

# Cargo may decide that $(server) doesn't need to be updated after all. The "touch" in this recipe
# updates the file's timestamp manually, which prevents make from invoking this rule if it isn't
# needed
$(server): $(manifest) $(shell find src -type f) $(toolchain)
	@cargo build --manifest-path $(manifest) --target arm-unknown-linux-gnueabihf
	@touch $(server)

server: $(server)


####################################################################################################
# The staging directory contains everything required to install LunaCam onto an Arch ARM system
####################################################################################################

staging := $(repo)/staging
templates := $(repo)/templates

staging: $(shell find $(repo)/system -type f) $(server) $(shell find $(templates) -type f)
	@mkdir -p $(staging)
	@cp -R $(repo)/system/* $(staging)/
	@mkdir -p $(staging)/root/usr/local/bin
	@cp $(server) $(staging)/root/usr/local/bin/lunacam
	@mkdir -p $(staging)/root/usr/local/share/lunacam/templates
	@cp -R $(templates)/* $(staging)/root/usr/local/share/lunacam/templates


####################################################################################################
# Deploys complete LunaCam installation to a connected Raspberry Pi
####################################################################################################

pi_user := alarm
pi_pass := alarm
pi_host := 192.168.7.3

PI_CP = sshpass -p "$(pi_pass)" scp -r $(1) $(pi_user)@$(pi_host):~/
PI_CMD := sshpass -p "$(pi_pass)" ssh $(pi_user)@$(pi_host)

deploy: staging
	@echo copying staging artifacts to Pi
	@$(call PI_CP,$(staging))
	@echo installing LunaCam
	@$(PI_CMD) sudo /home/alarm/staging/install.sh /home/alarm/staging
	@echo resetting services
	@$(PI_CMD) sudo systemctl daemon-reload
	@$(PI_CMD) sudo systemctl restart lunacam


####################################################################################################
# Building the LunaCam SD card image
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
