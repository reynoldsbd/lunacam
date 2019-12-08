#!/usr/bin/pwsh

if ($args.Count -lt 1) {
    throw "unexpected number of arguments"
}

$sourceDir = Resolve-Path "$PSScriptRoot/../.."
$buildDir = "$sourceDir/build"

# TODO: prep CSS dir

$Env:STATE_DIRECTORY = "$buildDir/run"
$Env:LC_LOG = "info,lunacam=debug"
$Env:LC_TEMPLATES = "$sourceDir/templates"

$binary = $args[0]
$arguments = $args[1..($args.Count)]
&$binary $arguments
