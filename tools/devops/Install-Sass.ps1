#!/usr/bin/pwsh

<#

.SYNOPSIS
Installs the Dart-based Sass CLI tool

.PARAMETER Version
Version of Sass to install (ex: 1.23.0)

.PARAMETER Destination
Destination for Sass files

#>

param (
    [string]
    $Version,

    [string]
    $Destination
)

if (!(Test-Path $Destination)) {
    Write-Host "Creating destination directory"
    $null = New-Item -Type Directory -Force $Destination
}

Write-Host "Downloading Sass version $Version"
$sassTarball = Join-Path $Destination sass.tar.gz
$sassUrl = "https://github.com/sass/dart-sass/releases/download/$Version/dart-sass-$Version-linux-x64.tar.gz"
Invoke-WebRequest $sassUrl -OutFile $sassTarball

Write-Host "Extracting tools"
tar -xzf $sassTarball -C $destination

Write-Host "Configuring PATH"
Write-Host "##vso[task.setvariable variable=PATH]$Destination/dart-sass:${env:PATH}"
