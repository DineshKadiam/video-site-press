<!--
    SPDX-FileCopyrightText: 2024-2025 Simon Repp
    SPDX-License-Identifier: CC0-1.0
-->

# Installation and usage

You can download and install either the full desktop edition
or the command line edition from the
[download page](https://hyper8.org/download.html) – **both include
the command line interface**.

All remaining sections on this page will assume that you already have a
[site directory](introduction.html) to work with - if you don't, you can
simply create an empty folder on your disk (anywhere you like), for instance
by running the command below, and that will be your site directory:

```bash
mkdir my-site-dir
```

## Running the editor

Simply pass the site directory path to `hyper8` and it will start a web server
hosting the graphical editor. You can open the displayed address in your
browser and then perform every action that the desktop application provides
as well (authoring, previewing, building and syncing your site).

```bash
hyper8 my-site-dir
```

You can press `Ctrl+C` to shut down the editor at any point.

## Building (and previewing) the site

The `--build` option (short: `-b`) will build the site:

```bash
hyper8 my-site-dir --build
```

With the added `--preview` flag (short: `-p`) the site will be opened in the browser afterwards:

```bash
hyper8 my-site-dir --build --preview
```

## Syncing the site

If you've already set up an FTP/FTPS or SSH sync configuration for your site
you can sync it straight from the command line interface by adding the
`--sync-ftp` or `--sync-ssh` option, for instance:

```bash
hyper8 my-site-dir --build --sync-ssh
```

## Manually setting the build directory

By default, Hyper 8 will generate the site inside a `.hyper8_build/` directory
that is placed at the root of your site directory (e.g. `my-site-dir/.hyper8_build/`).

If you want to build (or if you will, *locally deploy*) the site to another directory
on your disk you can pass an option to do so, but please read the warning first:

**WARNING: The build directory gets WIPED during each build. Specifying the wrong
directory can lead to severe loss of data, so triple check what you pass here.**

With that covered, here is the magic option to override the build directory:

```bash
hyper8 my-site-dir --build --build-dir [danger-this-path-will-be-wiped]
```

## Using Hyper 8 without the graphical editor

You can populate and configure your site by putting content in
your site directory, using solely your file browser, text editor, terminal,
custom scripts, or whichever workflow you aim for or prefer.

How this works is thoroughly explained in the [Site directory specification](introduction.html)
and [Site directory examples](single-video-only.html) sections. For a quick
introduction there is also a video showcasing the process at
[simonrepp.com/video/hyper8/without-editor](https://simonrepp.com/video/hyper8/without-editor).
