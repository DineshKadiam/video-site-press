// SPDX-FileCopyrightText: 2025 Simon Repp
// SPDX-License-Identifier: AGPL-3.0-or-later

use std::sync::Arc;

use actix_web::{Either, HttpResponse};
use actix_web::web::{Data, Form, Path as ActixPath, Redirect};
use serde_derive::Deserialize;

use crate::{Context, ContextPath, Playlist};
use crate::editor::widgets::not_found;

#[derive(Deserialize)]
pub struct BannerUpdateForm {
    description: String
}

pub async fn update_banner(
    context: Data<Arc<Context>>,
    form: Form<BannerUpdateForm>,
    path: ActixPath<ContextPath>
) -> Either<Redirect, HttpResponse> {
    let mutation = |playlist_mut: &mut Playlist| {
        if let Some(banner) = &mut playlist_mut.banner {
            banner.description = Some(form.description.clone());
        }
        playlist_mut.write_manifest(&context);
    };

    match context.update_playlist(mutation, &path.path) {
        Ok(playlist) => {
            let redirect_url = format!("/playlist/{}", playlist.path);
            Either::Left(Redirect::to(redirect_url).see_other())
        }
        Err(()) => {
            let language = context.get_editor_language();
            return Either::Right(not_found(&context, &language, &path.path))
        }
    }
}
