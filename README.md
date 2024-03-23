# Hyper 8 Video System

A static site generator for video content

<img src="https://simonrepp.com/hyper8/readme.png?1" alt="Screenshot showing the hyper8 browser editor and a built page (alpha version)"/>

## Get an early glimpse

Watch the ~4min [introduction video](https://simonrepp.com/hyper8/teaser) to get an idea what is being built here.

Follow this [thread on the fediverse](https://post.lurk.org/@freebliss/112043777597496559) for continuing reports on the development.

You can also already browse the [work-in-progress documentation](https://simonrepp.com/hyper8/docs)
to get a feel for what hyper8 is all about.

## Hyper 8 is in Alpha

Hyper 8 can mostly be stably built and run from the main branch,
however, many things are subject to change at any time and various
things indicated in the interface, readme and docs may not actually
be implemented yet. Most notably, Hyper 8's browser interface performs
various disk operations (including recursive directory removals) in the
background and pending further stabilization the program should thus be
handled with caution, on a well back-upped system. Fuzz testing with
malicious input and then hitting every Delete button available in the
browser interface might yield unfortunate results.

## Documentation

Uses `path/to/collection` as site directory and locally serves
a web interface (currently fixed at `http://localhost:8081`) where you can
manage your collection.

```sh
hyper8 [SITE_DIR]
```

Run it like this to perform an instant build (wihout hosting the web interface):

```sh
hyper8 --build [SITE_DIR]
```

The build directory (by default `.hyper8_build` in the collection directory) can be configured:

```sh
hyper8 --build --build-dir [BUILD_DIR] [SITE_DIR] 
```

You can use `--workers [count]` to determine how many background workers Hyper 8 should
use to encode videos and thumbnails, etc., however the count can be no less than 2 (one
worker for fast jobs - video analysis, thumbnails, one worker for slow jobs - video transcoding).

```sh
hyper8 --workers 3 [SITE_DIR]
```

Run with `FAST_ENCODING=1 hyper8 ...` to encode everything at the lowest quality and
as fast as possible (debugging aid).

## Funding

Initial development of the Hyper 8 Video System is being funded by the [Austrian Federal
Ministry for Arts, Culture, Civil Service and Sport (BMKÖS)](https://www.bmkoes.gv.at/en.html) – thank you.

![BMKÖS Logo](https://simonrepp.com/hyper8/bmkoes_logo.jpg)

## License

Hyper 8 is licensed under the [AGPL-3.0-or-later](https://spdx.org/licenses/AGPL-3.0-or-later.html).
