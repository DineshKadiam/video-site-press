<!--
    SPDX-FileCopyrightText: 2024-2025 Simon Repp
    SPDX-License-Identifier: CC0-1.0
-->

# Command-line arguments

General usage is:

```
hyper8 [OPTIONS] <SITE_DIR>
```

Consult `hyper8 --help` for the most authoritative and up-to-date information on available arguments.

That said here's a glimpse at some particularly interesting ones:

- `--build` Directly builds the site without starting the editor
- `--build-dir <BUILD_DIR>` Override build directory (default is .hyper8_build/ inside the catalog directory). **Pay close attention where you point this to - this directory is wiped during the build process (!)**
- `--cache-dir <CACHE_DIR>` Override cache directory (default is .hyper8_cache/ inside the catalog directory). **Pay close attention where you point this to - this directory is wiped during the build process (!)**
- `--ip <IP>` Manually sets the ip address used by the editor or preview server (otherwise defaults to localhost)
- `--port <PORT>` Manually sets the port used by the editor or preview server (otherwise hyper8 chooses an available port on its own)
- `--preview` Use in conjunction with `--build` to immediately open the site in the browser after the build is complete
- `--workers <COUNT>` Set how many background workers should be run. The count can be no less than 2 (one worker for fast jobs - video analysis, thumbnails, one worker for slow jobs - video transcoding).
