<!--
    SPDX-FileCopyrightText: 2025 Simon Repp
    SPDX-License-Identifier: CC0-1.0
-->

# Linking to timecodes

You can annotate links so that they directly seek/jump to a specific timecode
in a video when opened. This can be used both in a link pointing to a
Hyper 8 site (e.g. linking to a video from a blog) or a link on the video
site itself (e.g. a link in the video description pointing to a specific
section of the video).

## Examples

Let's assume you just released a video on your Hyper 8 site at
`https://example.com/the-video/` and want to point your followers to the fancy
choreography happening 20 seconds into the video. The following link
will take people to the video page, automatically setting the player
to timecode 0:20:

`https://example.com/the-video/#time=20s`

Now this was about taking people to a specific timecode from an *external* page or platform, but the same also works directly
from within a video page. Assuming you are publishing a vlog, just released a new vlog entry at `https://example.com/new-vlog-entry/`
and want to link to various parts of your video, this is what you could
put into your `description` field for the episode:

```
In today's vlog entry I show off my favorite memes!

[What is a meme](#time=1m20s)
[History of memes](#time=14m)
[The Einstein-Rosen-Podolsky bridge theory meme](#time=52m12s)
[The new Bielefeld conspiracy meme](#time=1h26m3s)
[1080 Stunt Dog meme](#time=2h7m)
```

## Details on the syntax used

You can abbreviate the time parameter name (`t` as in **t**ime):

- `#t=30s` (Open the video at timecode 0:30)

Timecodes are flexible, but keep the order **h**ours -> **m**inutes -> **s**econds -> **m**illi**s**econds:

- `#time=30s` (30 seconds)
- `#time=30s400ms` (30 seconds, 400 milliseconds)
- `#time=2m` (2 minutes)
- `#time=2m30s` (2 minutes, 30 seconds)
- `#time=1h` (1 hour)
- `#time=1h30s` (1 hour, 30 seconds)
- `#time=1h2m` (1 hour, 2 minutes)
- `#time=1h2m30s` (1 hour, 2 minutes, 30 seconds)
- `#time=1h2m30s400ms` (1 hour, 2 minutes, 30 seconds, 400 milliseconds)

- **Not allowed**: `#time=30s2m1h`
