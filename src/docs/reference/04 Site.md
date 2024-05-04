# Site

The site refers to the entirety of content in your Hyper 8 site. More
specifically the site resides at the root of the site directory, so this is
also where - and the only place where - the `site.eno` manifest can be
placed.

## The site.eno manifest

Manifest for configuring the site.

```eno
base_url: https://example.com
disable_clean_urls
language: es
poster_aspect: 16:9
theme: dark

deploy_config:
path = /data/web/customer123/html
server = ssh-customer123.example.com
user = customer123
```

With `base_url` you configure at which domain your site will be hosted,
which is used for rendering certain features.

With `language` you can supply a language code (in BCP 47 format) that will
identify the site's content for browsers and screenreaders. For supported
languages (currently english and german - contributions welcome!) all
implemented texts will show up as translated in the final website. The Hyper
8 editor itself is also usable in different languages (although this is only
rudimentarily developed right now!), for this see the command line arguments
documentation.

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

With `deploy_config` you can configure the (remote) `path`, `server` and `user`
for an ssh/rsync based deployment. Note that for now only password-less
(ssh public/private key) authentication is possible. In order to prevent
accidental deployments to wrong remote servers and paths, deployment requires
the `base_url` to be specified and the server to be set up so the contents
at `deploy_config.path` are already accessible through the public `base_url`.
(On deploment, Hyper 8 will do a test upload of a text file to the remote
path and check if it's accessible through the public url - if it isn't, the
file is removed and the deployment aborted).

With `disable_clean_urls` Hyper 8 can be instructed to create a build with
fullly verbose internal (file) linking, meaning that each link on the site
ends on `/index.html` instead of just `/` (e.g. `https://example.com/collection/index.html`
instead of `https://example.com/collection/`). Although this arguably looks
a little more raw, it has the advantage of links working in the widest range
of environments - especially it enables the build to be browseable directly
from disk without any webserver inbetween, which makes a lot of sense if
you're planning to archive your page somewhere offline.
