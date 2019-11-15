#!/usr/bin/pwsh

$ErrorActionPreference = "Stop"

$repoDir = Split-Path $PSScriptRoot
$clientDir = Join-Path $repoDir client
$buildDir = Join-Path $repoDir build
$npmDir = Join-Path $buildDir node_modules
$cssDir = Join-Path $buildDir css

Write-Host "installing npm packages"
yarn install --cwd $clientDir --modules-folder $npmDir --silent

$sassIncludeDirs = @(
    "$npmDir/bulma"
    "$npmDir/bulma-switch/dist/css"
    "$npmDir/@fortawesome/fontawesome-free/scss"
)

$sassArgs = @()
foreach ($dir in $sassIncludeDirs) {
    $sassArgs += "-I"
    $sassArgs += $dir
}

Write-Host "compiling stylesheets"
Remove-Item -Recurse -Force $cssDir -ErrorAction Ignore
sass $sassArgs $clientDir/style/style.scss $cssDir/style.css

Write-Host "copying webfonts"
Copy-Item -Recurse $npmDir/@fortawesome/fontawesome-free/webfonts $cssDir/webfonts
