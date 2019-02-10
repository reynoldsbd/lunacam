# This script builds the LunaCam SD card image.


# TODO: use the value provided by DevOps
$sourceDir = Split-Path $PSScriptRoot

mkdir -f $sourceDir\target

docker build -t=mk-alarm-img $sourceDir\img-build

docker run -it --name img-build-container --privileged `
    -v $sourceDir\staging:/mnt `
    --tmpfs /tmp `
    mk-alarm-img
docker cp img-build-container:/alarm.img $sourceDir\target\alarm.img
docker rm img-build-container
