<!--
    SPDX-FileCopyrightText: 2024-2025 Simon Repp
    SPDX-License-Identifier: CC0-1.0
-->

# Video directories

A video in Hyper 8 is, at its simplest, represented by a directory
that contains a single video file:

```
my_video/                     <--- Video directory
└─ original.mp4                 <--- Video file
```

The name of the directory (which may only contain alphanumeric characters, `-`
and `_`) determines the permalink for the video, that is, the URL/web address
under which the video will appear publicy on the web if you publish your site
online (e.g. a local directory name `my_video` could make the video available under
the hypothetical web adress `https://example.com/my_video/`).

More widely speaking, a *video* in Hyper 8 can be video and/or audio, and can
be (re)presented through multiple video and audio files to cover a range of
qualities/formats. It can also be augmented with subtitle files and
a *poster* image that is shown in thumbnails and before the video is played
back. If there is no *poster* image, Hyper 8 will always compute one
automatically (by taking a frame from the video).

A video directory can be identified as a video directory also by just placing
a `video.eno` file inside it, regardless of whether there is even already a
video file in it.

## Video files

Currently the extensions `m4v`, `mp4`, `mkv`, `mov` and `webm` are supported.

## Subtitle files

Currently the extensions `srt` and `vtt` are supported.

By naming your subtitle files with a language code (in BCP 47 format), Hyper 8 can
automatically determine its language, for instance `en.srt` would indicate english
subtitles while `fr.vtt` would indicate french subtitles.

## Audio files

Currently the extensions `aac`, `aiff`, `alac`, `flac`, `mp3`, `ogg`, `opus`,
and `wav` are supported. Note that the streaming player will only present an
automatically chosen single "Audio" option to your viewers, but viewers can
access all audio formats through the downloads (when downloads are enabled).
