<!--
    SPDX-FileCopyrightText: 2024-2025 Simon Repp
    SPDX-License-Identifier: CC0-1.0
-->

# Supported Formats

## Video

In principle you can use any format you like - Hyper 8 will try to determine
whether the format you provided is actually playable in a standard browser
and report when it isn't, although this is by nature not 100% reliable.
Hyper 8 offers the possibility to request transcoding of the file into
formats that are guaranteed to be browser-compatible through the editor.

The video container formats Hyper 8 currently recognizes are:

- m4v
- mp4
- mkv
- mov

<!--
## Audio

TODO: Implement
**Heads up**: Although Hyper 8 already recognizes the listed audio formats and
registers your audio files, implementation and transcoding of audio-file only playback is
not yet implemented, i.e. this particular section still only describes the intent.

Each video can also be accompanied by audio files, so that people
can play back only the audio if they would rather do that to save
bandwidth. If the audio you provide is not playable in every
standard browser out there, Hyper 8 will let you know and offer
to transcode to ubiquitously supported formats for you.

The audio formats Hyper 8 currently recognizes are:

- aiff
- alac
- flac
- mp3
- ogg
- opus
- wav
-->

## Images

These are the formats you can use for poster images. Poster images
are always resized and transcoded for site builds.

- gif
- jpg/jpeg
- png
- webp

## Subtitles

Note that subtitle support is just an experimental prototype implementation
right now. You can put subtitle files alongside your video file(s) and they
will probably show up in the video player on your built site, but at this
point the functionality is neither guaranteed nor maintained - but it will
be in the future.

- srt
- vtt
