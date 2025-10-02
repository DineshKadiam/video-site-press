<!--
    SPDX-FileCopyrightText: 2024-2025 Simon Repp
    SPDX-License-Identifier: CC0-1.0
-->

# Playlist directories

A playlist is, at its simplest, a directory that contains a file called
`playlist.eno` that identifies it as a playlist:

```
my_playlist/                  <--- Playlist directory
└─ playlist.eno                 <--- Playlist manifest
```

The name of the directory (which may only contain alphanumeric characters, `-`
and `_`) determines the permalink for the playlist, that is, the URL/web address
under which the playlist will appear publicy on the web if you publish your site
online.

Naturally, an empty playlist is not much of a playlist, so in reality you will
have video directories inside the playlist directory as well, but note that
the presence of video directories itself does not make the directory a
playlist by itself, because by default a directory containing video
directories will be interpreted as a [Collection](collection-directories.html). The `playlist.eno`
manifest is thus mandatory for every playlist.

Generally speaking, a playlist is a list of videos primarily intended for
sequential playback.
