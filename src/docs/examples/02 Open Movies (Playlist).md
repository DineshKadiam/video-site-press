# Playlist: Open Movies

An example site that features a single playlist presenting the
[Blender Open Movies](https://studio.blender.org/films/).
(Note that this is an entirely hypothetical example)

This is the file and folder layout:

```
Open Movies/                  <--- Site/Playlist directory
├─ site.eno                     <--- Site manifest
├─ playlist.eno                 <--- Playlist manifest
├─ Open Movies.png              <--- Poster image
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

In the file `Open Movies/site.eno` we find the following content:

```eno
base_url: https://example.com/
```

... TODO