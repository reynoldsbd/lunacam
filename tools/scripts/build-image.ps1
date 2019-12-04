#!/usr/bin/pwsh

param (
    [ValidateSet(
        "Full",
        "PortalOnly",
        "CameraOnly"
    )]
    [string]
    $Variant = "Full",

    [switch]
    $NoZip
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
$null = New-Item -Type Directory $templateDir
Copy-Item -Recurse $repoDir/templates/* $templateDir

Copy-Item $pigenDir/utils.sh $pigenBuildDir/utils.sh
Copy-Item $pigenDir/config.sh $pigenBuildDir/config

$stageList = "stage0 stage1 stage2 common"

if ($Variant -ne "CameraOnly") {

    prepareStage "portal"
    $portalDir = "$pigenBuildDir/portal"

    & "$PSScriptRoot/build-css.ps1"

    $staticDir = "$portalDir/01-portal/files/static"
    Remove-Item -Recurse -Force $staticDir -ErrorAction Ignore
    $null = New-Item -Type Directory $staticDir
    Copy-Item -Recurse $buildDir/css $staticDir/css
    Copy-Item -Recurse $repoDir/client/js $staticDir/js

    $stageList += " portal"
}

if ($Variant -ne "PortalOnly") {

    prepareStage "stream"

    $stageList += " stream"
}

prepareStage "finalize"
$stageList += " finalize"

$imgSuffix = switch ($Variant) {
    "CameraOnly" { "-camera" }
    "PortalOnly" { "-portal" }
}
$deployZip = if ($NoZip) { "0" } else { "1" }
@"
export STAGE_LIST="$stageList"
export LC_IMG_SUFFIX="$imgSuffix"
export DEPLOY_ZIP="$deployZip"
"@ >> $pigenBuildDir/config

Push-Location $pigenBuildDir
Write-Host "building raspbian image"
$Env:CONTAINER_NAME = "lc_pigen_$Variant"
$Env:PRESERVE_CONTAINER = "1"
$Env:CONTINUE = "1"
./build-docker.sh -c config
Pop-Location
