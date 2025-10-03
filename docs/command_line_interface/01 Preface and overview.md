<!--
    SPDX-FileCopyrightText: 2025 Simon Repp
    SPDX-License-Identifier: CC0-1.0
-->

# Preface and overview of the command line interface

> Heads up: If you're not familiar with command line interfaces, this section
  is likely not relevant for you, as it's primarily intended for people who
  are already
> familiar with them. However, if you're curious to explore new technical
> skills and possibilities, you are still warmly invited to read it! In the
  worst
> case it will be confusing and/or boring, but in the best case it might
> introduce you to a new, powerful way of working with computers.

At its core, the command line interface provides exactly the same
functionality as the desktop application - it is neither less powerful nor
less rich in features, instead it just provides different workflows that the
graphical interface can not offer (such as automation and scripting).

The command line interface provides the following functionality:

- Without any specific options passed, the CLI will start a **web server**
  that hosts the Hyper 8 editor, which you (and, if you want, others)
  can use through a browser. The options `--ip`, `--port` and
  `--workers` can be used to set the server configuration.

- The options `--build`/`-b` (optionally paired with `--preview`/`-p`) and
  `--deploy`/`-d` can be used to perform the respective actions directly
  from the CLI (without the graphical editor). The optional `--preview`
  flag interacts with the `--ip` and `--port` options to determine the
  preview server configuration.

- The options `--build-dir` and `--cache-dir` let you manually override
  to which directory Hyper 8 writes the build (or respectively) cache
  files.

  **WARNING: Both directories are WIPED during the build process.**<br>
  Please pay very close attention where you point them to.

  (By default, build and cache files are written to `.hyper8_build/`
  and `.hyper8_cache/` inside the root of your site directory)
