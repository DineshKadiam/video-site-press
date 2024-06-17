<!--
    SPDX-FileCopyrightText: 2024 Simon Repp
    SPDX-License-Identifier: CC0-1.0
-->

# Hyper 8 Video System

A static site generator for video

<img src="https://simonrepp.com/hyper8/readme.png?2" alt="Screenshot showing the Hyper 8 browser editor and a built page"/>

## Overview

See the [website](https://simonrepp.com/hyper8) for more screenshots and information.

Watch an old ~4min [teaser/introduction](https://simonrepp.com/hyper8/teaser) to get a glimpse of Hyper 8's underlying concepts.

Follow this [thread on the fediverse](https://post.lurk.org/@freebliss/112043777597496559) for continuing reports on the development.

## Hyper 8 is in Beta

Hyper 8 can be stably built and run from the main branch, just keep in mind
that it's not production grade software (yet) and things are still moving.
Making backups of your Hyper 8 site directory is recommended while Hyper 8 is
still in its early days.

### Known issues

Hyper 8 is a young project and a few things are knowingly incomplete - although for the most part harmlessly so:

- Banner images are well implemented on the backend, but not included in the layout of built sites yet
- Unfinished background encoding jobs are not persisted when you shut down the Hyper 8 server
- Some background jobs do not necessarily give immediate feedback of them happening, these include:
  - `Random Poster` button: After clicking wait a few seconds, then refresh - a new poster should be there
  - `Render Format` button: After clicking you see the background job on the jobs page, but not on the video page itself (yet)
  - Some other smaller tasks like metadata computation or image resizing might not communicate themselves well,
    in general if anything is off you can always wait a few seconds and refresh the page to work around such glitches.
- The cache system (which by default puts its data in a hidden `.hyper8_cache` folder in your site directory)
  removes some, but not all, stale data it creates. Until this is fully implemented you can choose to occasionally delete
  the cache to clear out stale data (Note that these are not large amounts of data at all though, just transcoded images and
  serialized *metadata* for video).
- The `--build` and `--deploy` options in the CLI are not correctly implemented yet, for now perform these things from the
  interactive browser interface only
- If you manually edit `collection.eno`, `playlist.eno`, `site.eno` or `video.eno` *and* there are errors displayed
  in the browser interface (= yellow dot(s) in the left sidebar), make sure to resolve these errors manually before performing any
  consecutive updates from the browser interface, as Hyper 8 can not yet keep/persist malformed manifest data during updates.
- In certain circumstances various foreground or background processes
  (building, deploying, encoding, etc.) might still crash - you will usually
  find some feedback on this in the terminal window which you can include
  with bug reports

## Documentation

Available at [simonrepp.com/hyper8/docs](https://simonrepp.com/hyper8/docs) (some areas are still in progress).

## Funding

Initial development (up to the first beta) was funded by the [Austrian Federal Ministry for Arts, Culture, Civil Service and Sport (BMKÖS)](https://www.bmkoes.gv.at/en.html)
in cooperation with [dérive](https://derive.at) - thank you so much!

![BMKÖS Logo](https://simonrepp.com/hyper8/thumb_funding.jpg)

If you'd like to support Hyper 8's ongoing development you can donate to the project at [Liberapay](https://liberapay.com/simonrepp) and [Ko-fi](https://ko-fi.com/simonrepp).

## License

Hyper 8 is licensed under the [AGPL-3.0-or-later](https://spdx.org/licenses/AGPL-3.0-or-later.html).
