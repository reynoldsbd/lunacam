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

static_dir := $(build)/portal/static

sass_cmd := sass
sass_cmd += -I $(pkg_dir)/bulma
sass_cmd += -I $(pkg_dir)/bulma-switch/dist/css
sass_cmd += -I $(pkg_dir)/@fortawesome/fontawesome-free/scss
stylesheets := $(static_dir)/css/style.css
$(static_dir)/css/%.css: style/%.scss $(pkg)
	@$(sass_cmd) $< $@

jsfiles := $(static_dir)/js/base.js
jsfiles += $(static_dir)/js/admin/cameras.js
$(static_dir)/js/%.js: js/%.js
	@$(PAL_CREATE_DIR) $(dir $@)
	@cp $< $@

webfonts_src := $(pkg_dir)/@fortawesome/fontawesome-free/webfonts
webfonts_dest := $(static_dir)/css/fa-fonts
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
export LC_STATIC ?= $(static_dir)
export LC_TEMPLATES ?= $(PAL_CURRENT_DIR)/templates
export DATABASE_URL ?= $(build)/run/portal.db

run: portal static
	@$(PAL_CREATE_DIR) $(dir $(DATABASE_URL))
	@diesel migration run
	@cargo run -q
.PHONY: run



####################################################################################################
# Cleaning
####################################################################################################

clean:
	@cargo clean
	@$(call PAL_RM,$(DATABASE_URL) $(stylesheets) $(jsfiles) $(webfonts))
.PHONY: clean
