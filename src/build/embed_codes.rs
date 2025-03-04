// SPDX-FileCopyrightText: 2025 Simon Repp
// SPDX-License-Identifier: AGPL-3.0-or-later

use std::fs;

use indoc::formatdoc;

use crate::{Context, Site, Video};
use crate::build::{AssetHashes, Layout};
use crate::build::outline::Traversal;
use crate::build::widgets::{copy_button, unlisted_badge};
use crate::util::{
    html_double_escape_inside_attribute,
    html_escape_inside_attribute,
    html_escape_outside_attribute
};

/// Returns a two-field tuple where the fields have the following use:
/// .0 => Intended to be directly copied via copy-to-clipboard
/// .1 => Intended to be rendered to the page, so people can copy it themselves.
/// The title parameter provides a text that indicates to screen-reader users
/// what to expect inside the iframe. See description at
/// https://developer.mozilla.org/en-US/docs/Web/HTML/Element/iframe#accessibility_concerns
fn embed_code(aspect_ratio: f32, embed_url: &str, title: &str) -> (String, String) {
    let title_double_escaped = html_double_escape_inside_attribute(title);
    let title_escaped = html_escape_inside_attribute(title);

    let inline_style = format!("aspect-ratio: {aspect_ratio}; border: none; height: 100%; width: 100%;");

    let copy_code = html_escape_inside_attribute(
        &format!(r#"
            <iframe allow="fullscreen" loading="lazy" src="{embed_url}" style="{inline_style}" title="{title_escaped}"></iframe>
        "#)
    );

    let display_code = formatdoc!(r#"
        <div class="embed_code_wrapper">
            <pre class="embed_code"><span class="embed_syntax_special">&lt;</span>iframe
            allow<span class="embed_syntax_special">=</span><span class="embed_syntax_value">"fullscreen"</span>
            loading<span class="embed_syntax_special">=</span><span class="embed_syntax_value">"lazy"</span>
            src<span class="embed_syntax_special">=</span><span class="embed_syntax_value">"{embed_url}"</span>
            style<span class="embed_syntax_special">=</span><span class="embed_syntax_value">"{inline_style}"</span>
            title<span class="embed_syntax_special">=</span><span class="embed_syntax_value">"{title_double_escaped}"</span><span class="embed_syntax_special">&gt;</span>
        <span class="embed_syntax_special">&lt;/</span>iframe<span class="embed_syntax_special">&gt;</span></pre>
        </div>
    "#);

    (copy_code, display_code)
}

/// Writes the page on which the embed html code can be viewed and copied.
pub fn embed_codes_html(
    asset_hashes: &AssetHashes,
    context: &Context,
    site: &Site,
    video: &Video,
    traversal: &Traversal
) {
    let mut layout = Layout::new();
    let translations = &site.language.translations;

    let t_embed = translations.embed;
    let video_title = match &video.title {
        Some(title) => title,
        None => if video.path.is_empty() { "Video" } else { video.slug() }
    };
    let title = format!("{t_embed} – {video_title}");
    let e_title = html_escape_outside_attribute(&title);

    let embed_url = match video.path.is_empty() {
        true => site.absolute_url_unchecked("embed"),
        false => site.absolute_url_unchecked(&format!("{}/embed", video.path))
    };

    let recommended_version = video.recommended_version().unwrap();
    let aspect_ratio = recommended_version.video_meta.as_ref().unwrap().aspect_ratio();

    let t_video_player_widget_for_xxx = translations.video_player_widget_for_xxx(&title);
    let (embed_copy_code, embed_display_code) = embed_code(aspect_ratio, &embed_url, &t_video_player_widget_for_xxx);

    let t_copy = translations.copy;
    let (copy_code_button, copy_code_templates) = copy_button("content", &embed_copy_code, t_copy, site);

    layout.add_clipboard_script();
    layout.add_templates(&copy_code_templates);

    let r_unlisted_badge = if video.unlisted() {
        layout.no_indexing();
        unlisted_badge(translations)
    } else {
        String::new()
    };

    let body = format!(r#"
        <div class="video_info">
            <h1>{e_title} {r_unlisted_badge}</h1>
            {embed_display_code}
            <div class="actions">
                {copy_code_button}
            </div>
        </div>
    "#);

    let html = layout.render(
        asset_hashes,
        &body,
        site,
        &title,
        traversal
    );

    let t_embed_codes_permalink = translations.embed_codes_permalink;
    let embed_code_dir = context.build_dir.join(&video.path).join(t_embed_codes_permalink);
    fs::create_dir_all(&embed_code_dir).unwrap();
    let index_path = embed_code_dir.join("index.html");
    fs::write(index_path, html).unwrap();
}
