<!--
    SPDX-FileCopyrightText: 2024-2025 Simon Repp
    SPDX-License-Identifier: CC0-1.0
-->

# Installation

On debian-based Linux hyper8 can already be installed via packages (amd64). On
Arch Linux and Manjaro faircamp is available through the AUR. openSUSE On macOS
faircamp can be installed using Homebrew. On other platforms Hyper 8 needs to
be built from source for now (which is not very complicated though), see the instructions
at the bottom of the page.

## Linux

**Arch Linux, Manjaro**

Install [hyper8](https://aur.archlinux.org/packages/hyper8) from the [AUR](https://aur.archlinux.org/)

**Debian 12, elementary OS 8, Linux Mint 22, Ubuntu 23.04 - 24.10**

Install the [latest debian 12 package](https://simonrepp.com/hyper8/packages/hyper8_0.17.1-1+deb12_amd64.deb) (amd64)

**openSUSE**

Install the [hyper8 package](https://build.opensuse.org/package/show/home:edogawa/hyper8)

## macOS

With [Homebrew](https://brew.sh/), run these two lines to install hyper8 from [homebrew-hyper8](https://codeberg.org/simonrepp/homebrew-hyper8):

```
brew tap simonrepp/homebrew-hyper8 https://codeberg.org/simonrepp/homebrew-hyper8
brew install hyper8
```

## Other platforms

Follow these general build instructions below:

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
