LunaCam turns your Raspberry Pi into a streaming video camera with a user-friendly interface and
control panel.

The project is named "LunaCam" because I use it to watch my dog, Luna, while I'm away from home.


# Getting Started

To ensure a smooth and compatible installation, LunaCam requires a customized SD card image. At this
time, you must build the image yourself.

> Although it is possible to build LunaCam on macOS and Linux, these instructions are currently
> tailored to Windows.

## Dependencies

Building the LunaCam SD card image requires the following software to be installed:

* Windows 10 Pro - required for Docker
* [Docker for Windows](https://docs.docker.com/docker-for-windows/install/)
  * Enable Docker setting *"Expose daemon on tcp://localhost:2375 without TLS"*
  * If you want build to run faster, configure Docker to use more CPUs and RAM
* Ubuntu [WSL](https://docs.microsoft.com/en-us/windows/wsl/install-win10) distro
  * `sudo apt install build-essential`
  * [Docker for Linux](https://docs.docker.com/install/linux/docker-ce/ubuntu/)
    * Set *DOCKER_HOST* environment variable to *tcp://localhost:2375*
  * [Sass](https://sass-lang.com/install)

## Customization

You may customize the SD card image by adding files to the directory */system/root.local*. The
contents of this directory are copied into LunaCam's root filesystem. LunaCam is based on Arch
Linux, and many aspects of the system can be customized by simply "dropping in" configuration files.

For cases where drop-in configuration is not sufficient, you may also provide custom logic using
*/system/local.sh*. If present, this script is run at the end of the image creation process. The
script is run as *root* and **inside** the new system, so commands like `systemctl enable foo` will
have the desired effect of enabling the *foo* service on the new system.

### Network Configuration

Most of LunaCam's features can be controlled using its UI, but at the very least you should
configure the network so you can access that UI.

> This section assumes your Pi's wireless adapter is named *wlan0*. This is the case for models with
> built in adapters, such as the Raspberry Pi 3 and Zero W. If you're using a third-party adapter,
> you may need to use a different adapter name.

First, create the file */system/root.local/etc/wpa_supplicant/wpa_supplicant-wlan0.conf* with the
following contents (replacing *\<SSID>* and *\<PSK>* with appropriate values):

```
network={
  ssid="<SSID>"
  psk="<PSK>"
}
```

Then, add the following to */system/root.local/etc/systemd/network/wlan0.network*:

```
[Match]
Name=wlan0

[Network]
DHCP=yes
MulticastDNS=yes
```

Finally, create */system/local.sh* and add the following to enable WiFi at boot time:

```bash
#!/bin/bash

systemctl enable wpa_supplicant@wlan0
```

### Secrets

Both */system/root.local* and */system/local.sh* are ignored by the source control system. So, if
you are a developer working on LunaCam, you may safely store secrets here without risk of
accidentally committing them to source control.

However, remember that these files are copied into the SD card image. In other words, anybody with
access to your image will also have access to any secrets you've added as part of image
customization.

## Image Creation

Once you are finished customization, open a WSL shell to the directory containing LunaCam's source
code and run the command `make sd`. This will create a file called *lunacam.img*, which is your
customized SD card image.

This image can be flashed to an SD card and run on a Raspberry Pi in the same manner as any other SD
card image. If you're looking for an easy-to-use tool for this, check out
[Balena Etcher](https://www.balena.io/etcher/).
