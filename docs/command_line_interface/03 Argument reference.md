<!--
    SPDX-FileCopyrightText: 2024-2026 Simon Repp
    SPDX-License-Identifier: CC0-1.0
-->

# Argument reference

**For the latest and most complete information run `hyper8 --help`**

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

  Directly build the site without starting the editor

- `--build-dir <BUILD_DIR>`

  The path to which the files for the site are written.

  **WARNING: This directory is WIPED during the build process.**<br>
  Please pay very close attention where you point this to.

  (Default location: .hyper8_build/ inside the site directory).

- `--cache-dir <CACHE_DIR>`

  The path at which all computational results (computed metadata, resized images, temporary encoding artifacts) are stored.

  **WARNING: This directory may be WIPED during the build process.**<br>
  Please pay very close attention where you point this to.

  (Default location: .hyper8_cache/ inside the site directory)

- `--ip <IP>`

  Manually set the ip address used by the editor or preview server (otherwise default to localhost)

- `--non-interactive`

  Disable interactive command line prompts (sync confirmations, password prompts).

- `--port <PORT>`

  Manually set the port used by the web editor (otherwise automatically choose an available port)

- `--preview`, `-p`

  Open the site in the browser after building. Use with `--build`.

- `--workers <COUNT>`

  Set how many background workers should be run. The count can be no less than 2 - one worker for fast jobs (video analysis, thumbnails), one worker for slow jobs (audio/video transcoding).

- `--version`, `-V`

  Print version

## Sync options

- `--check-ftp`

  Run a check to verify that the FTP/FTPS sync configuration and server setup is correct.

- `--check-ssh`

  Run a check to verify that the SSH sync configuration and server setup is correct.

- `--simulate`

  Simulate synchronization by merely printing, but not executing, file synchronization actions. Use with `--sync-ftp`/`--sync-ssh`.

- `--sync-ftp`

  Synchronize the build directory to the configured remote server over FTP/FTPS. Use with `--build` or on its own.

- `--sync-ssh`

  Synchronize the build directory to the configured remote server over SSH. Use with `--build` or on its own.
