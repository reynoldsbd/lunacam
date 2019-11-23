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

$repoDir = Split-Path (Split-Path $PSScriptRoot)
$buildDir = Join-Path $repoDir build
$pigenDir = Join-Path (Join-Path $repoDir tools) pi-gen
$pigenBuildDir = Join-Path $buildDir pi-gen
$rustBuildDir = "$buildDir/target/arm-unknown-linux-gnueabihf/release"

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

    # Ensure all copied scripts are executable
    Get-ChildItem -Recurse $pigenBuildDir/$name |
        Where-Object Name -Match "\.sh$" |
        Select-Object -ExpandProperty FullName |
        xargs chmod +x
}

prepareStage "common"
$commonDir = "$pigenBuildDir/common"

& "$PSScriptRoot/build-lcsvc.ps1" -Variant $Variant
Copy-Item $rustBuildDir/lcsvc $commonDir/03-lcsvc/files/lcsvc

$templateDir = "$commonDir/03-lcsvc/files/templates"
Remove-Item -Recurse -Force $templateDir -ErrorAction Ignore
$null = New-Item -Type Directory $templateDir
Copy-Item $repoDir/templates $templateDir

Copy-Item $pigenDir/config.sh $pigenBuildDir/config

if (($Variant -eq "Full") -or ($Variant -eq "PortalOnly")) {
    prepareStage "portal"
    $portalDir = "$pigenBuildDir/portal"

    & "$PSScriptRoot/build-css.ps1"

    $staticDir = "$portalDir/01-portal/files/static"
    Remove-Item -Recurse -Force $staticDir -ErrorAction Ignore
    $null = New-Item -Type Directory $staticDir
    Copy-Item $buildDir/css $staticDir/css
    Copy-Item $repoDir/client/js $staticDir/js

    Copy-Item $pigenDir/config-portal.sh $pigenBuildDir/config-portal
}

if (($Variant -eq "Full") -or ($Variant -eq "CameraOnly")) {
    prepareStage "agent"

    Copy-Item $pigenDir/config-agent.sh $pigenBuildDir/config-agent
}

switch ($Variant) {
    "Full" {
        throw "full image variant not yet supported"
    }
    "PortalOnly" {
        $configName = "portal"
    }
    "CameraOnly" {
        $configName = "agent"
    }
}

Push-Location $pigenBuildDir
Write-Host "building raspbian image"
docker rm -v pigen_work
./build-docker.sh -c config-$configName
Pop-Location