// SPDX-FileCopyrightText: 2026 Simon Repp
// SPDX-License-Identifier: AGPL-3.0-or-later

use std::sync::Arc;

use actix_session::Session;
use actix_web::HttpResponse;
use actix_web::http::header::ContentType;
use actix_web::web::{Data, Path as ActixPath};
use indoc::formatdoc;

use hyper8_core::SitePath;
use hyper8_util::{html_escape_inside_attribute, js_escape_inside_double_quoted_string};

use crate::{
    Context,
    Layout,
    Location,
    SessionState
};
use crate::routes;
use crate::widgets::not_found;

pub async fn chapters(
    context: Data<Arc<Context>>,
    session: Session,
    site_path: ActixPath<SitePath>
) -> HttpResponse {
    let session_state = SessionState::all(&context.server_mode, &session);
    let translations = &session_state.language.translations;

    let video = match context.get_video(&site_path) {
        Some(video) => video,
        None => return not_found(&context, &session_state, &site_path)
    };

    let site = context.get_site();

    let head_title = video.title_or_slug_or_generic_label(translations);

    let t_chapters = translations.chapters;

    let delete_icon = hyper8_icons::failure(translations.delete);
    let down_icon = hyper8_icons::chevron_down(translations.down);
    let up_icon = hyper8_icons::chevron_up(translations.up);

    let data_chapters = if let Some(chapters) = &video.chapters && !chapters.is_empty() {
        chapters
            .iter()
            .map(|chapter| {
                let mut props = Vec::new();

                let begin = &chapter.begin.representation;
                props.push(format!(r#""begin":"{begin}""#));

                if let Some(end) = &chapter.end {
                    let end = &end.representation;
                    props.push(format!(r#""end":"{end}""#));
                }

                let title = js_escape_inside_double_quoted_string(&chapter.title);
                props.push(format!(r#""title":"{title}""#));

                let props_escaped = html_escape_inside_attribute(&props.join(","));

                format!("{{{props_escaped}}}")
            })
            .collect::<Vec<_>>()
            .join(",")
    } else {
        String::new()
    };

    let update_url = routes::video_update_chapters(&video);

    let body = formatdoc!(r#"
        <h2>{t_chapters}</h2>

        <div class="chapters" data-chapters="[{data_chapters}]"></div>

        <div class="form_group">
            <form action="{update_url}" method="post">
                <input name="data" type="hidden">
                <button class="action" data-save-chapters>Save chapters</button>
            </form>
        </div>

        <div>
            Supported begin/end timecode formats for chapters:
            <table>
                <tr>
                    <td><code>00:03:18,608</code></td>
                    <td>hh:mm:ss,fff</td>
                    <td><a href="https://en.wikipedia.org/wiki/SubRip">SubRip (srt)</a></td>
                </tr>
                <tr>
                    <td><code>00:03:18.608</code></td>
                    <td>hh:mm:ss.fff</td>
                    <td><a href="https://developer.mozilla.org/en-US/docs/Web/API/WebVTT_API/Web_Video_Text_Tracks_Format#cue_timings">WebVTT</a></td>
                </tr>
                <tr>
                    <td><code>03:18.608</code></td>
                    <td>mm:ss.fff</td>
                    <td><a href="https://developer.mozilla.org/en-US/docs/Web/API/WebVTT_API/Web_Video_Text_Tracks_Format#cue_timings">WebVTT</a></td>
                </tr>
                <tr>
                    <td><code>30s</code></td>
                    <td>(s)econds</td>
                    <td>Hypertime</td>
                </tr>
                <tr>
                    <td><code>30s400ms</code></td>
                    <td>(s)econds+(m)illi(s)econds</td>
                    <td>Hypertime</td>
                </tr>
                <tr>
                    <td><code>2m</code></td>
                    <td>(m)inutes</td>
                    <td>Hypertime</td>
                </tr>
                <tr>
                    <td><code>2m30s</code></td>
                    <td>(m)inutes+(s)econds</td>
                    <td>Hypertime</td>
                </tr>
                <tr>
                    <td><code>1h</code></td>
                    <td>(h)ours</td>
                    <td>Hypertime</td>
                </tr>
                <tr>
                    <td><code>1h30s</code></td>
                    <td>(h)ours+(s)econds</td>
                    <td>Hypertime</td>
                </tr>
                <tr>
                    <td><code>1h2m</code></td>
                    <td>(h)ours+(m)inutes</td>
                    <td>Hypertime</td>
                </tr>
                <tr>
                    <td><code>1h2m30s</code></td>
                    <td>(h)ours+(m)inutes+(s)econds</td>
                    <td>Hypertime</td>
                </tr>
                <tr>
                    <td><code>1h2m30s400ms</code></td>
                    <td>(h)ours+(m)inutes+(s)econds+(m)illi(s)econds</td>
                    <td>Hypertime</td>
                </tr>
                <tr>
                    <td><code>413</code></td>
                    <td>raw seconds without unit</td>
                    <td>Hypertime</td>
                </tr>
            </table>
        </div>
        <template data-delete-icon>{delete_icon}</template>
        <template data-down-icon>{down_icon}</template>
        <template data-up-icon>{up_icon}</template>
    "#);

    let layout = Layout::new();

    let html = layout.render(
        &body,
        &context,
        Location::chapters(&video.site_path),
        &session_state,
        &site,
        head_title
    );

    HttpResponse::Ok()
        .content_type(ContentType::html())
        .body(html)
}
