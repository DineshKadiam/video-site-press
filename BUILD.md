<!--
    SPDX-FileCopyrightText: 2025 Simon Repp
    SPDX-License-Identifier: CC0-1.0
-->

# Building from source

## Install prerequisites

What you need to have installed to build and run hyper8:

- **Rust (version 1.80.1 or later)**

  Install [rust](https://rust-lang.org) on your system following the official [installation instructions](https://www.rust-lang.org/tools/install).

- **OpenSSL (including headers)**

  This will likely already be installed on your system.

  If it isn't, your search engine will probably hint you in the right direction with the query "[your-system-name] openssl headers".
  For example, on Debian 12, the command for installation is: `apt install libssl-dev`

- **FFmpeg (any somewhat recent version, ffprobe must be included)**

  Install [ffmpeg](https://ffmpeg.org) (including the ffprobe binary) on your system, see [this page](https://ffmpeg.org/download.html) for instructions for various platforms.

## Run the compilation/installation command

First check out and enter the repository:

```bash
git clone https://codeberg.org/simonrepp/hyper8.git
cd hyper8
```

Now run this command:

```bash
cargo install --locked --path .
```

## Uninstalling

If you want to uninstall hyper8 at any point, run:

```
cargo uninstall hyper8
```
