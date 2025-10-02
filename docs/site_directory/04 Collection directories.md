<!--
    SPDX-FileCopyrightText: 2024-2025 Simon Repp
    SPDX-License-Identifier: CC0-1.0
-->

# Collection directories

A collection is, at its simplest, a directory containing at least one
non-empty directory that is either a video, a playlist, or a collection
itself.

```
my_collection/                <--- Collection directory
└─ my_video/                    <--- Video directory
   └─ original.mp4                <--- Video file
```

The name of the directory (which may only contain alphanumeric characters, `-`
and `_`) determines the permalink for the collection, that is, the URL/web address
under which the collection will appear publicy on the web if you publish your site
online.

A collection can also be created by putting a file named `collection.eno` in
an empty directory, although of course the usefulness of such an empty
collection will be limited.

As already hinted at above, a collection generally speaking can contain
videos, playlists and subcollections (which again can contain videos,
playlists, and subcollections (which again can contain ...)). As such it is a
generic, flexible way to group and structure content hierarchically on your
Hyper 8 site.

Inside each collection the ordering (as it appears on the built site)
is always such that ...

- all videos directly contained in the collection appear at the top,
  themselves ordered according to the collection's  `video_order` field
- playlists and subcollections appear below, in the order specified
  by the collection's `container_order` field
