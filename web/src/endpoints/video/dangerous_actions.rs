// SPDX-FileCopyrightText: 2025 Simon Repp
// SPDX-License-Identifier: AGPL-3.0-or-later

use std::sync::Arc;

use actix_session::Session;
use actix_web::HttpResponse;
use actix_web::http::header::ContentType;
use actix_web::web::{Data, Path as ActixPath};
use indoc::formatdoc;

use hyper8_core::{SitePath, Video};

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

    match context.get_video(&site_path) {
        Some(video) => {
            dangerous_actions_page(
                &context,
                None,
                &session_state,
                &video
            )
        }
        None => not_found(&context, &session_state, &site_path)
    }
}

pub fn dangerous_actions_page(
    context: &Context,
    form_feedback: Option<MoveFormFeedback>,
    session_state: &SessionState,
    video: &Video
) -> HttpResponse {
    let site = context.get_site();
    let translations = &session_state.language.translations;
    let t_not_saved = translations.not_saved;

    let slug_form = if video.site_path.is_root_path() {
        String::new()
    } else {
        let mut input_permalink = Input::permalink(
            "slug",
            &video.parent_site_path_unchecked(),
            &site,
            translations
        );

        input_permalink.required();

        let feedback = if let Some(MoveFormFeedback { slug }) = &form_feedback {
            input_permalink.validated_value(slug);
            format!(r#"<span class="feedback">{t_not_saved}</span>"#)
        } else {
            input_permalink.value(video.slug_unchecked());
            String::new()
        };

        let t_a_permalink_forms_part_of_text = translations.a_permalink_forms_part_of_text;
        let t_update_permalink = translations.update_permalink;

        let move_action = routes::video_move(video);
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

    let convert_form = if site.get_any(&video.parent_site_path_unchecked()).is_playlist() {
        String::new()
    } else {
        let t_this_video_can_be_seamlessly_converted_to_a_video_in_a_collection =
            translations.this_video_can_be_seamlessly_converted_to_a_video_in_a_collection;
        let t_this_video_can_be_seamlessly_converted_to_a_video_in_a_playlist =
            translations.this_video_can_be_seamlessly_converted_to_a_video_in_a_playlist;
        let t_move_into_new_collection = translations.move_into_new_collection;
        let t_move_into_new_playlist = translations.move_into_new_playlist;

        let convert_to_collection_action = routes::convert_to_collection(&video.site_path);
        let convert_to_playlist_action = routes::convert_to_playlist(&video.site_path);

        formatdoc!(r#"
            <p>
                {t_this_video_can_be_seamlessly_converted_to_a_video_in_a_collection}
            </p>
            <form action="{convert_to_collection_action}" method="post">
                <button class="danger">{t_move_into_new_collection}</button>
            </form>
            <p>
                {t_this_video_can_be_seamlessly_converted_to_a_video_in_a_playlist}
            </p>
            <form action="{convert_to_playlist_action}" method="post">
                <button class="danger">{t_move_into_new_playlist}</button>
            </form>
        "#)
    };

    let t_dangerous_actions = translations.dangerous_actions;
    let t_delete_video = translations.delete_video;
    let t_deleting_the_video_text = translations.deleting_the_video_text;
    let t_really_delete = translations.really_delete;

    let delete_action = routes::video_delete(video);

    let body = formatdoc!(r#"
        <div class="form_group">
            <h2>{t_dangerous_actions}</h2>

            {slug_form}

            {convert_form}

            <p>
                {t_deleting_the_video_text}
            </p>

            <form
                action="{delete_action}"
                data-confirm
                data-confirm-label="{t_delete_video}"
                data-confirm-text="{t_really_delete}"
                method="post">
                <button class="danger">{t_delete_video}</button>
            </form>
        </div>
    "#);

    let video_title = video.title_or_slug_or_generic_label(translations);
    let head_title = format!("{t_dangerous_actions} – {video_title}");
    let html = layout(
        &body,
        context,
        Location::dangerous_actions(&video.site_path),
        session_state,
        &site,
        &head_title
    );

    HttpResponse::Ok()
        .content_type(ContentType::html())
        .body(html)
}
