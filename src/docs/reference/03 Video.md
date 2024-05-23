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
the video is played back. In fact, if there is on *poster* image, Hyper 8
will compute one automatically (by taking a frame from the video), always.

## Video files

Currently the extensions `m4v`, `mp4`, `mkv`, `mov` are supported.

## Subtitle files

Currently the extensions `srt` and `vtt` are supported.

By naming your subtitle files with a language code (in BCP 47 format), Hyper 8 can
automatically determine its language, for instance `en.srt` would indicate english
subtitles while `fr.vtt` would indicate french subtitles.

## The video.eno manifest

Manifest for tagging and configuring a video, which can be placed inside the
video directory. A video directory can be identified as a video directory
also by just placing a `video.eno` file inside it, regardless of whether
there is even already a video file in it.

```eno
poster:
description = Confused looking people in an office
file = episode_1.jpg

release_date: 2024-02-12
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

Stand-alone URLs inside the `description` field are automatically converted to links, and you can
additionally use markdown-style inline links such as `[Example](https://example.com)` to customize
the text of a link (Note that no other markdown syntax is supported inside `description` though).

Any number of `link` fields can be given. The `label` for a link is optional.
The `verification` attribute refers to to the [rel="me"](https://microformats.org/wiki/rel-me) mechanism,
its value can be either `rel-me` to add the `rel="me"` attribute to the link or `rel-me-hidden` to add
the attribute and additionally hide the link from the site, thereby making it only visible to servers
that want to verify you are the owner of the site through this attribute.
