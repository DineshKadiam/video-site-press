// SPDX-FileCopyrightText: 2025 Simon Repp
// SPDX-License-Identifier: AGPL-3.0-or-later

use std::fs;
use std::sync::Arc;

use actix_web::{Either, HttpResponse};
use actix_web::web::{Data, Form, Path as ActixPath, Redirect};
use serde_derive::Deserialize;
use slug::slugify;

use crate::{Context, ContextPath, Field, Playlist, Video};
use crate::editor::widgets::not_found;
use crate::util::validate_slug;

use super::edit_page;
use super::PlaylistFormFeedback;

#[derive(Deserialize)]
pub struct CreateVideoForm {
    slug: String,
    title: String
}

pub async fn create_video(
    context: Data<Arc<Context>>,
    form: Form<CreateVideoForm>,
    path: ActixPath<ContextPath>
) -> Either<Redirect, HttpResponse> {
    let language = context.get_editor_language();

    let slug_trimmed = form.slug.trim();
    let title_trimmed = form.title.trim();

    let error = |slug_error, title_error| -> _ {
        let playlist = match context.get_playlist(&path.path) {
            Some(playlist) => playlist,
            None => return Either::Right(not_found(&context, &language, &path.path))
        };

        let form_feedback = PlaylistFormFeedback::CreateVideo {
            slug: Field::validated(slug_trimmed, slug_error),
            title: Field::validated(title_trimmed, title_error)
        };

        Either::Right(edit_page(&context, Some(form_feedback), &language, &playlist))
    };

    if slug_trimmed.is_empty() && title_trimmed.is_empty() {
        let t_title_or_permalink_needs_to_be_provided = language.translations.title_or_permalink_needs_to_be_provided;

        return error(
            Some(t_title_or_permalink_needs_to_be_provided),
            Some(t_title_or_permalink_needs_to_be_provided)
        );
    }

    let succeed = |video_path: String, title: Option<&str>| {
        let video_dir = context.site_dir.join(&video_path);
        fs::create_dir_all(&video_dir).unwrap();

        let mut video_mut = Video::new(video_path);

        if let Some(title) = title {
            video_mut.title = Some(title.to_string());
        }

        let video = Arc::new(video_mut);

        let video_moving = Arc::clone(&video);
        let mutation = |playlist: &mut Playlist| {
            playlist.videos.push(video_moving);
        };

        if context.update_playlist(mutation, &path.path).is_err() {
            return Either::Right(not_found(&context, &language, &path.path));
        }

        video.write_manifest(&context);

        let redirect_url = format!("/video/{}", video.path);
        Either::Left(Redirect::to(redirect_url).see_other())
    };

    if slug_trimmed.is_empty() {
        let slug = slugify(title_trimmed);
        let mut video_path = if &path.path == "" {
            slug.to_string()
        } else {
            format!("{}/{slug}", &path.path)
        };

        let mut duplicate = 1;
        while context.site_dir.join(&video_path).exists() {
            video_path = if &path.path == "" {
                format!("{slug}_{duplicate}")
            } else {
                format!("{}/{slug}_{duplicate}", &path.path)
            };
            duplicate += 1;
        }

        return succeed(video_path, Some(title_trimmed));
    } else {
        if let Err(message) = validate_slug(slug_trimmed) {
            return error(Some(&message), None);
        }

        let video_path = if &path.path == "" {
            slug_trimmed.to_string()
        } else {
            format!("{}/{slug_trimmed}", &path.path)
        };

        if context.site_dir.join(&video_path).exists() {
            let message = language.translations.permalink_already_used_by_another_collection_playlist_or_video;
            return error(Some(message), None);
        } else {
            let title = if title_trimmed.is_empty() { None } else { Some(title_trimmed) };
            return succeed(video_path, title);
        }
    }
}
