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
$pigenDir = Join-Path (Join-Path $repoDir tools) pi-gen
$pigenBuildDir = Join-Path $buildDir pi-gen
$rustBuildDir = "$buildDir/target/arm-unknown-linux-gnueabihf/release"

# Build prerequisites
& "$PSScriptRoot/build-lcsvc.ps1" -Variant $Variant
if (($Variant -eq "Full") -or ($Variant -eq "PortalOnly")) {
    & "$PSScriptRoot/build-css.ps1"
}

if (!(Test-Path $pigenBuildDir)) {
    Write-Host "cloning pi-gen"
    git clone --depth 1 https://github.com/RPi-Distro/pi-gen $pigenBuildDir
    $null = New-Item $pigenBuildDir/stage2/SKIP_IMAGES
}

function prepareStage {
    param (
        [string]
        $name
    )

    Write-Host "preparing stage $name"
    Remove-Item -Recurse -Force $pigenBuildDir/$name -ErrorAction Ignore
    Copy-Item -Recurse $pigenDir/$name $pigenBuildDir/$name
    Copy-Item $pigenDir/prerun.sh $pigenBuildDir/$name/prerun.sh
}

prepareStage "common"
Copy-Item $pigenDir/config.sh $pigenBuildDir/config

switch ($Variant) {
    "Full" {
        throw "full image variant not yet supported"
    }
    "PortalOnly" {
        prepareStage "portal"

        $portalDir = "$pigenBuildDir/portal"

        Copy-Item $rustBuildDir/lcsvc $portalDir/01-portal/files/lcportal

        $staticDir = "$portalDir/01-portal/files/static"
        Remove-Item -Recurse -Force $staticDir -ErrorAction Ignore
        $null = New-Item -Type Directory $staticDir
        Copy-Item $buildDir/css $staticDir/css
        Copy-Item $repoDir/client/js $staticDir/js

        $templateDir = "$portalDir/01-portal/files/templates"
        Remove-Item -Recurse -Force $templateDir -ErrorAction Ignore
        $null = New-Item -Type Directory $templateDir
        Copy-Item $repoDir/templates $templateDir

        Copy-Item $pigenDir/config-portal.sh $pigenBuildDir/config-portal

        $configName = "portal"
    }
    "CameraOnly" {
        prepareStage "agent"

        $agentDir = "$pigenBuildDir/agent"

        Copy-Item $rustBuildDir/lcsvc $agentDir/02-agent/files/lcagent

        Copy-Item $pigenDir/config-agent.sh $pigenBuildDir/config-agent

        $configName = "agent"
    }
}

Push-Location $pigenBuildDir
Write-Host "building raspbian image"
docker rm -v pigen_work
./build-docker.sh -c config-$configName
Pop-Location
