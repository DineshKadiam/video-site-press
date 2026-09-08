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
- [roundness](#roundness)
- [ssh_config](#ssh_config)
- [theme](#theme)

## <a name="base_url"></a> `base_url`

With `base_url` you configure at which (sub)domain (respectively also
subdirectory if not at the root) your site will be hosted, which is used for
rendering certain features and for verifying that the server setup is
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

```eno
ftp_config:
mode = secure
password = SuchExquisitelyLongSecret
path = /data/web/user123/html
port = 21
server = ftp-user123.example.com
user = user123
```

To make FTP/FTPS-based site synchronization available you always need to
configure the `server`, `path` and `user` fields of `ftp_config`.

Additionally you need to provide a password in one of two ways:
1. Set `password_prompt = enabled` to be prompted a password on demand each time you
   sync your site
2. Store the password in the manifest using the `password` field – but only do
   this if your local Hyper 8 site directory is accessible only to you or people
   that you 100% trust with the FTP password, as it obviously will be plainly
   accessible to everyone with access to the `site.eno` file

Optionally you can also set these two fields:
- `port`, which otherwise uses FTP's default port 21 by default
- `mode`, to change from the default `secure` (encrypted) transmission to `insecure` –
  this should only be done in exceptional circumstances, as it will transmit
  your password and data through the network in plaintext.

In the CLI you can use the `--check-ftp` flag to safely validate your
configuration before doing a live sync. Additionally the `--simulate`
flag allows running a simulated synchronization for gaining extra confidence
before live synchronization. In the graphical editor this functionality is also
available in the top-right header menu.

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

## <a name="roundness"></a> `roundness` **EXPERIMENTAL**

*This feature is experimental – feel free to try it out, but keep in mind that
it might get changed, renamed or removed without notice in any upcoming
release, until stabilized*

```eno
roundness: 16%
```

Roundness controls how rounded or angular your site looks and feels. It rounds
or sharpens corners on various interface elements, most notably the video
player, video thumbnails and buttons.

`roundness` accepts a percent value between `0%` and `100%` (the percent sign
is optional), and its default is `80%`.

## <a name="ssh_config"></a> `ssh_config`

```eno
ssh_config:
password_prompt = enabled
path = /data/web/user123/html
port = 22
private_key = /home/example/.ssh/id_rsa
public_key = /home/example/.ssh/id_rsa.pub
server = ssh-user123.example.com
user = user123
```

To make SSH-based site synchronization available you always need to
configure the `server`, `path` and `user` fields of `ssh_config`.

The authentication method for SSH is automatically determined based on the
credentials you provide:
- If you provide no extra credentials, authentication is attempted with the
  first public key found in your SSH agent
- If you provide just a password, regular password-based authentication is
  performed. You can provide a password either by setting `password_prompt = enabled`
  and entering your password in the prompt each time you sync your site, or,
  you can store your password in your manifest using the `password` field – but only do
  this if your local Hyper 8 site directory is accessible only to you or people
  that you 100% trust with this password, as it obviously will be plainly
  accessible to everyone with access to the `site.eno` file
- If you provide at least a `private_key` (file path), key file authentication
  is performed – in this case, if configured, the `password_prompt` or
  `password` is used for unlocking the key file. A `public_key` file does not
  have to be provided if the private key file contains the public key as
  well.

Optionally you can also set the `port`, which otherwise uses SSH's default
port 22 by default.

In the CLI you can use the `--check-ssh` flag to safely validate your
configuration before doing a live synchronization. Additionally the
`--simulate` flag allows running a simulated synchronization for gaining
extra confidence before live synchronization. In the graphical editor this
functionality is also available in the top-right header menu.

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
