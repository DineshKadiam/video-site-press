<!--
    SPDX-FileCopyrightText: 2025 Simon Repp
    SPDX-License-Identifier: CC0-1.0
-->

# The site directory

In its source form, a Hyper 8 site is simply a set of files and folders that
is contained within a directory on your disk, which is referred to as
the *site directory*.

As an example, this could be the site directory of a simple Hyper 8 site:

```
my-hyper8-site/
├─ site.eno
├─ collection.eno
├─ my-first-video/
│  ├─ video.eno
│  ├─ poster.jpg
│  └─ first-video.mp4
└─ my-second-video/
   ├─ video.eno
   ├─ poster.jpg
   └─ second-video.mp4
```

Within a site directory, a site can be built and edited by placing files and
folders according to the conventions and specifications documented in this
section, either manually using a file explorer, semi-manually through usage
of the graphical Hyper 8 editor (which is how most people build and edit a
site), or even in a fully automated way by letting a custom script or another
program populate the directory with content.

If you are using the graphical editor and have no specific requirements
outside of this usage, reading this section is completely optional. What you
will learn in this section, is that a Hyper 8 site rests on a very simple
data model and can easily be inspected and edited by hand, if one has the
need or an interest to do so.

If you are developer, or simply favor working with non-graphical tools, your
own tools, script-based workflows, etc. this section will give you all needed
information for building Hyper 8 sites without the graphical editor.
