LunaCam is a secure and self-hosted video streaming system designed primarily
for the Raspberry Pi. With it, you can stream video from one or more cameras
over the Internet using only a web browser.

*Why "LunaCam"?*

I started this project as a way to monitor my dog, Luna, while I'm away from
home. My hope is that this project will be used for other purposes, but as of
yet I haven't found a generic (and unused) name that's quite as catchy!


# Overview

A minimal LunaCam setup consists of a single Raspberry Pi with an attached
camera module, running a specialized version of Raspbian. The Pi exposes a web
portal as the primary means of viewing/controlling video streams and
administering the overall system.

Additional video streams can be added to the system by provisioning additional
Pis with a special "camera-only" version of LunaCam's OS. These Pis can then be
added to the system using the first Pi's web portal. All cameras in the system
can be viewed from the web portal.

Video streams exposed by LunaCam are encrypted. Users must enter a username and
password before they are allowed to view any stream. The web portal provides a
basic means for administering user accounts.

A LunaCam system is intended to live behind some kind of firewall, such as a
home router. As with much other web-based, self-hosted software, accessing
LunaCam over the Internet requires port forwarding or a reverse-proxy.


# Supported Hardware

LunaCam should work on any model of the Raspberry Pi. It is optimized for the
Raspberry Pi Zero W in particular. The only (currently) supported camera is the
official camera module.

My personal recommendation is Adafruit's Pi Zero W Camera Pack:

https://www.adafruit.com/product/3414


# Getting Started

Start by downloading [the latest release](https://github.com/reynoldsbd/lunacam/releases)
of LunaCam's Raspbian-based OS. There are several variations of this OS image to
choose from:

* *lunacam-X.Y.Z.zip* - Contains a full LunaCam stack, including web UI and
  support for streaming from an attached camera module. **Choose this image if
  you're just starting out.**
* *lunacam-camera-X.Y.Z.zip* - No web UI, only support for streaming. Use this
  image when adding an additional camera to an existing LunaCam system.
* *lunacam-portal-X.Y.Z.zip* - Web UI only. Does not support streaming from a
  camera module. This image is useful for offloading the web UI workload to a
  device that does not have an attached camera.

Unzip the image, flash it to an SD card, and use it to boot your favorite
Raspberry Pi. Congratulations, LunaCam is now installed and running! However,
some additional configuration is needed before you have a usable setup.

Log in to the device using "admin" for both the username and password. If you're
using a Zero W (or any other model supporting USB OTG), note that an ethernet
gadget is configured out of the box with an IP address of 192.168.7.3.

Once connected, run `sudo raspi-config` and configure the following:

1. Change the *admin* user's default password
2. Change the default hostname
3. Connect to the network
4. Resize the root partition to fit your SD card


# Local Development

During active development, LunaCam can be compiled and run on nearly any
workstation. This makes it very easy to build and test changes locally, without
the hassle of cross-compilation or deploying bits to hardware.

Development generally requires the following tools, which should be easy to
acquire for any operating system. On Windows, the use of Ubuntu via
[WSL](https://docs.microsoft.com/en-us/windows/wsl/about) is recommended.

* [Rust](https://rustup.rs/)
* Clang version 3.9 or higher
  * *clang* package on Ubuntu
* [PowerShell 6](https://docs.microsoft.com/en-us/powershell/scripting/install/installing-powershell?view=powershell-6)
* [Yarn](https://yarnpkg.com/lang/en/docs/install/)
* [Sass](https://sass-lang.com/install)
* [diesel_cli](https://github.com/diesel-rs/diesel/tree/master/diesel_cli) (if modifying the database schema)
  * Recommend installing with `--no-default-features` and `--features "sqlite-bundled"`

## Building and Running

Start by running */toosl/scripts/build-css.ps1*, which compiles Sass stylesheets
and prepares the static CSS directory. This script should be re-run whenever
something under */client/style/* is modified.

Then use Cargo to build and run:

```shell
cargo run
```

If developing code applicable to the camera-only variant, you'll need to toggle
some feature flags:

```shell
cargo run --no-default-features --features "stream-api"
```

## Building an SD Card Image

For more thorough testing, you can build complete SD card images locally using
*/tools/scripts/build-image.ps1*. This process is only known to work on Linux
(including WSL) and requires installing some additional dependencies:

* Basic build tools and 32-bit libc headers
  * On Ubuntu, install *build-essential* and *libc6-dev-i386*
* Docker
  * If on a true Linux host, make sure to add your user account to the *docker*
    group
  * If using WSL, follow [these instructions](https://nickjanetakis.com/blog/setting-up-docker-for-windows-and-wsl-to-work-flawlessly)
