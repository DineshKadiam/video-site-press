# Installation

Alpha release packages are currently available for Arch and Debian-based Linux.
On other platforms Hyper 8 needs to be built from source, for now.

**Arch Linux, Manjaro**

Install [hyper8](https://aur.archlinux.org/packages/hyper8) from the [AUR](https://aur.archlinux.org/)

**Debian 12, Ubuntu 23.04 - 24.04**

Install the [latest debian 12 package](https://simonrepp.com/hyper8/packages/hyper8_0.12.0-1+deb12_amd64.deb)

**Building from source**

As prerequisites you need `ffmpeg` and `rust` installed
on your system, then run these commands:

```
git clone https://codeberg.org/simonrepp/hyper8.git
cd hyper8
cargo install --locked --path .
```

To uninstall:

```
cargo uninstall hyper8
```
