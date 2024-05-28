# Command-line arguments

General usage is:

```
hyper8 [OPTIONS] <SITE_DIR>
```

Consult `hyper8 --help` for the most authoritative and up-to-date information on available arguments.

That said here's a glimpse at some particularly interesting ones:

- `--build-dir <BUILD_DIR>` Override build directory (default is .hyper8_build/ inside the catalog directory). **Pay close attention where you point this to - this directory is wiped during the build process (!)**
- `--cache-dir <CACHE_DIR>` Override cache directory (default is .hyper8_cache/ inside the catalog directory). **Pay close attention where you point this to - this directory is wiped during the build process (!)**
- `--editor-language <CODE>` Translate the editor interface to the given language. This is only rudimentarily implemented right now, and only 'de' and 'en' are supported right now. Note that Hyper 8 will try to auto-determine the language from your system locale if you don't supply this flag, so this is only necessary if you need or want to specify it manually anyway.
- `--port <PORT>` Manually sets the port used by the web editor (otherwise hyper8 chooses an available port on its own)
- `--workers <COUNT>` Set how many background workers should be run. The count can be no less than 2 (one worker for fast jobs - video analysis, thumbnails, one worker for slow jobs - video transcoding).