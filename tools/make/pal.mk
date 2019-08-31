# Platform Abstraction Layer
#
# Provides helpers for writing Makefiles that work on multiple platforms (e.g. Windows and Linux)

ifndef PAL_MK
PAL_MK :=


ifeq "$(OS)" "Windows_NT"

SHELL := powershell.exe
.SHELLFLAGS := -NoProfile

PAL_WINDOWS := _

PAL_CREATE_DIR = $$null = New-Item -Type Directory -Force

PAL_CURRENT_DIR := $(shell Get-Location | Write-Host)

PAL_ENUM_DIR = $(shell Get-ChildItem $(1) -File -Recurse -Name | % { "$(1)/$$_" })

_pal_space :=
_pal_space +=
_pal_comma := ,
PAL_RM = Remove-Item $(subst $(_pal_space),$(_pal_comma),$(strip $(1)))
PAL_RM += -Recurse -Force -ErrorAction SilentlyContinue; exit 0

PAL_TOUCH_FILE =                             \
if (Test-Path $(1)) {                        \
    (Get-Item $(1)).LastWriteTime = Get-Date \
} else {                                     \
    $$null = New-Item -Type File $(1)        \
}



else

PAL_CREATE_DIR = mkdir -p

PAL_CURRENT_DIR := $(shell pwd)

PAL_ENUM_DIR = $(shell find $(1) -type f)

PAL_RM = rm -rf $(1)

PAL_TOUCH_FILE = touch $(1)

endif

endif
