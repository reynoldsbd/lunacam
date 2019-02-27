Source code for LunaCam.


# Developing

To avoid long build times and generally preserve sanity, much of LunaCam development can take place
on your local workstation using `cargo run`. Generally speaking, there are no esoteric dependencies.

**Notes:**

* Requires Rust 1.32 or greater
* Some dependency crates expect certain compilers/linkers to be available in `$PATH`. This
  requirement is usually satisfied by installing `build-essential`/`base-devel`/etc. (on Linux) or
  [Visual Studio](https://visualstudio.microsoft.com/) on Windows.
* TODO: setup *config.json*


# SD Card Image

This project includes a build system capable of preparing an SD card image pre-installed with
LunaCam.

### Cross-Compilation

Binaries for the Raspberry Pi are cross-compiled from a build host (i.e. your x86_64 workstation).
This is necessary because some models of the Raspberry Pi (e.g. the Zero W) don't have enough RAM to
build a large Rust program. (Cross compiling is also *tremendously* faster than building on the Pi's
slow ARM chip.)

Cross-compilation requires the GCC toolchain provided by Raspberry Pi, which in turn means LunaCam
SD card images may only be built from Linux.

The 

### Cross Compiler for ARMv6

Install a GCC cross-compiler targeting ARMv6:

```
sudo apt install gcc-arm-unknown-gnueabihf
```

### Docker

Follow [this guide](https://nickjanetakis.com/blog/setting-up-docker-for-windows-and-wsl-to-work-flawlessly),
adapted as follows to work on Sid under WSL:

* Use Docker's PGP key for Debian, rather than Ubuntu.
  ```
  curl -fsSL https://download.docker.com/linux/debian/gpg | sudo apt-key add -
  ```
  > In practice, the Docker uses the same keys for Debian and Ubuntu, so this isn't strictly
  > necessary.
* When installing the Docker PPA, use the path for Debian/Stretch rather than Ubuntu.
  ```
  sudo add-apt-repository \
     "deb [arch=amd64] https://download.docker.com/linux/ubuntu \
     stretch \
     stable"
  ```

### Rust

[Install Rust](https://rustup.rs/). This project uses version 1.32.

Install the `arm-unknown-linux-gnueabihf` target:

```
rustup target add arm-unknown-linux-gnueabihf
```


# References

If we want to try cross-compiling from Windows:
https://medium.com/@wizofe/cross-compiling-rust-for-arm-e-g-raspberry-pi-using-any-os-11711ebfc52b
https://users.rust-lang.org/t/building-for-raspberry-from-windows-10/21648/17

Getting data into docker:
https://nickjanetakis.com/blog/docker-tip-73-connecting-to-a-remote-docker-daemon
