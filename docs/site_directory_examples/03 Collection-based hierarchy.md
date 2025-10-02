<!--
    SPDX-FileCopyrightText: 2024-2025 Simon Repp
    SPDX-License-Identifier: CC0-1.0
-->

# Presenting videos using collection-based hierarchies

An example site directory for a site that uses collections to build a custom
site structure - where videos are both in the root collection of the site, in
a nested playlist, and in a nested subcollection. This example again builds
on an entirely hypothetical scenario of how video content from and around the
world of [Blender](https://blender.org) could be presented using Hyper 8's
collections.

> Tip: If you compare this example with the previous example (which presents a
> single playlist only), you will notice that the playlist from the previous example
> (Open Movies) has simply - without any modification - been moved into a
> subdirectory here to form part of a collection. This demonstrates that Hyper
> 8's fundamental building blocks (videos, playlists, collections) can be
> easily moved within and across Hyper 8 sites.

## File and folder layout

```
Blender Video Archive/        <--- Site/Collection directory
├─ site.eno                     <--- Site manifest
├─ collection.eno               <--- Collection manifest
├─ interview-with-anja/         <--- Video directory
│  ├─ video.eno                   <--- Video manifest
│  ├─ poster.jpg                  <--- Poster image
│  └─ interview-anja.webm         <--- Video file
├─ interview-with-ton/          <--- Video directory
│  ├─ video.eno                   <--- Video manifest
│  ├─ poster.jpg                  <--- Poster image
│  └─ interview-ton.mp4           <--- Video file
├─ ...
├─ open-movies/                 <--- Playlist directory
│  ├─ playlist.eno                 <--- Playlist manifest
│  ├─ elephants-dream/             <--- Video directory
│  │  ├─ video.eno                   <--- Video manifest
│  │  ├─ poster.jpg                  <--- Poster image
│  │  ├─ ElephantsDream_HD.mp4       <--- 720 pixel wide video
│  │  └─ ElephantsDream_SD.mp4       <--- 480 pixel wide video
│  ├─ cosmos-laundromat/           <--- Video directory
│  │  ├─ video.eno                   <--- Video manifest
│  │  ├─ poster.jpg                  <--- Poster image
│  │  ├─ cl-720.mp4                  <--- 720 pixel wide video
│  │  └─ cl-480.mp4                  <--- 480 pixel wide video
│  ├─ hero/
│  │  └─ ...
│  └─ ...
├─ conference-recordings/        <--- (Sub-)Collection directory
│  ├─ collection.eno               <--- Subcollection manifest
│  ├─ bcon19/                      <--- Video directory
│  │  ├─ video.eno                   <--- Video manifest
│  │  ├─ poster.jpg                  <--- Poster image
│  │  └─ 19_summary.mp4              <--- Video file
│  ├─ bcon23/                      <--- Video directory
│  │  ├─ video.eno                   <--- Video manifest
│  │  ├─ poster.jpg                  <--- Poster image
│  │  └─ 23-outtakes.mp4             <--- Video file
│  └─ ...
└─ ...
```

## Blender Video Archive/site.eno

```eno
base_url: https://example.com/
theme: adaptive_cool
```

## Blender Video Archive/collection.eno

```eno
title: Blender Video Archive

link:
label = Official Website
url = https://blender.org/

-- description
A fictional video archive that collects and presents videos from the world
of Blender - interviews, open movies, and conference recordings
-- description
```

## Blender Video Archive/interview-with-anja/video.eno

```eno
title: Interview with Anja

poster:
description = Anja in front of the Blender HQ in Amsterdam
file = poster.jpg

-- description
An interview with Anja, financial manager at Blender
-- description
```

## Blender Video Archive/interview-with-ton/video.eno

```eno
title: Interview with Ton

poster:
description = Ton in front of the Blender HQ in Amsterdam
file = poster.jpg

-- description
An interview with Ton, CEO of Blender
-- description
```

## Blender Video Archive/open-movies/playlist.eno

```eno
title: Blender Open Movies

link:
label = Official Website
url = https://studio.blender.org/films/

-- description
Open movies by the Blender Institute and Studio, licensed under CC-BY.
-- description
```

## Blender Video Archive/open-movies/elephants-dream/video.eno

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

## Blender Video Archive/open-movies/cosmos-laundromat/video.eno

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

## Blender Video Archive/conference-recordings/collection.eno

```eno
title: Blender conference recordings

link:
label = Blender conference website
url = https://conference.blender.org

-- description
Recordings and outtakes from blender conference, taken over many years.
-- description
```

## Blender Video Archive/conference-recordings/bcon19/video.eno

```eno
title: BCON19 Reel

poster:
description = Scattered, happy people in front of De Balie
file = poster.jpg
```

## Blender Video Archive/conference-recordings/bcon23/video.eno

```eno
title: BCON23 Reel

poster:
description = The golden sun setting over a canal and bridge in Amsterdam
file = poster.jpg
```
