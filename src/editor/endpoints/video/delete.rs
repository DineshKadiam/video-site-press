// SPDX-FileCopyrightText: 2025 Simon Repp
// SPDX-License-Identifier: AGPL-3.0-or-later

use std::sync::Arc;

use actix_web::{Either, HttpResponse};
use actix_web::web::{Data, Path as ActixPath, Redirect};

use crate::{Context, SitePath, VideoParent};
use crate::editor::widgets::not_found;

pub async fn delete(
    context: Data<Arc<Context>>,
    site_path: ActixPath<SitePath>
) -> Either<Redirect, HttpResponse> {
    let video_parent = match context.delete_video(&site_path) {
        Ok(video_parent) => video_parent,
        Err(()) => {
            let language = context.get_editor_language();
            return Either::Right(not_found(&context, &language, &site_path))
        }
    };

    let redirect_url = match video_parent {
        Some(VideoParent::Collection(collection)) => {
            format!("/collection/{}", collection.site_path.normalized())
        }
        Some(VideoParent::Playlist(playlist)) => {
            format!("/playlist/{}", playlist.site_path.normalized())
        }
        None => String::from("/")
    };

    Either::Left(Redirect::to(redirect_url).see_other())
}
