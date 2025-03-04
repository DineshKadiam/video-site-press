<!--
    SPDX-FileCopyrightText: 2024-2025 Simon Repp
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

> All options at a glance: [copy_link](#copy_link), [description](#description), [download](#download), [embedding](#embedding), [link](#link), [offline](#offline), [platform_integration](#platform_integration), [title](#title), [unlisted](#unlisted), [video_order](#video_order)

Manifest for tagging and configuring a collection.

```eno
title: Soothing clips
download: enabled
offline
platform_integration: link_previews
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

## <a name="copy_link"></a> `copy_link`

To disable the "Copy link" button for videos (by default it's enabled) you can
use the `copy_link` option, with either `enabled` or `disabled` as value.
This is inherited by playlists and videos, but can be changed on a granular
basis in their manifests.

```eno
copy_link: disabled
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

The `offline` flag, if given, indicates that the collection should not be
included in the built site, and therefore also not deployed to the server.

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

## <a name="title"></a> `title`

The title of the collection.

```eno
title: Soothing clips
```

## <a name="unlisted"></a> `unlisted`

The `unlisted` flag, if given, indicates that the collection should not appear
or be linked to on any of the public pages, i.e. only those that are given
the link to them, will know of their existence.

```eno
unlisted
```

## <a name="video_order"></a> `video_order`

The `video_order` field supports the following values: `release_date_asc`,
`release_date_desc`, `sort_number_asc`, `sort_number_desc`, `title_asc`,
`title_desc`. Note that this only affects ordering of the videos directly
inside a collection, it does not affect ordering of playlists or
subcollections.

```eno
video_order: sort_number_desc
```
