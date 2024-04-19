# Collection

A collection is, at its simplest, a directory containing at least one
non-empty directory that is either a video, a playlists, or a collection
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

## The collection.eno manifest

Manifest for tagging and configuring a collection.

```eno
title: Awesome clickbaitz videos
unlisted

-- description
New videos are released every tuesday!

Also check us out on the [fediverse](https://social.example.com/)
-- description
```
