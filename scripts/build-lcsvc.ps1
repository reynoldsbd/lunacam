#!/usr/bin/pwsh

param (
    [ValidateSet(
        "Standalone",
        "PortalOnly",
        "CameraOnly"
    )]
    [string]
    $Mode = "Standalone"
)

$ErrorActionPreference = "Stop"

$repoDir = Split-Path $PSScriptRoot
$buildDir = Join-Path $repoDir build

# Cross-compiling binaries compatible with the Pi Zero works best with RPi's
# official (albeit OLD) toolchain
$piToolsDir = Join-Path $buildDir rpi-tools
if (!(Test-Path $piToolsDir)) {
    Write-Host "cloning rpi build tools"
    git clone --depth 1 https://github.com/raspberrypi/tools $piToolsDir
}
if (!$env:PATH.Contains($piToolsDir)) {
    $env:PATH = "$piToolsDir/arm-bcm2708/arm-linux-gnueabihf/bin" + [IO.Path]::PathSeparator + $env:PATH
}

switch ($Mode) {
    "Standalone" {
        $featureFlags = @()
    }
    "PortalOnly" {
        $featureFlags = @(
            "--no-default-features"
            "--features"
            "portal stream"
        )
    }
    "CameraOnly" {
        $featureFlags = @(
            "--no-default-features"
            "--features"
            "stream-api"
        )
    }
}

Push-Location $repoDir
cargo build                              `
    --target arm-unknown-linux-gnueabihf `
    --release                            `
    $featureFlags                        `
    --bin lcsvc
Pop-Location
