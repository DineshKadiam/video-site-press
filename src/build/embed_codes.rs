// SPDX-FileCopyrightText: 2025 Simon Repp
// SPDX-License-Identifier: AGPL-3.0-or-later

use std::fs;
use url::Url;

use indoc::formatdoc;

use crate::{Context, HyperUrl, Site, Video};
use crate::build::Layout;
use crate::build::outline::Traversal;
use crate::build::widgets::{copy_button, unlisted_badge};
use crate::util::{
    html_double_escape_inside_attribute,
    html_escape_inside_attribute,
    html_escape_outside_attribute
};

use super::MergedOptions;

/// Returns a two-field tuple where the fields have the following use:
/// .0 => Intended to be directly copied via copy-to-clipboard
/// .1 => Intended to be rendered to the page, so people can copy it themselves.
/// The title parameter provides a text that indicates to screen-reader users
/// what to expect inside the iframe. See description at
/// https://developer.mozilla.org/en-US/docs/Web/HTML/Element/iframe#accessibility_concerns
fn embed_code(embed_url: &Url, title: &str) -> (String, String) {
    let title_double_escaped = html_double_escape_inside_attribute(title);
    let title_escaped = html_escape_inside_attribute(title);

    let copy_code = html_escape_inside_attribute(
        &formatdoc!(r#"
            <iframe loading="lazy" src="{embed_url}" style="min-height: 270px; min-width: 480px;" title="{title_escaped}"></iframe>
        "#)
    );

    let display_code = formatdoc!(r#"
        <div class="embed_code_wrapper">
            <pre class="embed_code"><span class="embed_syntax_special">&lt;</span>iframe
            loading<span class="embed_syntax_special">=</span><span class="embed_syntax_value">"lazy"</span>
            src<span class="embed_syntax_special">=</span><span class="embed_syntax_value">"{embed_url}"</span>
            style<span class="embed_syntax_special">=</span><span class="embed_syntax_value">"min-height: 270px; min-width: 480px;"</span>
            title<span class="embed_syntax_special">=</span><span class="embed_syntax_value">"{title_double_escaped}"</span><span class="embed_syntax_special">&gt;</span>
        <span class="embed_syntax_special">&lt;/</span>iframe<span class="embed_syntax_special">&gt;</span></pre>
        </div>
    "#);

    (copy_code, display_code)
}

/// Writes the page on which the embed html code can be viewed and copied.
pub fn embed_codes_html(
    base_url: &HyperUrl,
    context: &Context,
    merged_options: &MergedOptions,
    site: &Site,
    video: &Video,
    traversal: &Traversal
) {
    let mut layout = Layout::new();
    let index_suffix = site.index_suffix();

    let title = match &video.title {
        Some(title) => title,
        None => if video.path == "" { "Hyper 8" } else { video.slug() }
    };
    let e_title = html_escape_outside_attribute(title);

    let embed_interfix = if video.path == "" { "embed" } else { "/embed" };
    let embed_url = base_url.value
        .join(&format!("{}{embed_interfix}{index_suffix}", video.path))
        .unwrap();

    let t_video_player_widget_for_xxx = site.language.translations.video_player_widget_for_xxx(title);
    let (embed_copy_code, embed_display_code) = embed_code(&embed_url, &t_video_player_widget_for_xxx);

    let t_copy = site.language.translations.copy;
    let (copy_code_button, copy_code_templates) = copy_button("content", &embed_copy_code, t_copy, site);

    layout.add_clipboard_script();
    layout.add_templates(&copy_code_templates);

    let unlisted = unlisted_badge(merged_options.unlisted, &site.language.translations);

    let body = format!(r#"
        <div class="video_info">
            <div class="align apart flex" style="margin-top: 2rem;">
                <div>
                    <h1 class="video_title">{e_title} {unlisted}</h1>
                </div>
            </div>
                {embed_display_code}
            <div class="actions">
                {copy_code_button}
            </div>
        </div>
    "#);

    let html = layout.render(&body, site, title, traversal);

    let embed_code_dir = context.build_dir.join(&video.path).join("embed-codes");
    fs::create_dir_all(&embed_code_dir).unwrap();
    let index_path = embed_code_dir.join("index.html");
    fs::write(index_path, html).unwrap();
}
