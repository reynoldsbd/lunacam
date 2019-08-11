repo := $(abspath $(dir $(dir $(lastword $(MAKEFILE_LIST)))))
curdir := $(abspath $(dir $(lastword $(MAKEFILE_LIST))))


####################################################################################################
# Server binary
####################################################################################################

portal_bin := $(repo)/target/debug/lunacam-portal
$(portal_bin):
	@cargo build

compile: $(portal_bin)
.PHONY: compile


####################################################################################################
# Static files
####################################################################################################

static_dir := $(curdir)/static

stylesheets := $(static_dir)/css/style.css
sass_cmd := sass \
	-I $(curdir)/node_modules/bulma \
	-I $(curdir)/node_modules/bulma-switch/dist/css \
	-I $(curdir)/node_modules/@fortawesome/fontawesome-free/scss
$(static_dir)/css/%.css: $(curdir)/style/%.scss
	@$(sass_cmd) $< $@

jsfiles := \
	$(static_dir)/js/base.js \
	$(static_dir)/js/admin.js
$(static_dir)/js/%.js: $(curdir)/js/%.js
	@mkdir -p $(static_dir)/js
	@cp $< $@

webfonts_src := $(curdir)/node_modules/@fortawesome/fontawesome-free/webfonts
webfonts_dest := $(static_dir)/css/fa-fonts
webfonts := $(addprefix $(webfonts_dest)/,$(notdir $(shell find $(webfonts_src) -type f)))
$(webfonts_dest)/%: $(webfonts_src)/%
	@mkdir -p $(webfonts_dest)
	@cp $< $@


static: $(stylesheets) $(jsfiles) $(webfonts)
.PHONY: static


####################################################################################################
# Running
####################################################################################################

run: compile static
	@cargo run
.PHONY: run
