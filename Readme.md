Source code for LunaCam.

# Building LunaCam

Build process currently designed to run on Debian Sid, running under WSL. Building requires the
following programs:

### Cross Compiler for ARMv6

Install a GCC cross-compiler targeting ARMv6:

```
sudo apt gcc-arm-unknown-gnueabihf
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
