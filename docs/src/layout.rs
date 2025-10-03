// SPDX-FileCopyrightText: 2024-2025 Simon Repp
// SPDX-License-Identifier: AGPL-3.0-or-later

use indoc::formatdoc;

use hyper8_version::VERSION_WITHOUT_PATCH;

use crate::{Docs, Page};
use crate::{
    COMMAND_LINE_INTERFACE_TITLE,
    DESKTOP_APPLICATION_TITLE,
    SITE_DIRECTORY_EXAMPLES_TITLE,
    SITE_DIRECTORY_TITLE
};

const ASSET_HASH_DOCS_CSS: &str = env!("HYPER8_DOCS_STATIC_ASSET_HASH_DOCS_CSS");
const ASSET_HASH_FAVICON_PNG: &str = env!("HYPER8_DOCS_STATIC_ASSET_HASH_FAVICON_PNG");

pub fn layout(
    active_page: &Page,
    body: &str,
    docs: &Docs
) -> String {
    let section_links = |pages: &[Page]| {
        pages
            .iter()
            .map(|page| {
                let active = if page == active_page { r#"class="active" "# } else { "" };
                let slug = &page.slug;
                let title = &page.title;

                format!(r#"<a {active}href="{slug}.html">{title}</a>"#)
            })
            .collect::<Vec<String>>()
            .join("\n")
    };

    let command_line_interface = section_links(&docs.command_line_interface);
    let desktop_application = section_links(&docs.desktop_application);
    let site_directory = section_links(&docs.site_directory);
    let site_directory_examples = section_links(&docs.site_directory_examples);
    let index_active = if active_page == &docs.index { "active" } else { "" };

    let logo_polychrome_icon = hyper8_icons::logo_polychrome(Some("Hyper 8"));

    let title = &active_page.title;

    formatdoc!(r##"
        <!doctype html>
        <html>
            <head>
                <title>{title}</title>
                <meta charset="utf-8">
                <meta name="description" content="{title}">
                <meta name="viewport" content="width=device-width, initial-scale=1">
                <link href="favicon.png?{ASSET_HASH_FAVICON_PNG}" rel="icon" type="image/png">
                <link href="docs.css?{ASSET_HASH_DOCS_CSS}" rel="stylesheet">
            </head>
            <body>
                <header>
                    <a class="{index_active} title" href="index.html">
                        {logo_polychrome_icon}
                        <span>Hyper 8 Video System {VERSION_WITHOUT_PATCH}</span>
                    </a>
                    <a class="open_nav" href="#nav">☰</a>
                </header>

                <div class="split">
                    <nav id="nav">
                        <div class="nav_inner">
                            <a class="close_nav" href="#">✕</a>

                            <span>{DESKTOP_APPLICATION_TITLE}</span>
                            {desktop_application}

                            <span>{COMMAND_LINE_INTERFACE_TITLE}</span>
                            {command_line_interface}

                            <span>{SITE_DIRECTORY_TITLE}</span>
                            {site_directory}

                            <span>{SITE_DIRECTORY_EXAMPLES_TITLE}</span>
                            {site_directory_examples}

                            <span>More Resources</span>
                            <a href="https://simonrepp.com/hyper8/" target="_blank">Website</a>
                            <a href="https://simonrepp.com/video/hyper8/" target="_blank">Video guides</a>
                            <a href="https://simonrepp.com/hyper8/changes/" target="_blank">Release notes</a>
                            <a href="https://codeberg.org/simonrepp/hyper8/" target="_blank">Source code</a>
                        </div>
                    </nav>
                    <main>
                        {body}
                    </main>
                </div>
            </body>
        </html>
    "##)
}
