#!/usr/bin/pwsh

param (
    [ValidateSet(
        "Full",
        "PortalOnly",
        "CameraOnly"
    )]
    [string]
    $Variant = "Full"
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

switch ($Variant) {
    "Full" {
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
Write-Host "cross-compiling lcsvc binary"
cargo build                              `
    --target arm-unknown-linux-gnueabihf `
    --release                            `
    $featureFlags                        `
    --bin lcsvc
Pop-Location
