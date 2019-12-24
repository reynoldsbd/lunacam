#!/usr/bin/pwsh

$ErrorActionPreference = "Stop"

if ($args.Count -lt 1) {
    throw "unexpected number of arguments"
}

$sourceDir = Resolve-Path "$PSScriptRoot/../.."
$buildDir = "$sourceDir/build"

# Rebuild stylesheets if necessary
$cssDir = "$buildDir/css"
$styleLastModified = Get-ChildItem -Recurse "$sourceDir/client/style" |
    Measure-Object -Property LastWriteTime -Maximum |
    Select-Object -ExpandProperty Maximum
$cssLastBuilt = if (Test-Path $cssDir) {
    Get-ChildItem -Recurse $cssDir |
        Measure-Object -Property LastWriteTime -Maximum |
        Select-Object -ExpandProperty Maximum
}
if (!$cssLastBuilt -or ($styleLastModified -ge $cssLastBuilt)) {
    Write-Host "Compiling CSS"
    &"$PSScriptRoot/build-css.ps1"
}


$Env:STATE_DIRECTORY = "$buildDir/run"
$Env:LC_LOG = "info,lunacam=debug"
$Env:LC_TEMPLATES = "$sourceDir/templates"

$binary = $args[0]
$arguments = $args[1..($args.Count)]
&$binary $arguments
