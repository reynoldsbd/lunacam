#!/bin/bash
set -e

# Installs the Dart-based Sass CLI tool

version=$1
destination=$2
# destination=$(realpath $2)

echo "version: $version"
echo "destination: $destination"
exit 1

echo "creating destination directory"
mkdir -p $destination

echo "downloading Sass version $version"
sassTarball="$destination/sass.tar.gz"
sassUrl="https://github.com/sass/dart-sass/releases/download/$version/dart-sass-$version-linux-x64.tar.gz"
wget $sassUrl -qO $sassTarball

echo "extracting"
tar -xzf $sassTarball -C $destination

echo "configuring PATH"
echo "##vso[task.setvariable variable=PATH]$destination/dart-sass:$PATH"
