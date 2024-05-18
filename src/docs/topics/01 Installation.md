# Installation

A first Debian package is available, everywhere else, Hyper 8 needs to be built from source currently.

**Debian 12, Ubuntu 23.04 - 24.04**

Install the [latest debian 12 package](https://simonrepp.com/hyper8/packages/hyper8_0.12.0-1+deb12_amd64.deb)

**Building from source**

As prerequisites you need `ffmpeg` and `rust` installed
on your system.

```
git clone https://codeberg.org/simonrepp/hyper8.git
cd hyper8
cargo install --locked --path .
```

