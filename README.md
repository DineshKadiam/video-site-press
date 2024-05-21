# Hyper 8 Video System

A static site generator for video

<img src="https://simonrepp.com/hyper8/readme.png?1" alt="Screenshot showing the Hyper 8 browser editor and a built page (alpha version)"/>

## Overview

Watch the ~4min [introduction video](https://simonrepp.com/hyper8/teaser) to get an idea what is being built here.

See the [website](https://simonrepp.com/hyper8) for some more images and information.

Follow this [thread on the fediverse](https://post.lurk.org/@freebliss/112043777597496559) for continuing reports on the development.

## Hyper 8 is in Alpha

Hyper 8 can be stably built and run from the main branch. Note that some
things are in flux, some are make-shift implementations, and some may not
work yet. Making backups of your Hyper 8 site directory is recommended while
Hyper 8 is still in its early days.

### Known issues

- The cache system (which by default puts its data in a hidden `.hyper8_cache` folder in your site directory)
  removes some, but not all, stale data it creates. Until this is fully implemented it makes sense to occasionally delete
  the cache to clear out stale data (Note that these are not large amounts of data though, just transcoded images and
  serialized video metadata).

## Documentation

Available at [simonrepp.com/hyper8/docs](https://simonrepp.com/hyper8/docs) (some areas are still in progress).

## Funding

Initial development of the Hyper 8 Video System is made possible by the [Austrian Federal
Ministry for Arts, Culture, Civil Service and Sport (BMKÖS)](https://www.bmkoes.gv.at/en.html)

![BMKÖS Logo](https://simonrepp.com/hyper8/image_funding.jpg)

## License

Hyper 8 is licensed under the [AGPL-3.0-or-later](https://spdx.org/licenses/AGPL-3.0-or-later.html).
