// SPDX-FileCopyrightText: 2025 Simon Repp
// SPDX-License-Identifier: AGPL-3.0-or-later

use std::fs;
use std::sync::Arc;

use actix_web::{Either, HttpResponse};
use actix_web::web::{Data, Form, Path as ActixPath, Redirect};
use serde_derive::Deserialize;

use crate::{Context, SitePath};
use crate::editor::routes;
use crate::editor::widgets::not_found;

#[derive(Deserialize)]
pub struct UpdateForm {
    content: String
}

pub async fn update_manifest(
    context: Data<Arc<Context>>,
    form: Form<UpdateForm>,
    site_path: ActixPath<SitePath>
) -> Either<Redirect, HttpResponse> {
    let language = context.get_editor_language();

    match context.get_playlist(&site_path) {
        Some(playlist) => {
            let playlist_dir = context.site_dir.join(playlist.site_path.filesystem_relative());
            let manifest_path = playlist_dir.join("playlist.eno");

            fs::write(manifest_path, &form.content).unwrap();

            let site = context.get_site();
            context.update_from_filesystem(&playlist_dir, &site);

            let redirect_url = routes::playlist_manifest(&playlist);
            Either::Left(Redirect::to(redirect_url).see_other())
        }
        None => return Either::Right(not_found(&context, &language, &site_path))
    }
}
