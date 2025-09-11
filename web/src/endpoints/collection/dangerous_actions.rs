// SPDX-FileCopyrightText: 2025 Simon Repp
// SPDX-License-Identifier: AGPL-3.0-or-later

use std::sync::Arc;

use actix_session::Session;
use actix_web::HttpResponse;
use actix_web::http::header::ContentType;
use actix_web::web::{Data, Path as ActixPath};
use indoc::formatdoc;

use hyper8_core::{Collection, SitePath};

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
    let session_state = SessionState::all(&context.editor_state, &session);

    match context.get_collection(&site_path) {
        Some(collection) => {
            dangerous_actions_page(
                &collection,
                &context,
                None,
                &session_state
            )
        }
        None => not_found(&context, &session_state, &site_path)
    }
}

pub fn dangerous_actions_page(
    collection: &Collection,
    context: &Context,
    form_feedback: Option<MoveFormFeedback>,
    session_state: &SessionState
) -> HttpResponse {
    let site = context.get_site();
    let translations = &session_state.language.translations;
    let t_not_saved = translations.not_saved;

    let slug_form = if collection.site_path.is_root_path() {
        String::new()
    } else {
        let mut input_permalink = Input::permalink(
            "slug",
            &collection.parent_site_path_unchecked(),
            &site,
            translations
        );

        input_permalink
            .id("slug_collection")
            .required();

        let feedback = if let Some(MoveFormFeedback { slug }) = &form_feedback {
            input_permalink.validated_value(slug);
            format!(r#"<span class="feedback">{t_not_saved}</span>"#)
        } else {
            input_permalink.value(collection.slug_unchecked());
            String::new()
        };

        let t_a_permalink_forms_part_of_text = translations.a_permalink_forms_part_of_text;
        let t_update_permalink = translations.update_permalink;

        let move_action = routes::collection_move(collection);
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

    let convert_to_playlist_form = if collection.playlists.is_empty() && collection.subcollections.is_empty() {
        let t_convert_to_playlist = translations.convert_to_playlist;
        let convert_to_playlist_action = routes::convert_to_playlist(&collection.site_path);
        formatdoc!(r#"
            <p>
                This collection contains no playlists or subcollections and can therefore
                seamlessly be converted to a playlist.
            </p>

            <form action="{convert_to_playlist_action}" method="post">
                <button class="danger">{t_convert_to_playlist}</button>
            </form>
        "#)
    } else {
        String::new()
    };

    let convert_to_video_form = if
        collection.playlists.is_empty() &&
        collection.subcollections.is_empty() &&
        collection.videos.len() < 2 {
        let message = if collection.videos.is_empty() {
            "This collection contains nothing and can therefore seamlessly be converted to a video."
        } else {
            "This collection contains only a single video and can therefore seamlessly be converted to a video."
        };

        let t_convert_to_video = translations.convert_to_video;
        let convert_to_video_action = routes::convert_to_video(&collection.site_path);
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
    let t_delete_collection = translations.delete_collection;
    let t_deleting_the_collection_text = translations.deleting_the_collection_text;
    let t_really_delete = translations.really_delete;

    let delete_action = routes::collection_delete(collection);

    let body = formatdoc!(r#"
        <div class="form_group">
            <h2>{t_dangerous_actions}</h2>

            {slug_form}

            {convert_to_playlist_form}

            {convert_to_video_form}

            <p>
                {t_deleting_the_collection_text}
            </p>

            <form
                action="{delete_action}"
                data-confirm
                data-confirm-label="{t_delete_collection}"
                data-confirm-text="{t_really_delete}"
                method="post">
                <button class="danger">{t_delete_collection}</button>
            </form>
        </div>
    "#);

    let collection_title = collection.title_or_slug_or_generic_label(translations);
    let head_title = format!("{t_dangerous_actions} – {collection_title}");
    let html = layout(
        &body,
        context,
        Location::dangerous_actions(&collection.site_path),
        session_state,
        &site,
        &head_title
    );

    HttpResponse::Ok()
        .content_type(ContentType::html())
        .body(html)
}
