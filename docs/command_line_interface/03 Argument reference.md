<!--
    SPDX-FileCopyrightText: 2024-2025 Simon Repp
    SPDX-License-Identifier: CC0-1.0
-->

# Argument reference

General usage is:

```
hyper8 [OPTIONS] [SITE_DIR]
```

If you are using the **full desktop edition** of Hyper 8, calling `hyper8`
without any arguments starts the graphical launcher (and conversely, passing
any argument at all enables command line mode).

If you are using the **command line edition** of Hyper 8, the `SITE_DIR`
argument is mandatory, and not passing any argument at all will just print
the usage information.

## Options

- `--build`, `-b`

  Directly builds the site without starting the editor

- `--build-dir <BUILD_DIR>`

  The path to which the deployable, built site is written.

  **WARNING: This directory is WIPED during the build process.**<br>
  Please pay very close attention where you point this to.

  (Default location: .hyper8_build/ inside the site directory).

- `--cache-dir <CACHE_DIR>`

  The path at which all computational results (computed metadata, resized images, temporary encoding artifacts) are stored.

  **WARNING: This directory may be WIPED during the build process.**<br>
  Please pay very close attention where you point this to.

  (Default location: .hyper8_cache/ inside the site directory)

- `--deploy`, `-d`

  Directly deploys the site - only applicable in combination with `--build`

- `--ip <IP>`

  Manually sets the ip address used by the editor or preview server (otherwise defaults to localhost)

- `--port <PORT>`

  Manually sets the port used by the editor or preview server (otherwise chooses an available port on its own)

- `--preview`, `-p`

  Use in conjunction with `--build` to immediately open the site in the browser after the build is complete

- `--workers <COUNT>`

  Set how many background workers should be run. The count can be no less than 2 - one worker for fast jobs (video analysis, thumbnails), one worker for slow jobs (audio/video transcoding).

## Note on accuracy

This page will generally be kept up-to-date with Hyper 8's latest CLI usage
information, but for the most authoritative and up-to-date information it is
still recommended to just consult `hyper8 --help` for your installed
version.
