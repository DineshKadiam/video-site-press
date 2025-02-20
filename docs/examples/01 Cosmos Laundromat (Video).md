<!--
    SPDX-FileCopyrightText: 2024-2025 Simon Repp
    SPDX-License-Identifier: CC0-1.0
-->

# Video: Cosmos Laundromat

An example site that presents only a single video - the open movie
[Cosmos Laundromat](https://gooseberry.blender.org/about/) by the
Blender Institute (note that this is an entirely hypothetical example).

## File and folder layout

```
cosmos_laundromat/            <--- Site/Video directory
├─ site.eno                     <--- Site manifest
├─ video.eno                    <--- Video manifest
├─ poster.jpg                   <--- Poster image
├─ cl-720.mp4                   <--- 720 pixel wide video
└─ cl-480.mp4                   <--- 480 pixel wide video
```

## cosmos_laundromat/site.eno

```eno
base_url: https://example.com/
theme: light_neutral
```

## cosmos_laundromat/video.eno

```eno
title: Cosmos Laundromat: First Cycle

link:
label = Official Website
url = https://gooseberry.blender.org/about/

poster:
description = A cute sheep on a green island
file = poster.jpg

-- description
An open movie by the Blender Institute, licensed under CC-BY.
-- description
```