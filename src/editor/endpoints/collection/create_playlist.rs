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
    ContextPath,
    Field,
    Playlist
};
use crate::editor::widgets::not_found;
use crate::util::validate_slug;

use super::edit_page;
use super::CollectionFormFeedback;

#[derive(Deserialize)]
pub struct CreatePlaylistForm {
    slug: String,
    title: String
}

pub async fn create_playlist(
    context: Data<Arc<Context>>,
    form: Form<CreatePlaylistForm>,
    path: ActixPath<ContextPath>
) -> Either<Redirect, HttpResponse> {
    let language = context.get_editor_language();

    let slug_trimmed = form.slug.trim();
    let title_trimmed = form.title.trim();

    let error = |slug_error, title_error| -> _ {
        let collection = match context.get_collection(&path.path) {
            Some(collection) => collection,
            None => return Either::Right(not_found(&context, &language, &path.path))
        };

        let form_feedback = CollectionFormFeedback::CreatePlaylist {
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

    let succeed = |playlist_path: String, title: Option<&str>| {
        let playlist_dir = context.site_dir.join(&playlist_path);
        fs::create_dir_all(&playlist_dir).unwrap();

        let mutation = |collection: &mut Collection| -> Arc<Playlist> {
            let mut playlist_mut = Playlist::new(playlist_path, collection.propagate_options());
            if let Some(title) = title {
                playlist_mut.title = Some(title.to_string());
            }
            let playlist = Arc::new(playlist_mut);
            collection.playlists.push(Arc::clone(&playlist));
            playlist
        };

        match context.update_collection(mutation, &path.path) {
            CollectionUpdate::Updated { optional_return_value: playlist, .. } => {
                playlist.write_manifest(&context);

                let redirect_url = format!("/playlist/{}", playlist.path);
                Either::Left(Redirect::to(redirect_url).see_other())
            }
            CollectionUpdate::NotFound => {
                Either::Right(not_found(&context, &language, &path.path))
            }
        }
    };

    if slug_trimmed.is_empty() {
        let slug = slugify(title_trimmed);
        let mut playlist_path = if path.path.is_empty() {
            slug.to_string()
        } else {
            format!("{}/{slug}", &path.path)
        };

        let mut duplicate = 1;
        while context.site_dir.join(&playlist_path).exists() {
            playlist_path = if path.path.is_empty() {
                format!("{slug}_{duplicate}")
            } else {
                format!("{}/{slug}_{duplicate}", &path.path)
            };
            duplicate += 1;
        }

        succeed(playlist_path, Some(title_trimmed))
    } else {
        if let Err(message) = validate_slug(slug_trimmed) {
            return error(Some(&message), None);
        }

        let playlist_path = if path.path.is_empty() {
            slug_trimmed.to_string()
        } else {
            format!("{}/{slug_trimmed}", &path.path)
        };

        if context.site_dir.join(&playlist_path).exists() {
            let message = language.translations.permalink_already_used_by_another_collection_playlist_or_video;
            error(Some(message), None)
        } else {
            let title = if title_trimmed.is_empty() { None } else { Some(title_trimmed) };
            succeed(playlist_path, title)
        }
    }
}
