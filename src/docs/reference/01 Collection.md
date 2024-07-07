<!--
    SPDX-FileCopyrightText: 2024 Simon Repp
    SPDX-License-Identifier: CC0-1.0
-->

# Collection

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

Alternatively, a collection can also be created by putting a file named
`collection.eno` in an empty directory, although of course the usefulness of
an empty collection will be limited.

As already hinted at above, a collection generally speaking can contain
videos, playlists and subcollections (which again can contain videos,
playlists, and subcollections (which again can contain ...)). As such it is a
generic, flexible way to group and structure content hierarchically on your
Hyper 8 site.

Content in a collection currently appears in the following order on the built site:
- Videos (ordered according to the `video_order` field)
- Playlists (those with the latest video additions come first)
- Subcollections (those with the latest content addition come first)

## The collection.eno manifest

Manifest for tagging and configuring a collection.

```eno
title: Awesome clickbait videos
offline
video_order: sort_number_desc
unlisted

link:
label = My Website
url = https://example.com

link:
label = My Social Media
url = https://social.example.com
verification = rel-me

-- description
New videos are released every tuesday!

Also check us out on the [fediverse](https://social.example.com/)
-- description
```

Stand-alone URLs inside the `description` field are automatically converted to links, and you can
additionally use markdown-style inline links such as `[Example](https://example.com)` to customize
the text of a link (Note that no other markdown syntax is supported inside `description` though).
Any number of empty lines in the description field separates paragraphs in the text.

Any number of `link` fields can be given. The `label` for a link is optional.
The `verification` attribute refers to to the [rel="me"](https://microformats.org/wiki/rel-me) mechanism,
its value can be either `rel-me` to add the `rel="me"` attribute to the link or `rel-me-hidden` to add
the attribute and additionally hide the link from the site, thereby making it only visible to servers
that want to verify you are the owner of the site through this attribute.

The `offline` flag, if given, indicates that the collection should not be included in the built
site, and therefore also not deployed to the server.

The `unlisted` flag, if given, indicates that the collection should not appear or be linked to
on any of the public pages, i.e. only those that are given the link to them, will know of their
existence.

The `video_order` field supports the following values: `release_date_asc`, `release_date_desc`,
`sort_number_asc`, `sort_number_desc`, `title_asc`, `title_desc`. Note that this only affects
ordering of the videos directly inside a collection, it does not affect ordering of playlists
or subcollections.
