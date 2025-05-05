<!--
    SPDX-FileCopyrightText: 2024-2025 Simon Repp
    SPDX-License-Identifier: CC0-1.0
-->

# Video

A video in Hyper 8 is, at its simplest, represented by a directory
that contains a single video file:

```
my_video/                     <--- Video directory
└─ original.mp4                 <--- Video file
```

The name of the directory (which may only contain alphanumeric characters, `-`
and `_`) determines the permalink for the video, that is, the URL/web address
under which the video will appear publicy on the web if you publish your site
online.

More widely speaking, a *video* in Hyper 8 is a single video-based media
artifact that can be (re)presented through different video (but also audio)
files to cover a range of qualities/formats. It can also be augmented with
subtitle files and a *poster* image that is shown in thumbnails and before
the video is played back. In fact, if there is no *poster* image, Hyper 8
will compute one automatically (by taking a frame from the video), always.

## Video files

Currently the extensions `m4v`, `mp4`, `mkv`, `mov`, `webm` are supported.

## Subtitle files

Currently the extensions `srt` and `vtt` are supported.

By naming your subtitle files with a language code (in BCP 47 format), Hyper 8 can
automatically determine its language, for instance `en.srt` would indicate english
subtitles while `fr.vtt` would indicate french subtitles.

## The video.eno manifest

> All options at a glance: [copy_link](#copy_link), [description](#description), [download](#download), [embedding](#embedding), [link](#link), [offline](#offline), [platform_integration](#platform_integration), [poster](#poster), [release_date](#release_date), [sort_number](#sort_number), [subtitles](#subtitles), [title](#title), [unlisted](#unlisted)

Manifest for tagging and configuring a video, which can be placed inside the
video directory. A video directory can be identified as a video directory
also by just placing a `video.eno` file inside it, regardless of whether
there is even already a video file in it.

```eno
poster:
description = Confused looking people in an office
file = episode_1.jpg

download: enabled
offline
platform_integration: link_previews
release_date: 2024-02-12
sort_number: 1
title: The pilot episode of the show
unlisted

link:
label = My Website
url = https://example.com

link:
label = My Social Media
url = https://social.example.com
verification = rel-me

-- description
An unsettling truth comes to light as three old friends
reunite at an unexpected gathering.
-- description
```

## <a name="copy_link"></a> `copy_link`

To disable the "Copy link" button (by default it's enabled) you can use the
`copy_link` option, with either `enabled` or `disabled` as value.

```eno
copy_link: disabled
```

## <a name="description"></a> `description`

Stand-alone URLs inside the `description` field are automatically converted to links, and you can
additionally use markdown-style inline links such as `[Example](https://example.com)` to customize
the text of a link (Note that no other markdown syntax is supported inside `description` though).
Any number of empty lines in the description field separates paragraphs in the text.

## <a name="download"></a> `download`

This opt-in setting enables a download button on the video page, openly inviting
and allowing visitors to conveniently choose and download a specific version
of the video to their computer.

```eno
download: enabled
```

Note that the absence of a download button (i.e. the default setting of this
option) does not mean that it is not possible to download your video - this
is always easily possible for anyone with some technical knowledge. Enabling
the download button however is an invitation to your visitors that you want them
to be easily able to do so, and inversely, not enabling it - as on every
other website - simply implies that you do not openly invite downloading, and
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

The `offline` flag, if given, indicates that the video should not be included
in the built site, and therefore also not deployed to the server.

```eno
offline
```

## <a name="platform_integration"></a> `platform_integration`

This opt-in feature adds metadata on your pages that is crawled and harvested
by many platforms in order to present your page (when it is being linked to)
using a uniform "card" design pattern inside social feeds and timelines.

To enable metadata that platforms use to generate link previews (this is based
on Facebook's [Open Graph](https://ogp.me) protocol), specify the
`link_previews` setting like this:

```eno
platform_integration: link_previews
```

To enable link previews *plus* additional metadata that lets platforms
directly embed your audiovisual content into their timelines through embedded
players (this utilizes Facebook's [Open Graph](https://ogp.me) protocol,
"player card" metadata declarations that were originally introduced by twitter,
as well as Hyper 8's native embed widgets) specify the `content_embeds`
setting like this:

```eno
platform_integration: content_embeds
```

To disable this feature, either remove the option (it is disabled by default)
or explicitly specify `disabled` if you want to override it somewhere:

```eno
platform_integration: disabled
```

## <a name="poster"></a> `poster`

The image that is displayed in thumbnails and before you press play on the
video. Hyper 8 will auto-generate one from the video initially, but in any
case you should add a textutal description later for screenreader users.

```eno
poster:
description = Confused looking people in an office
file = episode_1.jpg
```

## <a name="release_date"></a> `release_date`

When this video was released, in `YYYY-MM-DD` format. This is both displayed
and potentially used for sorting, unless you manually configure a different
sort order for the parent collection or playlist.

```eno
release_date: 2025-02-17
```

## <a name="sort_number"></a> `sort_number`

The `sort_number` field can optionally be used to enforce a custom order of
appearance for videos that appear in a collection or playlist. It accepts
whole numerical values (1, 2, 3) - including zero and negative ones
(0, -1, -2). In playlists this custom ordering will automatically be picked
up by the default ordering mechanism (which sorts first by `sort_number`
ascending, then by `release_date` ascending), but the `order` field in a
playlist manifest can be used to modify this behavior - e.g. to use
descending order or to sort by release date even if `sort_number` is present
on videos, to sort alphabetically by title, etc.

## <a name="subtitles"></a> `subtitles`

Subtitles can currently be supplied as `.srt` or `.vtt` files - simply putting
such files into a video directory already includes them in the video player
on your site. If the filename is a valid language code¹ (e.g. `en.srt` or
`de-at.srt`), its language info and label will be auto-inferred.

To manually annotate the display label and language info for a subtitle file,
use the `subtitles` option like this:

```eno
subtitles:
file = en_0.srt
label = English (closed captions)
language = en
```

¹ Following the BCP 47 standard (https://www.rfc-editor.org/info/bcp47)

## <a name="title"></a> `title`

The title of the video.

```eno
title: The pilot episode of the show
```

## <a name="unlisted"></a> `unlisted`

The `unlisted` flag, if given, indicates that the video should not appear or
be linked to on any of the public pages, i.e. only those that are given the
link to them, will know of their existence.

```eno
unlisted
```
