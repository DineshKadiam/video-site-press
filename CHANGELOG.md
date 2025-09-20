<!--
    SPDX-FileCopyrightText: 2024-2025 Simon Repp
    SPDX-License-Identifier: CC0-1.0
-->

# Changelog

Rich release notes including screenshots, videos and detailed commit information are also available at:
https://simonrepp.com/hyper8/changes/

## 0.24

Released on September 20, 2025

### Transcripts

The new transcript sidebar panel allows your visitors to browse and read
through a transcript of the video (based on your provided subtitles).
Visitors can interactively filter the transcript and click the timecode
next to each line to jump to that specific part of the video.

- Sketch out a basic transcript sidebar panel implementation ([`8510358`](https://codeberg.org/simonrepp/hyper8/commit/85103580297d9fdc8cb050bd69c0a1d0b35e04af))
- Link to timecodes in the transcript sidebar, use millisecond precision ([`cea3fee`](https://codeberg.org/simonrepp/hyper8/commit/cea3fee37201d5bfc498c4ebefcab3fdad61c88e))
- Implement transcript filter feature, layout and style transcript sidebar ([`75daca3`](https://codeberg.org/simonrepp/hyper8/commit/75daca390b9fbeffad1f650d7219fc0d5ed888f1))
- Conditionally attach transcript filter handler, refactor volume initialization ([`db73457`](https://codeberg.org/simonrepp/hyper8/commit/db734577a5168a25d4a9288c233adbad356ec2db))

### Desktop Integration

Hyper 8 is now a native graphical application, fully integrated into the
desktop! You can still, and will always be able to, use Hyper 8 from the
command line (even as a separate CLI-only edition!), but you don't have to
anymore. After installation you can start Hyper 8 right from your desktop,
which opens the graphical launcher where you can open an existing (or a start
a new) Hyper 8 site in the editor.

The editor can now be used in three modes: Standalone -
where the editor runs in its own window, Browser-based - where the editor opens in your
system browser (as previously), and Collaborative - where the editor is also accessible
to everyone on the same Wi-Fi or LAN as you.

As all of this is a huge technical change and published for the first time,
please help out by reporting any issues you encounter. With your help, in the
coming weeks, 1.0 will officially introduce Hyper 8 as a desktop application,
ready for the world.

- Extract EditorConfig into own module ([`2ee0f0c`](https://codeberg.org/simonrepp/hyper8/commit/2ee0f0cac60b7891af44216b76d31c6e694cb1fa))
- Split main crate into workspace subcrates (cli/core/icons/translations/web) ([`20a1486`](https://codeberg.org/simonrepp/hyper8/commit/20a1486885b4a2a87cc8ddb918a72f520a2a748e))
- Switch from sidecar license files to adjacent REUSE.toml files ([`838643a`](https://codeberg.org/simonrepp/hyper8/commit/838643a8e4ba3ba71b82731f258de74913978b48))
- Establish the desktop application prototype ([`feba75b`](https://codeberg.org/simonrepp/hyper8/commit/feba75ba8aca764ad4feeba5ef996c6c4ac5d9e7))
- Extend and tweak Tauri bundle configuration ([`7700fe1`](https://codeberg.org/simonrepp/hyper8/commit/7700fe179fa5b692c49e71644d98ab58a35aba23))
- Restructure launcher, merge language/translation abstractions ([`0896233`](https://codeberg.org/simonrepp/hyper8/commit/089623368f35fe41ef3cb4eebdc22cd473c5ebe9))
- Exit the entire application when editor window is destroyed ([`8777e80`](https://codeberg.org/simonrepp/hyper8/commit/8777e800c9672dae35f96507a86fe5cf56ad84d8))
- Iterate on launcher base layout and usability ([`b5d04c8`](https://codeberg.org/simonrepp/hyper8/commit/b5d04c86d446aecf3778428f2c302aab022e35d4))
- Clean up launcher config and state separation, scaffold bookmark abstraction ([`12dd7c6`](https://codeberg.org/simonrepp/hyper8/commit/12dd7c6cf30d35d13619cf4b6cf906f2f3a0be73))
- Probe and display title and live url for site bookmarks ([`b987f9d`](https://codeberg.org/simonrepp/hyper8/commit/b987f9d9bb649b744d8a181a99d4829eb7f1bce3))
- Switch to programmatic launcher window creation ([`a7237c6`](https://codeberg.org/simonrepp/hyper8/commit/a7237c682a994b676052b142647cebc17c0f6c14))
- Inject core theme vars in launcher based on prefers-color-scheme evaluation ([`904e970`](https://codeberg.org/simonrepp/hyper8/commit/904e970b0f7a94b623b40db313f869d14097d282))
- Allow opening bookmarked sites in the file explorer ([`e4e9f0e`](https://codeberg.org/simonrepp/hyper8/commit/e4e9f0eaa17460df135b52a62d19b886e99676cf))
- Implement quick start feature in launcher ([`8158559`](https://codeberg.org/simonrepp/hyper8/commit/81585592b06eeadae9ad272b3fb1867c760f9727))
- Indicate and disable bookmarked but inaccessible sites in the launcher ([`7a66994`](https://codeberg.org/simonrepp/hyper8/commit/7a66994f0842e26e3bd55026e9656bec21a55c29))
- Iterate on launcher design, layout and usability ([`a5127f5`](https://codeberg.org/simonrepp/hyper8/commit/a5127f59d00521439f5f11e00bea21af5b918671))
- Only open the browser on editor server startup when requested ([`720f9f5`](https://codeberg.org/simonrepp/hyper8/commit/720f9f58e5a22712122b55220f0e8b1d90365ef5))
- Implement dynamically passing back the url after editor server initialization ([`9a222b7`](https://codeberg.org/simonrepp/hyper8/commit/9a222b70fbb1e2c66645726999eb38e9d93d4e03))
- Refine behavior, labeling and design for the main launcher actions ([`7cd104b`](https://codeberg.org/simonrepp/hyper8/commit/7cd104bc030941ab101ac8cac066da2839b7b5fb))
- Persist launcher state after reordering bookmarks on opening a bookmarked site ([`90c1ba9`](https://codeberg.org/simonrepp/hyper8/commit/90c1ba971a170288d9c5fcf8546be875d6acc2e1))
- Fully translate the graphical launcher ([`ef03e43`](https://codeberg.org/simonrepp/hyper8/commit/ef03e4363ab384c239941d6eb67b4571efa7eeef))
- Handle horizontal and vertical overflow in the site bookmarks list ([`ece125d`](https://codeberg.org/simonrepp/hyper8/commit/ece125d4c17cc711f46154917af4d9c1e166fb48))
- Don't ask permission when removing a site bookmark in the launcher ([`5fde411`](https://codeberg.org/simonrepp/hyper8/commit/5fde4119907fc709e238937f966db953686dd5d1))
- Better place and style bookmark subactions in the launcher ([`834005c`](https://codeberg.org/simonrepp/hyper8/commit/834005c91154abe16aca29a684cab1d976e0fa25))
- Introduce configurable browser/collaborative/standalone mode for the editor ([`f3069c6`](https://codeberg.org/simonrepp/hyper8/commit/f3069c6e949a62a41444c172c268cdc5c9e5ae89))
- Implement browser preview in standalone mode (pending a potential Tauri bug) ([`8322e0f`](https://codeberg.org/simonrepp/hyper8/commit/8322e0f5c6c5c3376452c7752d058e47add5827b))
- Share editor state across launcher, editor and cli ([`4ff292e`](https://codeberg.org/simonrepp/hyper8/commit/4ff292e56c9f3fd7ebeea6bfcc01a420b49aae64))
- Provide a new theme selector in the launcher ([`eb4bceb`](https://codeberg.org/simonrepp/hyper8/commit/eb4bceb38f1b61e6bc4761b32ea3099357345b05))
- Improve naming for the different editor modes ([`68c653f`](https://codeberg.org/simonrepp/hyper8/commit/68c653fa8353ada6c5d853b44c1a6b2f99b68139))
- Split launcher command implementations into separate modules ([`c3c7d38`](https://codeberg.org/simonrepp/hyper8/commit/c3c7d38339cb5696fb2f97c398546c37a01b7ae3))
- Encapsulate context creation fully in core, revisit site_dir existence checks ([`b0e101b`](https://codeberg.org/simonrepp/hyper8/commit/b0e101b43d782afd08d053c12a1e2c6f471630e4))
- Allow accessing the command line interface in the desktop application as well ([`eec61f5`](https://codeberg.org/simonrepp/hyper8/commit/eec61f577a98b0ea7d5367c7b4bcf0d0d21c9984))
- Use local network ip in collaborative mode, allow copying, provide more hints ([`91e0c97`](https://codeberg.org/simonrepp/hyper8/commit/91e0c97a9ac48d156869a756ca219554a5827f77))
- Work around tauri opener plugin not opening preview in system browser ([`b56ff27`](https://codeberg.org/simonrepp/hyper8/commit/b56ff279d2933242ebf72ffecdeaad8b0973a95c))
- Hide scrollbars when there is no overflow in launcher on windows ([`bebca28`](https://codeberg.org/simonrepp/hyper8/commit/bebca28af4f4abd2d2163ce3826dc788e36694ca))
- Fix app/state related deadlock on opening a bookmarked site on Windows ([`854c11b`](https://codeberg.org/simonrepp/hyper8/commit/854c11bd7df2c4b04be6621df36e98c65691295e))
- Ensure ffmpeg/ffprobe discovery on PATH for macOS desktop builds ([`05f97ab`](https://codeberg.org/simonrepp/hyper8/commit/05f97abc08210dcd8226d03f31741116137f868b))

### Linking to timecodes

You can now share links to your Hyper 8 videos that include a timecode
reference. For instance a hypothetical link
[https://example.com/video/#time=1h13m59s](https://example.com/video/#time=1h13m59s) will let the linked example video
start right at 1:13:59.

- Implement seeking through hash parameters at load and runtime ([`0e29e1c`](https://codeberg.org/simonrepp/hyper8/commit/0e29e1c0b1aab8bc071081669db3141b28f9f680))

### View without server

Hyper 8 sites are built to stand the test of time: As long as a copy of your site exists somewhere
(online or offline), it can always be put back onto a regular static file server and will resume
its duty just as on the day you first published it. More than that, if you build an archive-ready
version of your site (simply the regular site with
[clean urls disabled](https://simonrepp.com/hyper8/docs/site.html#disable_clean_urls)) your site
will be browseable straight from disk, without a server being required for viewing! Due to some
technicalities, this last feature was rendered inoperable for the past months, but 0.24 brings
it back in order. Happy archiving!

- Remove ESM based script imports in build, scope all scripts within IIFEs ([`9cbe79f`](https://codeberg.org/simonrepp/hyper8/commit/9cbe79f81ede2cf7e9e9a8b58f48352dd4018cb7))

### WebKit and iOS fixes

- Fix WebKit css issues with the splashscreen and select fields in the editor ([`b47942b`](https://codeberg.org/simonrepp/hyper8/commit/b47942b353e74abb42bf7c2e55becba7068b9674))
- Hide native clear search indicator in WebKit browsers ([`5dd192e`](https://codeberg.org/simonrepp/hyper8/commit/5dd192e486219f70ccbdc87294fdf600982b2a92))
- Intercept timeline input clicks to circumvent range/value deviation in WebKit ([`1666c2c`](https://codeberg.org/simonrepp/hyper8/commit/1666c2c2225fd388aa2881a111a3b6c0416e416c))
- Handle arbitrary iOS/iPhone fullscreen behavior, improve player state handling ([`1b238e7`](https://codeberg.org/simonrepp/hyper8/commit/1b238e77cfd0d8b1fc255effb8554358011ea4e5)) (with [Matthias Bilger](https://codeberg.org/m42e))

### Misc changes and fixes

- Switch to strongly typed, on demand translatable job errors ([`16eefcf`](https://codeberg.org/simonrepp/hyper8/commit/16eefcf6694a324d8a6d2ac2e7e5e2b92597aca9))
- Synchronize latest changes in site player implementation to embed player ([`07434d1`](https://codeberg.org/simonrepp/hyper8/commit/07434d1bd6c698c355d50763cc095feed4415a4f))
- Wrap bubbles before they horizontally overflow ([`18dfb3d`](https://codeberg.org/simonrepp/hyper8/commit/18dfb3d8cae4728e11703d054dae07712a59b35b)) (with [bthj](https://sonomu.club/@aflands))
- Prevent default event action on all global shortcut invocations ([`23ad3e3`](https://codeberg.org/simonrepp/hyper8/commit/23ad3e3d7516dc0881a942aec4620465860a01c2))
- Handle race/error condition where we have no video metadata on a media subpage ([`dc20260`](https://codeberg.org/simonrepp/hyper8/commit/dc2026022006d3dac51a01e162ef3cbe757ddcde))
- Handle manifests not yet existing when their respective edit page is rendered ([`8170f5f`](https://codeberg.org/simonrepp/hyper8/commit/8170f5ffcf03249e9a1ea07fa3dc69fa7d8ddd60))
- Raise url-encoded form payload limit for the update_subtitles endpoint ([`acfc29b`](https://codeberg.org/simonrepp/hyper8/commit/acfc29bba99848ba0f0f61a77f6289b5f4b581e9)) (with [Lioh](https://social.anoxinon.de/@Lioh))
- Do not hint at opening new tabs for anchor-based/timecode links ([`abbe96a`](https://codeberg.org/simonrepp/hyper8/commit/abbe96ab6cf3b0efbb19bfcece9538d963981bf5))
- Catch git rev-parse executing but not succeeding when determining revision ([`74ff1ac`](https://codeberg.org/simonrepp/hyper8/commit/74ff1ac7cf46d5b472686d02bba4dafe1ba41180))

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
