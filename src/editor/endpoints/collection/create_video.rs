// SPDX-FileCopyrightText: 2025 Simon Repp
// SPDX-License-Identifier: AGPL-3.0-or-later

use std::fs;
use std::sync::Arc;

use actix_web::{Either, HttpResponse};
use actix_web::web::{Data, Form, Path as ActixPath, Redirect};
use serde_derive::Deserialize;
use slug::slugify;

use crate::{
    Collection,
    CollectionUpdate,
    Context,
    Field,
    SitePath,
    Video
};
use crate::editor::routes;
use crate::editor::widgets::not_found;
use crate::util::validate_slug;

use super::edit::edit_page;
use super::edit::CollectionFormFeedback;

use super::pending_manifest_errors;

#[derive(Deserialize)]
pub struct CreateVideoForm {
    slug: String,
    title: String
}

pub async fn create_video(
    context: Data<Arc<Context>>,
    form: Form<CreateVideoForm>,
    site_path: ActixPath<SitePath>
) -> Either<Redirect, HttpResponse> {
    let language = context.get_editor_language();

    let slug_trimmed = form.slug.trim();
    let title_trimmed = form.title.trim();

    let error = |slug_error, title_error| -> _ {
        let collection = match context.get_collection(&site_path) {
            Some(collection) => collection,
            None => return Either::Right(not_found(&context, &language, &site_path))
        };

        let form_feedback = CollectionFormFeedback::CreateVideo {
            slug: Field::validated(slug_trimmed, slug_error),
            title: Field::validated(title_trimmed, title_error)
        };

        Either::Right(edit_page(&collection, &context, form_feedback, &language))
    };

    if slug_trimmed.is_empty() && title_trimmed.is_empty() {
        let t_title_or_permalink_needs_to_be_provided = language.translations.title_or_permalink_needs_to_be_provided;

        return error(
            Some(t_title_or_permalink_needs_to_be_provided),
            Some(t_title_or_permalink_needs_to_be_provided)
        );
    }

    let succeed = |video_site_path: SitePath, title: Option<&str>| {
        let video_dir = context.site_dir.join(video_site_path.filesystem_relative());
        fs::create_dir_all(&video_dir).unwrap();

        let mutation = |collection: &mut Collection| -> Arc<Video> {
            let mut video_mut = Video::new(collection.mutate_inherited_options(), video_site_path);
            if let Some(title) = title {
                video_mut.title = Some(title.to_string());
            }
            let video = Arc::new(video_mut);
            collection.videos.push(Arc::clone(&video));
            video
        };

        match context.update_collection(mutation, &site_path) {
            CollectionUpdate::InvalidManifest { collection } => {
                Either::Right(pending_manifest_errors(&collection, &context, &language))
            }
            CollectionUpdate::NotFound => {
                Either::Right(not_found(&context, &language, &site_path))
            }
            CollectionUpdate::Updated { optional_return_value: video, .. } => {
                video.write_manifest(&context);

                let redirect_url = routes::video_edit(&video);
                Either::Left(Redirect::to(redirect_url).see_other())
            }
        }
    };

    if slug_trimmed.is_empty() {
        let slug = slugify(title_trimmed);
        let mut video_site_path = site_path.join(&slug);

        let mut duplicate = 1;
        while context.site_dir.join(video_site_path.filesystem_relative()).exists() {
            video_site_path = site_path.join(&format!("{slug}_{duplicate}"));
            duplicate += 1;
        }

        succeed(video_site_path, Some(title_trimmed))
    } else {
        if let Err(message) = validate_slug(slug_trimmed) {
            return error(Some(&message), None);
        }

        let video_site_path = site_path.join(slug_trimmed);

        if context.site_dir.join(video_site_path.filesystem_relative()).exists() {
            let message = language.translations.permalink_already_used_by_another_collection_playlist_or_video;
            error(Some(message), None)
        } else {
            let title = if title_trimmed.is_empty() { None } else { Some(title_trimmed) };
            succeed(video_site_path, title)
        }
    }
}
