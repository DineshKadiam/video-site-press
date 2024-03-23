# Site

The site refers to the entirety of content in your Hyper 8 site. More
specifically the site resides at the root of the site directory, so this is
also where - and the only place where - the `site.eno` manifest can be
placed.

## The site.eno manifest

Manifest for configuring the site.

```eno
base_url: https://example.com
language: es
poster_aspect: 16:9
theme: dark
```

With `base_url` you configure at which domain your site will be hosted,
which is used for rendering certain features.

With `language` you can supply a language code (in BCP 47 format), that
will identify the site's content for browsers and screenreaders. Note
that this does not yet have any effect on user interface texts rendered
by Hyper 8, these will for now be always english, until localization is
implemented.

The `poster_aspect` option accepts either a numeric aspect ratio like `1.78`,
or a dividend/divisor notation like `16/9` or `4:3`. On a Hyper 8 site, this
controls the aspect ratios of video thumbnails everywhere. On an average
site the default `16:9` aspect ratio is a good choice, but for wider cinematic
material a wider aspect ratio is recommended, as is a portrait ratio like
for instance `9:16` for exclusively mobile video content.

The `theme` option can be set to either `adaptive`, `dark` or `light`. The
adaptive theme will dynamically present a site that is either dark or light
to visitors, depending on their system settings. Still, users are able to
manually toggle between the dark and light theme by clicking on the
respective control in the header of the page. In general the adaptive
theme is recommended as it makes the site more friendly and usable for
visitors with different needs, but of course for specific usecases this
can be weighed off - e.g. it might still make sense to configure a trailer
page for a horror movie with a dark theme only, as the imagery might be
dark itself and a light theme would potentially collide with the desired
aesthetics and visual impression intended by the creators.