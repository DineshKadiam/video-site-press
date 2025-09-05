// SPDX-FileCopyrightText: 2025 Simon Repp
// SPDX-License-Identifier: AGPL-3.0-or-later

use std::fs;
use std::sync::Arc;

use actix_session::Session as ActixSession;
use actix_web::{Either, HttpResponse};
use actix_web::web::{Data, Form, Path as ActixPath, Redirect};
use serde_derive::Deserialize;
use slug::slugify;

use hyper8_core::{
    Collection,
    CollectionUpdate,
    Playlist,
    SitePath
};
use hyper8_core::util::validate_slug;

use crate::{Context, Field, Session};
use crate::routes;
use crate::widgets::not_found;

use super::edit::edit_page;
use super::edit::CollectionFormFeedback;

use super::pending_manifest_errors;

#[derive(Deserialize)]
pub struct CreatePlaylistForm {
    slug: String,
    title: String
}

pub async fn create_playlist(
    context: Data<Arc<Context>>,
    form: Form<CreatePlaylistForm>,
    session: ActixSession,
    site_path: ActixPath<SitePath>
) -> Either<Redirect, HttpResponse> {
    let session = Session::new(&session);

    let slug_trimmed = form.slug.trim();
    let title_trimmed = form.title.trim();

    let error = |slug_error, title_error| -> _ {
        let collection = match context.get_collection(&site_path) {
            Some(collection) => collection,
            None => return Either::Right(not_found(&context, &session, &site_path))
        };

        let form_feedback = CollectionFormFeedback::CreatePlaylist {
            slug: Field::validated(slug_trimmed, slug_error),
            title: Field::validated(title_trimmed, title_error)
        };

        Either::Right(edit_page(&collection, &context, form_feedback, &session))
    };

    if slug_trimmed.is_empty() && title_trimmed.is_empty() {
        let t_title_or_permalink_needs_to_be_provided = session.language.translations.title_or_permalink_needs_to_be_provided;

        return error(
            Some(t_title_or_permalink_needs_to_be_provided),
            Some(t_title_or_permalink_needs_to_be_provided)
        );
    }

    let succeed = |playlist_site_path: SitePath, title: Option<&str>| {
        let playlist_dir = context.site_dir.join(playlist_site_path.filesystem_relative());
        fs::create_dir_all(&playlist_dir).unwrap();

        let mutation = |collection: &mut Collection| -> Arc<Playlist> {
            let mut playlist_mut = Playlist::new(collection.mutate_inherited_options(), playlist_site_path);
            if let Some(title) = title {
                playlist_mut.title = Some(title.to_string());
            }
            let playlist = Arc::new(playlist_mut);
            collection.playlists.push(Arc::clone(&playlist));
            playlist
        };

        match context.update_collection(mutation, &site_path) {
            CollectionUpdate::InvalidManifest { collection } => {
                Either::Right(pending_manifest_errors(&collection, &context, &session))
            }
            CollectionUpdate::NotFound => {
                Either::Right(not_found(&context, &session, &site_path))
            }
            CollectionUpdate::Updated { optional_return_value: playlist, .. } => {
                playlist.write_manifest(&context);

                let redirect_url = routes::playlist_edit(&playlist);
                Either::Left(Redirect::to(redirect_url).see_other())
            }
        }
    };

    if slug_trimmed.is_empty() {
        let slug = slugify(title_trimmed);
        let mut playlist_site_path = site_path.join(&slug);

        let mut duplicate = 1;
        while context.site_dir.join(playlist_site_path.filesystem_relative()).exists() {
            playlist_site_path = site_path.join(&format!("{slug}_{duplicate}"));
            duplicate += 1;
        }

        succeed(playlist_site_path, Some(title_trimmed))
    } else {
        if let Err(message) = validate_slug(slug_trimmed) {
            return error(Some(&message), None);
        }

        let playlist_site_path = site_path.join(slug_trimmed);

        if context.site_dir.join(playlist_site_path.filesystem_relative()).exists() {
            let message = session.language.translations.permalink_already_used_by_another_collection_playlist_or_video;
            error(Some(message), None)
        } else {
            let title = if title_trimmed.is_empty() { None } else { Some(title_trimmed) };
            succeed(playlist_site_path, title)
        }
    }
}
