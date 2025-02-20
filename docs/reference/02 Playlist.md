<!--
    SPDX-FileCopyrightText: 2024-2025 Simon Repp
    SPDX-License-Identifier: CC0-1.0
-->

# Playlist

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
directories will be interpreted as a `Collection`. The `playlist.eno`
manifest is thus mandatory for every playlist.

Generally speaking, a playlist is a list of videos primarily intended for
sequential playback.

## The playlist.eno manifest

> All options at a glance: [banner](#banner), [description](#description), [download](#download), [embedding](#embedding), [link](#link), [offline](#offline), [order](#order), [title](#title), [unlisted](#unlisted)

Manifest for tagging and configuring a playlist.

```eno
banner:
description = Colorful flags in the wind
file = my_poster.jpg

download: enabled
embedding: enabled
offline
order: release_date_asc
title: All episodes from the show
unlisted

link:
label = My Website
url = https://example.com

link:
label = My Social Media
url = https://social.example.com
verification = rel-me

-- description
This playlist includes all episodes from the show,
including the long-lost original three pilots.
-- description
```

## <a name="banner"></a> `banner`

Heads up: This is implemented in the backend but not yet displayed in the
interface.

```eno
banner:
description = Colorful flags in the wind
file = my_poster.jpg
```

## <a name="description"></a> `description`

Stand-alone URLs inside the `description` field are automatically converted to links, and you can
additionally use markdown-style inline links such as `[Example](https://example.com)` to customize
the text of a link (Note that no other markdown syntax is supported inside `description` though).
Any number of empty lines in the description field separates paragraphs in the text.

## <a name="download"></a> `download`

This opt-in setting enables download buttons on video pages, openly inviting
and allowing visitors to conveniently choose and download a specific version
of a video to their computer.

```eno
download: enabled
```

Note that the absence of a download button (i.e. the default setting of this
option) does not mean that it is not possible to download your videos - this
is always easily possible for anyone with some technical knowledge. Enabling
download buttons however is an invitation to your visitors that you want them
to be easily able to do so, and inversely, not enabling it - as on every
other website - simply implies that you do not openly invite downloads, and
visitors will have to research or inquire about your video's license before
making any use of it beyond watching it on your site.

## <a name="embedding"></a> `embedding`

This allows visitors/you to copy embed codes (html) that they/you can use to
embed your videos on their/your own sites.

Embedding is disabled by default. If you want to enable it you also need to
set the site's base_url (embeds work by displaying something from your site
on another site, for this the other site needs to point to your site's
address), and then set `embedding: enabled`, either at the collection,
playlist or video level. If you set it `enabled` at the catalog or playlist
level, you can also use `disabled` at a lower level to disable it again for
specific playlists or videos.

```eno
embedding: enabled
```

## <a name="link"></a> `link`

Any number of `link` fields can be given. The `label` for a link is optional.
The `verification` attribute refers to to the [rel="me"](https://microformats.org/wiki/rel-me) mechanism,
its value can be either `rel-me` to add the `rel="me"` attribute to the link or `rel-me-hidden` to add
the attribute and additionally hide the link from the site, thereby making it only visible to servers
that want to verify you are the owner of the site through this attribute.

```eno
link: https://example.com

link:
label = My Website
url = https://example.com

link:
label = My Social Media
url = https://social.example.com
verification = rel-me
```

## <a name="offline"></a> `offline`

The `offline` flag, if given, indicates that the playlist should not be included in the built
site, and therefore also not deployed to the server.

```eno
offline
```

## <a name="order"></a> `order`

The `order` field controls in which order the videos appear in the playlist on
the site. By default, videos are first ordered by their `sort_number`
ascending, falling back to `release_date` ascending if there are no sort
numbers, and given neither of those the order will be random. This field can
be set to one of the following:

- `default` (redundant as it's the default anyway)
- `release_date_asc`
- `release_date_desc`
- `sort_number_asc`
- `sort_number_desc`
- `title_asc`
- `title_desc`

```eno
order: release_date_asc
```

## <a name="title"></a> `title`

The title of the playlist.

```eno
title: All episodes from the show
```

## <a name="unlisted"></a> `unlisted`

The `unlisted` flag, if given, indicates that the playlist should not appear
or be linked to on any of the public pages, i.e. only those that are given
the link to them, will know of their existence.

```eno
unlisted
```
