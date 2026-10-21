<!--
    SPDX-FileCopyrightText: 2025-2026 Simon Repp
    SPDX-License-Identifier: CC0-1.0
-->

# Architectural documentation

## System overview

The Hyper 8 codebase is structured as a cargo workspace with multiple possible
build targets (binaries), and whose members (both libraries and binaries)
depend on each other in different ways:

- `cli`

  This build target produces the command line edition of Hyper 8 and
  (directly or indirectly) depends on most of the other members in the
  workspace to provide that functionality.

- `core`

  Provides an abstraction and asynchronous "engine" for loading and
  interacting with a Hyper 8 site tree in memory. This includes watching the
  site directory for changes (and reloading changed branches on the fly),
  executing various short and long-running background jobs that transcode or
  analyze media files, performing user updates to the site tree, and also
  generating the resulting Hyper 8 static sites.

- `desktop`

  This build target produces the full desktop edition of Hyper 8. Except for
  the stand-alone `docs` generator, it depends on all other members of the
  workspace - including `cli`, because the desktop edition also ships the
  command line interface. Technically this is a [Tauri](https://tauri.app/)
  application, however, 95% of it is just the graphical launcher! The
  graphical editor itself is served through the `web` interface, and the
  desktop application merely presents this interface to the user through
  a stand-alone, native webview window.

- `docs`

  A mostly self-contained build target that generates the documentation that
  is available at https://hyper8.org/docs

- `icons`

  A small, single-file library that provides the SVG-based icons that are
  used throughout the Hyper8 editor and graphical launcher.

- `translations`

  Provides, in different languages, all the translated strings that are used
  in the editor, graphical launcher and generated Hyper 8 sites.

- `version`

  A tiny library that merely provides the version number (and git revision)
  of Hyper 8 in a normalized form, from a single source of truth.

- `web`

  This implements the visible and interactive surface of the graphical
  editor and therefore strongly depends on the `core` library of Hyper 8.
  Technically this is a web server based on [Actix Web](https://actix.rs/).

## Design notes

This documents various design decisions, thoughts, and characteristics of the
system that are not necessarily obvious or trivial to arrive at, so they
don't have to be asked or thought through over and over again. Also if
changes becomes necessary the thinking process can start from an already
documented thought process, instead of starting at zero.

### Eager and lazy image loading

#### Poster images

The poster image in the player uses an overlaid `img` instead of the `poster`
attribute on `video` because the latter only supports a single resolution/src.
This `img` currently uses `loading="lazy"` combined with `sizes="auto, ..."`
to ensure the most fitting resolution is picked. Technically the image
could be considered as [LCP](https://web.dev/articles/lcp) which speaks against
lazy loading, but at the same time the video itself (including the play button)
also does qualify for LCP, maybe even more so, and lazy loading the poster
might therefore even benefit the fast availability of the video interface itself.
Note that lazy loading in this context means that the poster is loaded as soon as
the browser can determine its exact size and location in the layout – given that
the poster is practically always above the fold, it does not mean that the poster
is loaded at a significantly later point.

### Native time notation

For linking to timecodes (e.g. `https://example.com/video#time=30s400ms`) as
well as for specifying chapter begin/end timecodes in manifests (`- 1h4m30s Credit roll`),
Hyper 8 supports a native time notation nicknamed *Hypertime*:

- `30s` (30 seconds)
- `30s400ms` (30 seconds, 400 milliseconds)
- `2m` (2 minutes)
- `2m30s` (2 minutes, 30 seconds)
- `1h` (1 hour)
- `1h30s` (1 hour, 30 seconds)
- `1h2m` (1 hour, 2 minutes)
- `1h2m30s` (1 hour, 2 minutes, 30 seconds)
- `1h2m30s400ms` (1 hour, 2 minutes, 30 seconds, 400 milliseconds)
- `377` (raw, unit-less form for 377 seconds)

Thinking and current decisions regarding edge cases:
- Consider the possible declarations `90m` (useful and completely reasonable),
  `90m30s` (useful and reasonable enough), `1h90m` (odd and probably
  accidental) or even `1h121m` (just weird). While the latter ones are not
  desired usage forms, they are nontheless permitted to keep the notation
  simple and predictable. For instance if we just were to allow `90m` but not
  `1h90m` this already implies somewhat complex rules. Likewise error
  handling would become more complex, whereas treating `1h90m` as either
  intended or erroneous still allows authors to spot and correct the error
  later, as it would be the case with any other typo.
- The mixing of unit-based and unit-less numbers – e.g. `1h30`, `2m30`,
  `30s20` is currently allowed in the implementation, resolving to the
  unit-less number being interpreted as the one following the former unit,
  i.e.: `1h30` = `1h30m`, `2m30` = `2m30s`, `30s20` = `30s20ms`, which
  roughly follows how it would be interpreted in colloquial parlance ("One
  hour thirty" for instance). `200ms40` is considered an error though.
- There is currently no implementation (nor written spec/documentation) for a
  raw, unit-less form like `377.14` or `377,14`, although this might be added
  in the future.

### Permalinks

Hyper 8 ties the naming of permalinks for collections, playlists and videos in
a site to their folder names on disk (they are the same). This greatly limits
and addresses the potential for permalink collisions already at the file
system level. For the remaining collision potential(e.g. "video" vs. "Video")
it enforces strict validation of folder names, discarding and reporting those
that do not even qualify for being used.

### Portability

Collection, playlist and video folders inside a hyper8 site can be ported
(= moved as a folder) freely within the hierarchy of the site, but also can
be moved or copied into the hierarchy of another hyper8 site, just like
that.

### Video pages vs. "Video in playlist" pages

In Hyper 8 a video is only ever featured in a single playlist, and lives
uniquely within that context. That is, a video page always refers a unique,
singular page and if the video is in a playlist it is always shown in the
context of that playlist. This is in contrast to some commercial platforms,
where a video page can be "the video page itself" or any number of pages
of "the video being in the context of a playlist". Obviously both have their
pros and cons - Hyper 8 foregoes the possibility of increased flexibility and
"remixability" (or the possibility of remixability without data duplication at
least) for the benefits of a simpler "geography of mind" where things can be found in
one place and one place only, and even more notably for the benefit of a
much simpler and robust architecture that enables (e.g.) Hyper 8's
portability guarantees for videos across sites.
