<!--
    SPDX-FileCopyrightText: 2025-2026 Simon Repp
    SPDX-License-Identifier: CC0-1.0
-->

# Building from source

## Install prerequisites

What you need to have installed to build and run hyper8:

- **Rust (version 1.88 or later)**

  Install [rust](https://rust-lang.org) on your system following the official [installation instructions](https://www.rust-lang.org/tools/install).

- **OpenSSL (including headers)**

  This will likely already be installed on your system.

  If it isn't, your search engine will probably hint you in the right direction with the query "[your-system-name] openssl headers".
  For example, on Debian 13, the command for installation is: `apt install libssl-dev`

- **FFmpeg (any somewhat recent version, ffprobe must be included)**

  Install [ffmpeg](https://ffmpeg.org) (including the ffprobe binary) on your system, see [this page](https://ffmpeg.org/download.html) for instructions for various platforms.

- **Tauri prerequisites (can be omitted if you only want to build the hyper8 CLI)**

  If you want to build the full desktop application (in case you're unsure,
  this is probably what you want!) you need to install the
  [system dependencies](https://v2.tauri.app/start/prerequisites/#system-dependencies)
  for [Tauri](https://tauri.app) for your specific platform (Windows, Mac, Linux, etc.)
  as well as the [Tauri Command Line Interface](https://v2.tauri.app/reference/cli/).

## Get the source code

Check out and enter the repository:

```bash
git clone https://codeberg.org/simonrepp/hyper8.git
cd hyper8
```

## Choose between desktop and command line application

You can choose between two variants of Hyper 8:

- A) The full desktop application

  This includes a graphical launcher and lets you run the editor in its own
  application window. Note that this variant includes the entire CLI
  functionality as well, so you can still choose to interact with hyper8
  using only the command line whenever you like - this variant just gives you
  both options.

- B) Only the command line application

  This variant comes without the graphical *launcher*, so you will always need
  to *initially* interact with hyper8 through the command line. Note however
  that this variant still includes the graphical *editor* - contrary to the
  full desktop application it just does not run in its own window. To use the
  graphical editor, start it from the CLI, after which you can access it
  through your browser.

## A) Compiling/installing the desktop application

Inside the repository root directory run this command:

```bash
cargo tauri build
```

This procedure will output one or more native release bundles for your
platform. Open the `target/releases/bundles` directory to see them and
install whichever format suits your needs.

If you want to uninstall hyper8 at any later point, simply use your platform
or package manager's native uninstallation method.

## B) Compiling/installing the command line application

Inside the repository root directory run this command:

```bash
cargo install --locked --path cli
```

If you want to uninstall it at any point, run:

```
cargo uninstall hyper8-cli
```

## Additional build options

You can use environment variables to adjust the build outcome:

- Pass `HYPER8_VERSION` to override the version that is displayed throughout
  Hyper 8 (e.g. in the launcher, editor and under `--help`), for instance:
  `HYPER8_VERSION=2.0.0~preview cargo ...` (by default the version is taken
  from the package version specified in the root `Cargo.toml`).

- Pass `HYPER8_BUNDLED_FFMPEG` (e.g. `HYPER8_BUNDLED_FFMPEG=true cargo ...`)
  to produce a Hyper 8 binary that will always and only execute `ffmpeg` and
  `ffprobe` binaries that were placed adjacent to itself (i.e. the `hyper8`,
  `ffmpeg` and `ffprobe` binaries all need to be in the same folder when you
  run a `hyper8` binary built with `HYPER8_BUNDLED_FFMPEG`). This exists for
  building and packaging on platforms that require bundling of ffmpeg and
  ffprobe binaries for distribution (Windows, macOS).
