// SPDX-FileCopyrightText: 2025 Simon Repp
// SPDX-License-Identifier: AGPL-3.0-or-later

use std::sync::Arc;

use actix_session::Session;
use actix_web::HttpResponse;
use actix_web::http::header::ContentType;
use actix_web::web::{Data, Path as ActixPath};
use indoc::formatdoc;

use hyper8_core::{Playlist, SitePath};

use crate::{
    Context,
    Field,
    Input,
    Location,
    SessionState
};
use crate::routes;
use crate::widgets::{layout, not_found};

pub struct MoveFormFeedback {
    pub slug: Field
}

pub async fn dangerous_actions(
    context: Data<Arc<Context>>,
    session: Session,
    site_path: ActixPath<SitePath>
) -> HttpResponse {
    let session_state = SessionState::all(&context.server_mode, &session);

    match context.get_playlist(&site_path) {
        Some(playlist) => {
            dangerous_actions_page(
                &context,
                None,
                &playlist,
                &session_state
            )
        }
        None => not_found(&context, &session_state, &site_path)
    }
}

pub fn dangerous_actions_page(
    context: &Context,
    form_feedback: Option<MoveFormFeedback>,
    playlist: &Playlist,
    session_state: &SessionState
) -> HttpResponse {
    let site = context.get_site();
    let translations = &session_state.language.translations;
    let t_not_saved = translations.not_saved;

    let slug_form = if playlist.site_path.is_root_path() {
        String::new()
    } else {
        let mut input_permalink = Input::permalink(
            "slug",
            &playlist.parent_site_path_unchecked(),
            &site,
            translations
        );

        input_permalink.required();

        let feedback = if let Some(MoveFormFeedback { slug }) = &form_feedback {
            input_permalink.validated_value(slug);
            format!(r#"<span class="feedback">{t_not_saved}</span>"#)
        } else {
            input_permalink.value(playlist.slug_unchecked());
            String::new()
        };

        let t_a_permalink_forms_part_of_text = translations.a_permalink_forms_part_of_text;
        let t_update_permalink = translations.update_permalink;

        let move_action = routes::playlist_move(playlist);
        formatdoc!(r#"
            <p>
                {t_a_permalink_forms_part_of_text}
            </p>

            <form action="{move_action}" data-warn-discard method="post">
                {input_permalink}

                <div style="align-items: center; column-gap: .5rem; display: flex;">
                    <button class="danger">{t_update_permalink}</button> {feedback}
                </div>
            </form>
        "#)
    };

    let t_convert_to_collection = translations.convert_to_collection;
    let convert_to_collection_action = routes::convert_to_collection(&playlist.site_path);
    let convert_to_collection_form = formatdoc!(r#"
        <p>
            A playlist can seamlessly be converted to a collection
        </p>

        <form action="{convert_to_collection_action}" method="post">
            <button class="danger">{t_convert_to_collection}</button>
        </form>
    "#);

    let convert_to_video_form = if playlist.videos.len() < 2 {
        let message = if playlist.videos.is_empty() {
            "This playlist is empty and can be seamlessly converted to a video."
        } else {
            "This playlist contains a single video and can be converted to just a video, note however that playlist data will be removed or overwritten with data from the video where necessary."
        };

        let t_convert_to_video = translations.convert_to_video;
        let convert_to_video_action = routes::convert_to_video(&playlist.site_path);
        formatdoc!(r#"
            <p>{message}</p>

            <form action="{convert_to_video_action}" method="post">
                <button class="danger">{t_convert_to_video}</button>
            </form>
        "#)
    } else {
        String::new()
    };

    let t_dangerous_actions = translations.dangerous_actions;
    let t_delete_playlist = translations.delete_playlist;
    let t_deleting_the_playlist_text = translations.deleting_the_playlist_text;
    let t_really_delete = translations.really_delete;

    let delete_action = routes::playlist_delete(playlist);

    let body = formatdoc!(r#"
        <div class="form_group">
            <h2>{t_dangerous_actions}</h2>

            {slug_form}

            {convert_to_collection_form}

            {convert_to_video_form}

            <p>
                {t_deleting_the_playlist_text}
            </p>

            <form
                action="{delete_action}"
                data-confirm
                data-confirm-label="{t_delete_playlist}"
                data-confirm-text="{t_really_delete}"
                method="post">
                <button class="danger">{t_delete_playlist}</button>
            </form>
        </div>
    "#);

    let playlist_title = playlist.title_or_slug_or_generic_label(translations);
    let head_title = format!("{t_dangerous_actions} – {playlist_title}");
    let html = layout(
        &body,
        context,
        Location::dangerous_actions(&playlist.site_path),
        session_state,
        &site,
        &head_title
    );

    HttpResponse::Ok()
        .content_type(ContentType::html())
        .body(html)
}
