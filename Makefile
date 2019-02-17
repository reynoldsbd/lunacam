server-bin := target/arm-unknown-linux-gnueabihf/release/lunacam


.PHONY: server build-image pi-image

# Builds the LunaCam server binary
server: $(server-bin) $(shell find templates -type f)
	@cargo build --release --target arm-unknown-linux-gnueabihf

# Bundles artifacts needed to build the SD card image into a Docker volume
build-volume: server $(shell find pi-image -type f)
	@docker volume rm lunacam-build 2> /dev/null || true
	@docker volume create --name lunacam-build > /dev/null
	@docker rm copier 2> /dev/null || true
	@docker run -v lunacam-build:/data --name copier busybox true > /dev/null
	@docker cp $(server-bin) copier:/data
	@docker cp ./templates copier:/data
	@docker cp ./pi-image/. copier:/data

# Creates the Docker image used to build LunaCam's customized Arch ARM image
build-image: $(shell find build-image -type f)
	@docker build -f ./build-image/Dockerfile -t lunacam-build ./build-image

# Builds the LunaCam SD card image
pi-image: build-image build-volume
	@docker rm lunacam-build 2> /dev/null || true
	@docker run -it --privileged --name lunacam-build \
		-v lunacam-build:/artifacts \
		lunacam-build
	@docker cp lunacam-build:/alarm.img ./lunacam.img
