<!--
    SPDX-FileCopyrightText: 2025 Simon Repp
    SPDX-License-Identifier: CC0-1.0
-->

# The site root directory

The root directory of your site only has one notable characteristic: It is the
the only place where the [site manifest](the-site-manifest-site-eno-.html)
may be placed. The site manifest is a structured plaintext file that contains
settings which globally apply to your entire site, such as the language,
theme and deployment configuration. The contents of the site manifest
(and also all other manifests in the site directory) use a simple data
language called [eno](https://simonrepp.com/eno/) - you can read up on the
syntax at the linked page, but given its simplicity you will likely get the
hang of it just by looking at the given examples in this section.

Other than for this manifest, the root directory of your site is simultaneously
also another type of directory - which depends on the type of site you create.

- If your Hyper 8 site should present only a single video, the root directory
  of the site will be a video directory.
- If your site should present only a single playlist of videos, the root
  directory of the site will be a playlist directory.
- And lastly, if your site should present any number of videos and playlists,
  arranged into an arbitrary hierarchy of your choosing, the root directory
  of your site will be collection directory.

On the next page, we'll have a look at the possible contents of the site
manifest. As you do not strictly need a site manifest, you can also skip
right to another page in this section if you like.
