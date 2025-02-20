<!--
    SPDX-FileCopyrightText: 2024-2025 Simon Repp
    SPDX-License-Identifier: CC0-1.0
-->

# Installation

On debian-based Linux hyper8 can already be installed via packages (amd64). On
Arch Linux and Manjaro hyper8 is available through the AUR. On openSUSE a
package is provided through the openSUSE build service. On macOS hyper8 can
be installed using Homebrew. On other platforms Hyper 8 needs to be built
from source for now (which is fairly quick and easy though), see the
instructions at the bottom of the page.

## Linux

**Arch Linux, Manjaro**

Install [hyper8](https://aur.archlinux.org/packages/hyper8) from the [AUR](https://aur.archlinux.org/)

**Debian 12, elementary OS 8, Linux Mint 22, Ubuntu 23.04 - 24.10**

Install the [latest debian 12 package](https://simonrepp.com/hyper8/packages/hyper8_0.17.1-1+deb12_amd64.deb) (amd64)

**openSUSE**

Install the [hyper8 package](https://software.opensuse.org//download.html?project=home%3Aedogawa&package=hyper8)

## macOS

With [Homebrew](https://brew.sh/), run these two lines to install hyper8 from [homebrew-hyper8](https://codeberg.org/simonrepp/homebrew-hyper8):

```
brew tap simonrepp/homebrew-hyper8 https://codeberg.org/simonrepp/homebrew-hyper8
brew install hyper8
```

## Other platforms

Follow these general build instructions:

**Building from source**

What you need to have installed to build and run hyper8:
- ffmpeg (any somewhat recent version should work)
- Rust (version 1.80.1 or later)

**1. Install required dependencies**

Install [rust](https://rust-lang.org) on your system following the official [installation instructions](https://www.rust-lang.org/tools/install).

Install [ffmpeg](https://ffmpeg.org) on your system, see [this page](https://ffmpeg.org/download.html) for instructions for various platforms.

**2. Run the compilation/installation command**

Now you're ready to build and install hyper8 on your system.

First check out and enter the repository:

```bash
git clone https://codeberg.org/simonrepp/hyper8.git
cd hyper8
```

Now run this command:

```bash
cargo install --locked --path .
```

**Uninstalling**

If you want to uninstall hyper8 at any point, run:

```
cargo uninstall hyper8
```
