// SPDX-FileCopyrightText: 2025 Simon Repp
// SPDX-License-Identifier: AGPL-3.0-or-later

use std::sync::Arc;

use actix_web::{Either, HttpResponse};
use actix_web::web::{Data, Form, Path as ActixPath, Redirect};
use serde_derive::Deserialize;

use crate::{
    Context,
    ContextPath,
    Playlist,
    PlaylistUpdate
};
use crate::editor::widgets::not_found;

#[derive(Deserialize)]
pub struct DeleteLinkForm {
    label: String,
    url: String
}

pub async fn delete_link(
    context: Data<Arc<Context>>,
    form: Form<DeleteLinkForm>,
    path: ActixPath<ContextPath>
) -> Either<Redirect, HttpResponse> {
    let mutation = |playlist_mut: &mut Playlist| {
        if playlist_mut.links
            .iter()
            .position(|link|
                if form.label.is_empty()  {
                    link.label.is_none() && link.url == form.url
                } else {
                    link.label.as_ref().is_some_and(|label| label == &form.label) && link.url == form.url
                }
            )
            .map(|index| playlist_mut.links.remove(index))
            .is_some() {
            playlist_mut.write_manifest(&context);
        }
    };

    match context.update_playlist(mutation, &path.path) {
        PlaylistUpdate::Updated { playlist_updated, .. } => {
            let redirect_url = format!("/playlist/{}", playlist_updated.path);
            Either::Left(Redirect::to(redirect_url).see_other())
        }
        PlaylistUpdate::NotFound => {
            let language = context.get_editor_language();
            Either::Right(not_found(&context, &language, &path.path))
        }
    }
}
