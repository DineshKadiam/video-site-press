<!--
    SPDX-FileCopyrightText: 2024-2025 Simon Repp
    SPDX-License-Identifier: CC0-1.0
-->

# Changelog

## 0.20

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

- Handle ffprobe output variations caused by unrecognized data streams in video (d4afda7)
- Play videos inline on iOS (Walter Ebert, feb8c65)

## 0.17

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

- Minor vertical margin tweaks in build (ff55695)
- Add dark warm, light deep and light warm theme variants (4f4aff9, 173b40a)
- Communicate inherited visibility (offline/unlisted) in build and tree view (c03ad47)
- Introduce offline flag to indicate resources that should not be deployed (44b72c7)
- Move locations section into edit page title element (21e4ac2)
- Allow customizing video order in collections (babcbc3)
- Disable autocompletion in new playlist/subcollection forms (97b7f05)
- Fix title sometimes not being assigned when creating playlist/subcollection (80fdc47)

## 0.15

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

First official beta release
