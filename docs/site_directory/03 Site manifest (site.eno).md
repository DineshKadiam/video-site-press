<!--
    SPDX-FileCopyrightText: 2024-2026 Simon Repp
    SPDX-License-Identifier: CC0-1.0
-->

# The site manifest (site.eno)

Manifest for configuring the site.

An example manifest:

```eno
base_url: https://example.com
disable_clean_urls
language: es
poster_aspect: 16:9
theme: adaptive_cool

ftp_config:
mode = secure
password = SuchExquisitelyLongSecret
path = /data/web/user123/html
port = 21
server = ftp-user123.example.com
user = user123
```

Jump links for all options:

- [base_url](#base_url)
- [disable_clean_urls](#disable_clean_urls)
- [ftp_config](#ftp_config)
- [language](#language)
- [poster_aspect](#poster_aspect)
- [ssh_config](#ssh_config)
- [theme](#theme)

## <a name="base_url"></a> `base_url`

With `base_url` you configure at which (sub)domain (respectively also
subdirectory if not at the root) your site will be hosted, which is used for
rendering certain features and for verifying that the deployment settings are
correct.

```eno
base_url: https://example.com
```

## <a name="disable_clean_urls"></a> `disable_clean_urls`

With `disable_clean_urls` Hyper 8 can be instructed to create a build with
fullly verbose internal (file) linking, meaning that each link on the site
ends on `/index.html` instead of just `/` (e.g. `https://example.com/collection/index.html`
instead of `https://example.com/collection/`). Although this arguably looks
a little more raw, it has the advantage of links working in the widest range
of environments - especially it enables the build to be browseable directly
from disk without any webserver inbetween, which makes a lot of sense if
you're planning to archive your page somewhere offline.

```eno
disable_clean_urls
```

## <a name="ftp_config"></a> `ftp_config`

With `ftp_config` you can configure the remote `server`, `path`, `user`,
`password`, `port` and `mode` for FTP/SFTP deployment.

In order to prevent accidental deployments to wrong remote servers or paths,
deployment requires the [base_url](#base_url) to be specified and the server to be set
up so the contents at `ftp_config.path` are already accessible through the
public [base_url](#base_url). Before deployment, Hyper 8 will do a test upload of a text
file to the remote path and check if it's accessible through the public url -
if it isn't, the test file is removed and the deployment aborted.

```eno
ftp_config:
mode = secure
password = SuchExquisitelyLongSecret
path = /data/web/user123/html
port = 21
server = ftp-user123.example.com
user = user123
```

The `mode` option can be `secure` (also the default) or `insecure`, but the
latter should only be used in exceptional circumstances, as it will transmit
your password and all transferred data through the network in plaintext.

The `port` is optional, by default Hyper 8 will use the default FTP port (21).

Note that you should only configure and use FTP deployment through Hyper 8 if
your local Hyper 8 site directory is accessible only to you or people that
you 100% trust with the FTP password, as it obviously will be plainly
accessible to everyone with access to the `site.eno` file.

An on-demand password prompt option will be implemented in the future to also
allow deployment in environments where the passwords needs to be kept hidden
from other people with access to the site directory.

## <a name="language"></a> `language`

With `language` you can supply a language code (in BCP 47 format) that will
identify the site's content for browsers and screenreaders. For supported
languages (currently english and german - contributions welcome!) all
implemented texts will show up as translated in the final website. The Hyper
8 editor itself is also usable in different languages (although this is only
rudimentarily developed right now!), for this see the command line arguments
documentation.

```eno
language: es
```

## <a name="poster_aspect"></a> `poster_aspect`

The `poster_aspect` option accepts either a numeric aspect ratio like `1.78`,
or a dividend/divisor notation like `16/9` or `4:3`. On a Hyper 8 site, this
controls the aspect ratios of video thumbnails everywhere. On an average
site the default `16:9` aspect ratio is a good choice, but for wider cinematic
material a wider aspect ratio is recommended, as is a portrait ratio like
for instance `9:16` for exclusively mobile video content.

```eno
poster_aspect: 16:9
```

## <a name="ssh_config"></a> `ssh_config`

With `ssh_config` you can configure the remote `server`, `password`, `path`, `port`
and `user` for SSH-based deployment.

In order to prevent accidental deployments to wrong remote servers and paths,
deployment requires the [base_url](#base_url) to be specified and the server to be set
up so the contents at `ssh_config.path` are already accessible through the
public [base_url](#base_url). Before deployment, Hyper 8 will do a test upload of a text file
to the remote path and check if it's accessible through the public url - if
it isn't, the test file is removed and the deployment aborted.

```eno
ssh_config:
password = correcthorsebatterystaple
path = /data/web/user123/html
port = 22
server = ssh-user123.example.com
user = user123
```

## <a name="theme"></a> `theme`

```eno
theme: adaptive_cool
```

The `theme` option can be set to any of these:
- `adaptive_cool`
- `adaptive_deep`
- `adaptive_neutral`
- `adaptive_warm`
- `dark_cool`
- `dark_deep`
- `dark_neutral`
- `dark_warm`
- `light_cool`
- `light_deep`
- `light_neutral`
- `light_warm`

Adaptive themes will dynamically present a site that is either dark or light
to visitors, depending on their system settings, but users are then still able to
manually toggle between the dark and light theme by clicking on the
respective control on the page. In general the adaptive theme is recommended
as it makes the site more friendly and usable for visitors with different
needs, but of course for specific usecases this can be weighed off - e.g. it
might still make sense to configure a trailer page for a horror movie with a
dark theme only, as the imagery might be dark itself and a light theme would
potentially collide with the desired aesthetics and visual impression intended.
