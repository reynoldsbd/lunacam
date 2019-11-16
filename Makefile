repo := $(abspath $(dir $(lastword $(MAKEFILE_LIST))))
build ?= $(repo)/build



####################################################################################################
# Files under $(pseudo) represent non-file dependencies (e.g. Docker images)
####################################################################################################

pseudo := $(build)/pseudo
$(pseudo):
	@mkdir -p $(pseudo)



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
	@rsync -rp --delete $(pigen_dir)/common/ $(stg_common_dir)
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
	@rsync -rp --delete $(pigen_dir)/agent/ $(stg_agent_dir)
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
	@rsync -rp --delete $(pigen_dir)/portal/ $(stg_portal_dir)
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
