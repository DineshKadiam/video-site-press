<!--
    SPDX-FileCopyrightText: 2024-2025 Simon Repp
    SPDX-License-Identifier: CC0-1.0
-->

# Presenting only a single playlist

An example site directory for a site that features a single playlist
presenting the [Blender Open Movies](https://studio.blender.org/films/)
(note that this is an entirely hypothetical example).

> Tip: If you compare this example with the previous example (which presents a
> single video only), you will notice that the video from the previous example
> (Cosmos Laundromat) has simply - without any modification - been moved into a
> subdirectory here to form part of a playlist. This demonstrates that Hyper
> 8's fundamental building blocks (videos, playlists, collections) can be
> easily moved within and across Hyper 8 sites.

## File and folder layout

```
Open Movies/                  <--- Site/Playlist directory
├─ site.eno                     <--- Site manifest
├─ playlist.eno                 <--- Playlist manifest
├─ elephants-dream/             <--- Video directory
│  ├─ video.eno                   <--- Video manifest
│  ├─ poster.jpg                  <--- Poster image
│  ├─ ElephantsDream_HD.mp4       <--- 720 pixel wide video
│  └─ ElephantsDream_SD.mp4       <--- 480 pixel wide video
├─ cosmos-laundromat/           <--- Video directory
│  ├─ video.eno                   <--- Video manifest
│  ├─ poster.jpg                  <--- Poster image
│  ├─ cl-720.mp4                  <--- 720 pixel wide video
│  └─ cl-480.mp4                  <--- 480 pixel wide video
├─ hero/
│  └─ ...
└─ ...
```

## Open Movies/site.eno

```eno
base_url: https://example.com/
theme: adaptive_cool
```

## Open Movies/playlist.eno

```eno
title: Blender Open Movies

link:
label = Official Website
url = https://studio.blender.org/films/

-- description
Open movies by the Blender Institute and Studio, licensed under CC-BY.
-- description
```

## Open Movies/elephants-dream/video.eno

```eno
title: Elephant's Dream

link:
label = Official Website
url = https://studio.blender.org/films/elephants-dream/

poster:
description = An intriguging figure in a suite, he has sizable sideburns
file = poster.jpg

-- description
An open movie by the Blender Institute, licensed under CC-BY.
-- description
```

## Open Movies/cosmos-laundromat/video.eno

```eno
title: Cosmos Laundromat: First Cycle

link:
label = Official Website
url = https://studio.blender.org/films/cosmos-laundromat/

poster:
description = A cute sheep on a green island
file = poster.jpg

-- description
An open movie by the Blender Institute, licensed under CC-BY.
-- description
```
