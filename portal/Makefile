include $(repo)/tools/make/pal.mk
include $(repo)/tools/make/rust.mk



all: portal static
.PHONY: all



####################################################################################################
# Server binary
####################################################################################################

portal := $(call RUST_BIN_PATH,lunacam-portal)

$(portal): $(RUST_DEPS)
	@$(RUST_BUILD_CMD)
	@$(call PAL_TOUCH_FILE,$(portal))

portal: $(portal)
.PHONY: portal



####################################################################################################
# NPM Packages
####################################################################################################

pkg := $(pseudo)/pkg-portal
pkg_dir := $(build)/portal/node_modules

$(pkg): package.json yarn.lock
	@$(PAL_CREATE_DIR) $(pkg_dir)
	@yarn install --modules-folder $(pkg_dir) --silent
	@$(call PAL_TOUCH_FILE,$(pkg))



####################################################################################################
# Static files
####################################################################################################

export STATIC_DIR := $(build)/portal/static

sass_cmd := sass
sass_cmd += -I $(pkg_dir)/bulma
sass_cmd += -I $(pkg_dir)/bulma-switch/dist/css
sass_cmd += -I $(pkg_dir)/@fortawesome/fontawesome-free/scss
stylesheets := $(STATIC_DIR)/css/style.css
$(STATIC_DIR)/css/%.css: style/%.scss $(pkg)
	@$(sass_cmd) $< $@

jsfiles := $(STATIC_DIR)/js/base.js
jsfiles += $(STATIC_DIR)/js/camera.js
jsfiles += $(STATIC_DIR)/js/admin/cameras.js
$(STATIC_DIR)/js/%.js: js/%.js
	@$(PAL_CREATE_DIR) $(dir $@)
	@cp $< $@

webfonts_src := $(pkg_dir)/@fortawesome/fontawesome-free/webfonts
webfonts_dest := $(STATIC_DIR)/css/fa-fonts
webfonts := $(addprefix $(webfonts_dest)/,$(notdir $(call PAL_ENUM_DIR,$(webfonts_src))))
$(webfonts_dest)/%: $(webfonts_src)/% $(pkg)
	@$(PAL_CREATE_DIR) $(webfonts_dest)
	@cp $< $@
	@$(call PAL_TOUCH_FILE,$@)

static: $(stylesheets) $(jsfiles) $(webfonts)
.PHONY: static



####################################################################################################
# Running
####################################################################################################

export LC_LOG ?= info,lunacam_portal=debug
export LC_STATIC ?= $(STATIC_DIR)
export LC_TEMPLATES ?= $(PAL_CURRENT_DIR)/templates
export STATE_DIRECTORY ?= $(build)/run

run: portal static
	@$(PAL_CREATE_DIR) $(STATE_DIRECTORY)
	@cargo run -q
.PHONY: run



####################################################################################################
# Installation
#
# Technically, all is a prereq of install. However, this dependency is intentionally left out to
# prevent accidentally trying to cross-compile during imagebuild.
####################################################################################################

install: install.sh
	@./install.sh



####################################################################################################
# Cleaning
####################################################################################################

clean:
	@cargo clean
	@$(call PAL_RM,$(DATABASE_URL) $(stylesheets) $(jsfiles) $(webfonts))
.PHONY: clean
