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

Stand-alone URLs inside the `description` field are automatically converted to links, and you can
additionally use markdown-style inline links such as `[Example](https://example.com)` to customize
the text of a link (Note that no other markdown syntax is supported inside `description` though).

Any number of `link` fields can be given. The `label` for a link is optional.
The `verification` attribute refers to to the [rel="me"](https://microformats.org/wiki/rel-me) mechanism,
its value can be either `rel-me` to add the `rel="me"` attribute to the link or `rel-me-hidden` to add
the attribute and additionally hide the link from the site, thereby making it only visible to servers
that want to verify you are the owner of the site through this attribute.
