<!--
    SPDX-FileCopyrightText: 2024-2025 Simon Repp
    SPDX-License-Identifier: CC0-1.0
-->

# Changelog

## 0.23

Released on August 28, 2025

### Audio-only formats

You can now provide audio-only versions of your videos to your audience,
allowing them to merely listen to the audio in your videos without having to
transfer the full video stream.

- Introduce audio file abstractions, caching and processing, refactor jobs ([`bf9a702`](https://codeberg.org/simonrepp/hyper8/commit/bf9a702bee4e56aa13406f6f66689b62720e41f4))
- Extract VideoFile and VideoMeta abstractions into own modules ([`3c44d33`](https://codeberg.org/simonrepp/hyper8/commit/3c44d33fe680351e01204103112c9c625863b977))
- Implement encode audio endpoint, restructure media endpoint around added audio ([`34e8d0c`](https://codeberg.org/simonrepp/hyper8/commit/34e8d0cc9205628b679bdce3766bdf3434fbe47d))
- Switch internal terminology from "version" to "video file" everywhere ([`d3c83e3`](https://codeberg.org/simonrepp/hyper8/commit/d3c83e3f8fca2fab00607cc24bc410d89745b940))
- Integrate audio-only option in version selectors, write audio file to build ([`cc79049`](https://codeberg.org/simonrepp/hyper8/commit/cc79049787589b91b8aabf89a0c75ccc8ad98bf3))
- Implement unified audio/video file upload, gracefully handle unsupported types ([`6928a01`](https://codeberg.org/simonrepp/hyper8/commit/6928a016df957c9873ed154ecd42739f53d3e4f8))
- Render video player even when we only have an audio file available ([`43347f3`](https://codeberg.org/simonrepp/hyper8/commit/43347f3711916467fad4f46f4fd72f8c16ce48a8))

### Customizable ordering for playlists and subcollections

Customizable ordering for videos in collections and playlists is available for a long time already.
Now you can also customize the ordering of playlists and subcollections on your site!

- Introduce customizable ordering and sort numbers for collections and playlists ([`5c43bca`](https://codeberg.org/simonrepp/hyper8/commit/5c43bca3fa2f3d686f5d4ee82325047c43463a43))
- Extend new container based ordering to navigation and subcollection rendering ([`7d49e7c`](https://codeberg.org/simonrepp/hyper8/commit/7d49e7c443fe3f14a21b054fced6058e5a461f11))
- Generalize sort direction as a modifier passed to comparison functions ([`a041a88`](https://codeberg.org/simonrepp/hyper8/commit/a041a8871209c642a0ec198281e77a2fc8b68cc0))
- Fix flipped comparisons in video file fidelity comparison ([`381da90`](https://codeberg.org/simonrepp/hyper8/commit/381da900d0b1aaf82c77b610e89f4a15e66a98ca))

### Site design and layout refined

0.23 significantly improves the header section of your site, adding a new
browse icon and carefully crafted responsive behavior to retain usability
even on the narrowest of screens. Misc additions throughout the site,
such as integrating the new logo icon as a favicon, polishing responsive
behavior in the footer, adding further experimental "video bubbles" in
the layout, and so on, round off the package in this area.

- Refine header and navigation responsive behavior, add browse icon ([`a912fef`](https://codeberg.org/simonrepp/hyper8/commit/a912fef78a7af18216dc336f94dac9333f4eff2c))
- Fine-tune the search field responsive behavior ([`60800d0`](https://codeberg.org/simonrepp/hyper8/commit/60800d0d425a59a2403064346e85019db655abd0))
- Render video bubbles on collection/playlist pages, refactor markup and classes ([`e098b5d`](https://codeberg.org/simonrepp/hyper8/commit/e098b5de0e18843a1c79183ca937b19ab69e42f2))
- Tweak footer responsive behavior and markup, rewrite media queries ([`7080ea8`](https://codeberg.org/simonrepp/hyper8/commit/7080ea8e0692339f5bc091160bfeaebfe02a4c0e))
- Remove playlist video numbers on collection/playlist pages ([`37febcc`](https://codeberg.org/simonrepp/hyper8/commit/37febcc037dd76e8af58a95f80805af35f736ccc))
- Integrate new favicon in editor and built sites ([`6745ad5`](https://codeberg.org/simonrepp/hyper8/commit/6745ad559de2c0c015c3c9103a3781cddebffba6))

### Deployment from CLI

0.23 finally (re-)introduces deployment from the commandline. Running "hyper8
[path] --build --deploy" now does exactly that, also asking for
your confirmation before deployment starts.

- Implement synchronous deployment from terminal, including user confirmation ([`54075a0`](https://codeberg.org/simonrepp/hyper8/commit/54075a03cf609517b2b439c80a26f69dc99ef0da))

### Storage overview page

- Improve storage legend formatting, use LQIP colors for the diagram and legend ([`8f238aa`](https://codeberg.org/simonrepp/hyper8/commit/8f238aa7dad6b1e023eefbb5853b84356d80dbd1))

### Background jobs

Background processing has received a lot of attention at the foundational
level: Errors are better propagated to the interface, error messages are
localized throughout the worker implementation and background jobs page, the
entire job system has been refactored to be more extensible and maintainable
in the future, and lastly the background jobs page itself (one of the weaker
links in the interface) has seen some updates to its structure and layout,
improving clarity and overview (with more refinement to come in the future).

- Extend error propagation from background jobs, extend translation coverage ([`631eaf5`](https://codeberg.org/simonrepp/hyper8/commit/631eaf545b874a6053a0a1fd1a5ffba93f5688a5))
- Improve job system, incorporate failed jobs in --build and progress indicator ([`dada086`](https://codeberg.org/simonrepp/hyper8/commit/dada086bd5cc9c6b7ef15cbe8199e5a6c93a1015))
- Restructure and fully translate jobs page in the editor ([`e070ce8`](https://codeberg.org/simonrepp/hyper8/commit/e070ce880ef45d134de8c4f364c8e42fe1c157f0))
- Refactor job dispatch and move worker code into more specific submodules ([`0e18216`](https://codeberg.org/simonrepp/hyper8/commit/0e182164a6da921f2ed6c1aaf81950a069aa8b25))
- Translate all messages in background workers, revisit update abstractions ([`152ce23`](https://codeberg.org/simonrepp/hyper8/commit/152ce23baa649aa2989073d191bdf09ccc67c8d4))

### Translations

Throughout the interface pending localization tasks have been completed,
making the largest part of all messages in Hyper 8 translateable.

- Extend translation coverage
  * Extract translation strings into separate language modules ([`87fc925`](https://codeberg.org/simonrepp/hyper8/commit/87fc92572badd62dfb063d3b31370fdb3c412e7a))
  * Translate misc strings marked for translation ([`7754ce0`](https://codeberg.org/simonrepp/hyper8/commit/7754ce0d2710a8da261fda8ae158c161bce69a33))
  * Extend error propagation from background jobs, extend translation coverage ([`631eaf5`](https://codeberg.org/simonrepp/hyper8/commit/631eaf545b874a6053a0a1fd1a5ffba93f5688a5))
- Fully localize deployment page, improve clarity of dynamic progress reporting ([`f188ca4`](https://codeberg.org/simonrepp/hyper8/commit/f188ca4c605f8e3ee7b83566eeed679ae3821728))

### Misc

- Always allow re-encoding video files at their original resolution ([`64724e4`](https://codeberg.org/simonrepp/hyper8/commit/64724e4eb5d5293bd1e7d743195e069bfb302b77))
- Introduce language-aware alphabetical ordering everywhere (e.g. for letters with diacritics) (with [Sam Muirhead](https://animate-activate.studio))
  * Integrate ICU collation based alphabetical sorting ([`f8d4646`](https://codeberg.org/simonrepp/hyper8/commit/f8d464695791e12d60063225b80201e4776e4675))
- Add short -b/-p aliases for --build/--preview arguments ([`04f544c`](https://codeberg.org/simonrepp/hyper8/commit/04f544c25bb5089138cb6d4f3ef8de11eebf4eca))
- Use clearer labeling for image description input fields ([`108693c`](https://codeberg.org/simonrepp/hyper8/commit/108693c4aa230b129ceee856b97fec16116382b9)) (with [Arnauld](https://codeberg.org/arnauld))
- Improve errors for the critical first steps of verifying the deployment config ([`0f30aff`](https://codeberg.org/simonrepp/hyper8/commit/0f30affd8108d253493f95f2d4820221d39f554d)) (with [ecogex](https://ecogex.com))
- Improve clarity of dynamic progress reporting
  * Fully localize deployment page, improve clarity of dynamic progress reporting ([`f188ca4`](https://codeberg.org/simonrepp/hyper8/commit/f188ca4c605f8e3ee7b83566eeed679ae3821728))
- Move "Render format" button below advanced settings in video encoding form ([`202990d`](https://codeberg.org/simonrepp/hyper8/commit/202990d3a85af101e8d8b82fa43876fc3f3cae28)) (with [Arnauld](https://codeberg.org/arnauld))
- List videos before playlists and subcollections in the editor tree view ([`b7f4b1d`](https://codeberg.org/simonrepp/hyper8/commit/b7f4b1d52619d42422642babe3b8b470e29a996a))
- Automatically focus confirm action inside modal editor confirmation dialogs ([`bcee7c1`](https://codeberg.org/simonrepp/hyper8/commit/bcee7c1166a8e866dd54af115881587083dc6685))

### Fixes

- Fix client script exception occuring only for unlisted videos in playlists (with [Kristoffer Lislegaard](https://kristofferlislegaard.com/))
  * Conditionally bind stand-only replay button handler in video js ([`a8801db`](https://codeberg.org/simonrepp/hyper8/commit/a8801dbbf7effa0b58dca1fa762105a602ee9427))
- Discard "N/A" time values reported by ffmpeg HEVC encoder ([`6753f0a`](https://codeberg.org/simonrepp/hyper8/commit/6753f0acdd602e4029081825435de0be94187bce))
- Ensure banner preview images in the editor are constrained to 100% width ([`340c517`](https://codeberg.org/simonrepp/hyper8/commit/340c5171465ddab998be00dc64992aae63229c72))
- Pass the correct prefix when rendering bubbles for containers in a collection ([`1deb440`](https://codeberg.org/simonrepp/hyper8/commit/1deb440c2778298f74f8433f1ebc9afcd7e18893)) (with [DURAD](https://durad.xyz/))
- Reference favicon asset relatively and with asset hash ([`af31541`](https://codeberg.org/simonrepp/hyper8/commit/af315412e76b410674ff0ed6d8436368da65f044)) (with [DURAD](https://durad.xyz/))

## 0.22

Released on August 13, 2025

### Introducing Browse and Search

Visitors can now quickly browse and search all collections and playlists on a Hyper 8 site, directly from the header!

- Introduce search feature ([`06e7521`](https://codeberg.org/simonrepp/hyper8/commit/06e75214309092d8c5d3e2069213d966462d431e))
- Move search to header ([`f603714`](https://codeberg.org/simonrepp/hyper8/commit/f60371424096a991bec436e4797b8b06a871418f))
- Switch to a nested search model, exclude search feature on single video sites ([`71a649e`](https://codeberg.org/simonrepp/hyper8/commit/71a649eca182f177d41bf53398e31b5e4fb447aa))
- Drop unused walk_recursive/SiteItem abstractions ([`8ace473`](https://codeberg.org/simonrepp/hyper8/commit/8ace47391a3acb64f11e962c6f031203fed23bf2))
- Introduce browse feature, unify navigation in header ([`b307c4e`](https://codeberg.org/simonrepp/hyper8/commit/b307c4e6647c16bf6b795faf8e1e8bf35222c4a6))
- Add keyboard shortcut (b) for toggling the browser, unify site tree access ([`5361c79`](https://codeberg.org/simonrepp/hyper8/commit/5361c79ab8a3e6b7393d5075d568e6fc28b7483f))
- Sketch out mobile navigation, link title to home, write lqips to js site tree ([`4bc4dc8`](https://codeberg.org/simonrepp/hyper8/commit/4bc4dc8513e941f01c900afb8999e0e12c130fc2))
- Respect clean/explicit url mode for header home/title link ([`2b658b6`](https://codeberg.org/simonrepp/hyper8/commit/2b658b6e32647badfa8931b5f976576469a34fec))
- Respect clean/explicit url mode for header home/title link (add missing parts) ([`e2777d3`](https://codeberg.org/simonrepp/hyper8/commit/e2777d3ad808bfd438634436aa1278ecf348b2f0))
- Implement collapse/expand toggles and refine site tree navigation ([`dfeb789`](https://codeberg.org/simonrepp/hyper8/commit/dfeb789cdcb5a2211e2c2f08902cd6ef3d78ba77))
- Indicate and/or pre-expand current page in site tree navigation ([`2c4bd01`](https://codeberg.org/simonrepp/hyper8/commit/2c4bd010f1a4b043ac37a0251cc953759aa452d1))
- Iterate on design, displayed details, ordering, search and responsive behavior ([`d194865`](https://codeberg.org/simonrepp/hyper8/commit/d1948651fe2bb5e6ba24323219729abff03f646c))
- Ensure immediate transition from browse to search and back in all states ([`b77aedf`](https://codeberg.org/simonrepp/hyper8/commit/b77aedfe7e9a1c125a8929a4e349cd2db04624f9))

### New playlist sidebar and improved video page layout

Playlists are now also displayed as a contextual sidebar alongside their videos.
The playlist can be toggled away, which is then also remembered throughout the session,
so it stays out of your way if you want full attention only on the playing videos themselves.

- Revisit and activate the playlist context interface widget ([`7ae0853`](https://codeberg.org/simonrepp/hyper8/commit/7ae0853768c03b2e5519e4e5f38e394816b95bd8))
- Iterate on layout, design, outlines and contextual elements ([`8e7fc3c`](https://codeberg.org/simonrepp/hyper8/commit/8e7fc3c237f1d91803ce51ba80eb42a3ec88aa3a))
- Iterate on design, displayed details, ordering, search and responsive behavior ([`d194865`](https://codeberg.org/simonrepp/hyper8/commit/d1948651fe2bb5e6ba24323219729abff03f646c))
- Support toggling the playlist sidebar for a session, refactor script globals ([`bfe13c9`](https://codeberg.org/simonrepp/hyper8/commit/bfe13c9a28706c66f9208f3e683083bcbdfffbee))
- Tweak playlist sidebar responsive behavior ([`bf1a5ff`](https://codeberg.org/simonrepp/hyper8/commit/bf1a5ff325227fad4fee7e1dc7417cd4f08a2bf8))

### Player outro screen

When a video in a playlist finishes, your viewers can now directly jump to all
other videos of the playlist, right from the player itself. This also makes
playlist navigation in embedded videos possible. Also includes a
newly introduced "replay" icon.

- Extend player outro screen, use LQIPs in thumbnails, fix player state issues ([`c97753b`](https://codeberg.org/simonrepp/hyper8/commit/c97753b9b160bfc39d07f99e4518491483f05719))
- Add dedicated icons for "Replay" and "Next Video" links in video player ([`a7760a8`](https://codeberg.org/simonrepp/hyper8/commit/a7760a87ec91edd54dbe2a6eda6022752222a577))
- Synchronize and polish latest changes in site and embed player implementations ([`d55e262`](https://codeberg.org/simonrepp/hyper8/commit/d55e262e13b52b823fdb3a48249a285f247b3f24))

### Poster image cropping aid

In the editor you now have visual cropping aids that assist with choosing the right poster
images for videos, and correctly framing text inside them (safe margins).

- Visualize poster and thumbnail cropping against video on poster image page ([`6802d7e`](https://codeberg.org/simonrepp/hyper8/commit/6802d7e7e39a6ea6c13eedc5552a6f7da3ac0d61))
- Include textual poster format info in visual poster image cropping aid ([`53f9745`](https://codeberg.org/simonrepp/hyper8/commit/53f974579b526dbb5d407733835915b7f28ac1a9))

### Fast image previews / LQIPs

This release adds groundwork and first integrations of LQIPs (low quality image previews),
that is, computation of dominant colors from poster images to prepare extremely
fast loading preview versions of thumbnails which are shown to visitors right until the actual
thumbnails (wich themselves are also optimized for fast loading) are loaded. These LQIPs are
now also prominently featured in an experimental layout element that hints at the videos in a
parent container (collection or playlist) as small "bubbles".

- Implement asynchronous, cache-backed poster meta and LQIP computation ([`97db0c8`](https://codeberg.org/simonrepp/hyper8/commit/97db0c8ec9b3719770fedc9bba5c46904e13a701))
- Extend player outro screen, use LQIPs in thumbnails, fix player state issues ([`c97753b`](https://codeberg.org/simonrepp/hyper8/commit/c97753b9b160bfc39d07f99e4518491483f05719))

### Shortcuts

This release adds a number of new shortcuts for accessing the new navigation system,
as well as a new shortcut for muting/unmuting audio in the player. Additionally
all shortcuts are now also made discoverable to visitors through a shortcut help
panel that is accessible from the footer.

- Implement audio mute toggle, handle shortcuts case-insensitively, refine help ([`4379530`](https://codeberg.org/simonrepp/hyper8/commit/4379530deaff592fe7ad31ccdb07f4a71dfa3025))
- Introduce shortcuts link and help panel ([`904d72f`](https://codeberg.org/simonrepp/hyper8/commit/904d72f03a1ad57060c22741b3e9455ddb387daf))
- Synchronize and polish latest changes in site and embed player implementations ([`d55e262`](https://codeberg.org/simonrepp/hyper8/commit/d55e262e13b52b823fdb3a48249a285f247b3f24))

### Trial for a new logo icon

The logo icon that has been in use so far was merely a placeholder. The new
design is now a candidate (on trial) for Hyper 8's first official logo icon
design! Conceptually the logo is meant to have a fixed shape, but might adapt
to different color schemes throughout different versions and places of use
(that is, different gradients, different solid colors, etc.).

- Trial new logo design ([`a34711f`](https://codeberg.org/simonrepp/hyper8/commit/a34711f8c96dfd1c870cae5e1bad1cb829219c00))
- Refine monochrome logo icon variant to use subtle opacity gradients ([`b17c8b7`](https://codeberg.org/simonrepp/hyper8/commit/b17c8b712d02f23f0deb86d34d41d3e2e6f40f07))

### Command Line Interface

There is now a new --preview option for automatically opening the result of
a --build command in the browser right after it completes, and Hyper 8
will now return error codes to your terminal when something goes wrong,
helping you to build robust scripted workflows around Hyper 8.

- Add --preview option for directly opening the site after building through CLI
  * Add --preview option for directly opening the site after building through CLI ([`90a680f`](https://codeberg.org/simonrepp/hyper8/commit/90a680f29411cdc06b59c1307b139205cfdbecb2))
  * Prepare server-less preview implementation for upcoming ESM removal ([`9d968c4`](https://codeberg.org/simonrepp/hyper8/commit/9d968c4e43d4000d79a216574f5aeac282693f5e))
- Let the application return an error code for all obvious error branches ([`9db2797`](https://codeberg.org/simonrepp/hyper8/commit/9db27973ddc0bff40917cc5dd6d8004210b513f5))

### Cache improvements

Hyper 8 now separately tracks cache artifacts of different types
(computed metadata, resized images, etc.), greatly reducing the need for
recomputation during future updates to the architecture.

- Switch to discrete cache item types to allow partial cache invalidation ([`b8ea786`](https://codeberg.org/simonrepp/hyper8/commit/b8ea78622fbda8576be97feb1530a78af0861d9e))

### Fixes

- Force player aspect ratio in embeds as well ([`1e4d093`](https://codeberg.org/simonrepp/hyper8/commit/1e4d0933d71de69d5fb9edf3e8fa6461bcc86034)) (with [Wolfgang Dorninger](https://dorninger.servus.at))
- Fix wrapping behavior of action buttons on video page (with [Kristoffer Lislegaard](https://kristofferlislegaard.com/))
- Fix occasionally occuring partial overlay coverage in player outro screen (with [Dave Riedstra](https://daveriedstra.com/))
  * Extend player outro screen, use LQIPs in thumbnails, fix player state issues ([`c97753b`](https://codeberg.org/simonrepp/hyper8/commit/c97753b9b160bfc39d07f99e4518491483f05719))
- Fix "Next video" link text color in player outro screen ([`c23a4c9`](https://codeberg.org/simonrepp/hyper8/commit/c23a4c95a0a23d726cb33ab190c6897468179dc9))
- Disable preview button in the editor for offline content ([`d1b638e`](https://codeberg.org/simonrepp/hyper8/commit/d1b638ed237e9b273ad026c0e06460bc50bc1b88))
- Fix missing trailing slash in dynamically rendered configure deployment link ([`48ad039`](https://codeberg.org/simonrepp/hyper8/commit/48ad0391a5a9fa026082f59d862c25c702a09ed0))

## 0.21

Released on July 15, 2025

### Built-in FTP deployment

- Introduce built-in FTP deployment (4a4d62c)
- Use content-aware syncing to minimize service interruptions during deployment (a272944)

### Fixes and improvements

- Improve responsive behavior and fix glitches with portrait aspect ratio videos (b02a1e8)
- Ensure version selector in player controls is ordered descending by fidelity (28960a7)
- Url-encode arbitrary video filenames for src attribute usage (b791dbc)
- Transpose resolution when rotational side data in video stream is present (aa1888b)

## 0.20.1

Released on May 22, 2025

- Source feeds from public (instead of published) videos (9fac2ad)
- Add missing persistence of feeds option to collection/playlist manifests (ddae340)

## 0.20

Released on May 22, 2025

### Player improvements

- Allow on-the-fly resolution switching during video playback (a9a6ebe)
- Introduce pre-playback controls and video title/backlink on embedded player intro screen (7826508, aa2ff26)
- Dim inactive subtitle/volume controls, port adaptive volume controls to embeds (9a7cbe9)

### Data integrity and safeguards

Disable graphical editing/prevent graphical updates from editor while there are manifest errors (ece90f3, 1c208f8, 47f0361, 569b044)

### Editor improvements

- Move links to separate editor subpages (6187910)
- Move collection and playlist banner image editing to separate editor pages (67af786)
- Validate subtitle languages against IANA language tag registry (c8c5edc)
- Display video file count in tree view (63b263f)

### Fixes

- Fix collection feed not sourcing its recursively built list of videos (fd4f23d)
- Fix RSS feed title accidentally being used as title for items (97088a8)
- Fix pixel count initialization in highest quality video version getter (4700463)
- Fix subtitles not being written to manifest when first explicity set in editor (6142dd2)
- Fix incomplete adaptation of the endpoint code for site manifest updates (07fe771)
- Port missing subtitle control css/js adjustments to embeds (67f31ef)
- Ensure fullscreen button icon updates regardless how fullscreen state changes (2284dd9)
- Redirect to video files page after successful downsize request (2a8b59e)
- Perceptionally tweak horizontal centering of resolution selector play icon (6ce9510)

## 0.19

Released on May 06, 2025

### Feeds

- Introduce Atom and RSS feeds for the site, (sub)collections and playlists (c935aa4, df3746e)

### Subtitles

- Implement listing, uploading and deleting subtitles (aefb68d)
- Implement manifest option for explicitly setting subtitle label and language (7ad2d8e)
- Implement subtitle edit page in the editor (37f6cde)
- Improve usability and accessibility of the video player subtitle controls (7bbee82)

### Design/usability (Generated sites)

- Always use a light-on-dark player interface, remove version selector backdrop (d894034)
- Redesign and extend speed control with keyboard and mousewheel interaction (a14e75d)
- Fall back to mute/unmute functionality in browsers with read-only volume (34cd608)

### Design/usability (Editor)

- Introduce editor subpages for collections, playlists and videos (f1d3f80, b712772, 8862d8e, b7ccca8)
- Introduce video poster frame picker (8862d8e)
- Translate and improve usability for site setup and invalid site pages (1afda0f)
- Display "No title" label for tree view nodes without title (0885fda)

### Fixes

- Prevent embedded player using up additional, unneeded vertical space (b8c50d1)
- Remove accidental leading/trailing whitespace in embed copy codes (28d7b34)
- Revise markdown subset whitespace handling, fix accidental token duplication (bfad302)
- Fix plaintext descriptions being built but not used for platform integration (ebc0dee)
- Prevent portrait videos overflowing the player beyond the viewport height (664b123)
- Handle negative time values in ffmpeg encoding stats (f955cad)
- Retain site.eno when converting to collection/playlist at the site root (a339554)

### Misc

- Apply the site_dir name as initial title when converting from empty site (b6c4c01)
- Print Ctrl+C termination hint when starting server (aad087a)

## 0.18

Released on Mar 11, 2025

### Embeds

- Introduce video embedding (c344a14, 6cb87c3, 8fd51a1)

### Metadata and platform integration

- Introduce opt-in opengraph link previews (8a58980)
- Introduce opt-in opengraph and "player card" content embeds (92238b5, bf08629)
- Render noindex/nofollow robots meta tags on embed/unlisted pages (e1a0434)
- Add generator meta to pages, include git revision with detailed version output (89b57b7)

### Windows

- Introduce support for Windows (implement cross-platform aware site path abstraction) (08cf0dd, 1913b55)
- Canonicalize site directory without UNC form on windows when possible (5202482)

### New options

- Support enabling/disabling download buttons for videos (disabled by default) (c3e9c45)
- Support enabling/disabling copy link buttons for videos (enabled by default) (fc9613d)
- Indicate inherited state for options that propagate through the site tree (11cd60f)

### Generated sites

- Ensure vertically centered alignment of vertically cropped videos (b2c29cb)
- Make header/footer non-overlapping, move theme toggle to header, display footer (91a91cd)
- Hide volume controls when volume is read-only (primarily for browsers on iOS) (6129e1c)
- Provide downloads on dedicated page, optimize index load times (9edd66d)

### Misc additions

- Allow serving the editor under a custom ip address (0fb0fed)
- Introduce hash-based css/js asset invalidation (1ffdfb8)
- Await complete metadata coverage in site tree before building (6eb11d7)
- Improve manifest error coverage and messages (867549c)

### Fixes

- Fix broken realtime encoding progress reporting in editor (4ec176c)
- Omit download/embed buttons for videos without versions (4e24410)
- Implement post update propagation of options to site tree descendants (009e4a4)
- Remove and guard against invalid conversions of videos in playlists (21dc9d9)
- Fix feedback for collection to video conversion (fdb1381)
- Correct ffprobe-specific error message (dcaf5d7)
- Replace accidental eprint usage with eprintln (36ec0bf)

## 0.17.1

Released on Aug 25, 2024

- Handle ffprobe output variations caused by unrecognized data streams in video (d4afda7)
- Play videos inline on iOS (Walter Ebert, feb8c65)

## 0.17

Released on Jul 15, 2024

- Fix critical playback issue in older Safari browsers (Walter Ebert, c835f43)
- Propagate visibility upwards when moving video into new collection or playlist (a9ac6a3)
- Ensure optimal line length on description texts in build, iterate on buttons (f8ea9b0)
- Better hint at public address meaning in example given (c9b4e5d)
- Fix incompatible cache message being printed for new sites (0305139)
- Restore direct, quasi-synchronous building from command line (297d6ab)
- Remove early experimental downscale argument for poster computation (5846ff3)
- Fix computation of latest release dates in collections and playlists (ef0bd4c)
- Allow creating videos without providing video files right away (3e25c57)

## 0.16

Released on Jul 02, 2024

- Minor vertical margin tweaks in build (ff55695)
- Add dark warm, light deep and light warm theme variants (4f4aff9, 173b40a)
- Communicate inherited visibility (offline/unlisted) in build and tree view (c03ad47)
- Introduce offline flag to indicate resources that should not be deployed (44b72c7)
- Move locations section into edit page title element (21e4ac2)
- Allow customizing video order in collections (babcbc3)
- Disable autocompletion in new playlist/subcollection forms (97b7f05)
- Fix title sometimes not being assigned when creating playlist/subcollection (80fdc47)

## 0.15

Released on Jun 28, 2024

- Offer automatic and forced original frame rate option for encoding (72149a0)
- Display full public url context as prefix for all permalink input fields (6763d77)
- Improve editor navigation and orientation readability (8f58552)
- Hide advanced video encoding parameters behind a toggle (85648e6)
- Only show duration info when length between formats diverge (992e119)
- Ensure unlisted flag is transferred or inherited when converting between types (010407b)
- Introduce locations section for viewing/copying live urls and local paths (2693388)
- Make collection, playlists and videos directly previewable (9144d74)
- Fix divide by zero in storage graph computation (1690c7a)
- Add dark deep theme variant (0ade9f6)
- Fix incorrectly computed encoding progress in some edge cases (20a26da)
- Improve editor controls layouting (3ad9ead)
- Improve tree view icons and hierarchy readability (f91055e)
- Introduce dynamic, customizable editor tree view order (92d6e1f, 8a7f95c, 9734bfb)
- Fix video player interface z-index issues in chromium-based browsers (ed9634b)
- Allow creating playlists and subcollections with title and/or permalink (fb1950a)
- Render paragraphs based on empty line separation in markdown subset parser (e781d32)
- Update theme naming scheme, unify theme pickers (65bbbd4)
- Improve rendering of empty playlist and nodes without title in editor tree view (72f8fb2)
- Make use of video sort number to auto-sort collection videos as well (e010e6f)
- Introduce adaptive experimental theme (e715dc7, 86e0e19, c4f3ac2)
- Tie up some loose ends around slug generation/validation (b66412b)
- Normalize font size for textareas (3bc3dc7)
- Iterate on action/danger accents, add additional delete confirmations (476038d)
- Prompt user to confirm when they are about to discard changes made to a form (6796662)
- Fix redirect after deleting nested collection or playlist (b669811)

## 0.14

Released on Jun 17, 2024

- Improve readability of form grouping and various editor widgets (5eaf037)
- Link site url from deployment page (7fc88c8)
- Open external links in new tabs (790f033)
- Fix player timeline not picking up clicks for seeking in the center area (75dfe81)
- Fix critical issue sometimes preventing confirmation of deployment (dd8fc16)
- Begin factoring in container extension as browser support indicator (888f142)
- Improve text overflow handling and tweak hover/active styles in editor tree view (b050f58, 03625c3)
- Display playlist videos in tree view in site order (33489f1)
- Introduce explicitly configurable and implicitly smarter playlist ordering (fe5b252, b92eaa9, eb27696, 2b7764b)
- Fix missing sort of playlist videos in collection in editor tree view (81271b9)
- Fix readability of manifest error banner (94d6192)
- Fix issue preventing deletion of a collection within a collection (57eba17)
- Disable unused ffmpeg stdin/stdout background process streams (84d5445)
- Fix issues around manually typed urls of unlisted pages in preview (a539d30)
- Improve presentation of collection hierarchies in built site (e1fab97)
- Fix critical markdown subset parsing issue when encountering multiple inline links (a2e8ff6)

## 0.13

Released on May 28, 2024

First official beta release
