// SPDX-FileCopyrightText: 2025 Simon Repp
// SPDX-License-Identifier: AGPL-3.0-or-later

use std::path::Path;
use std::sync::Arc;

use actix_web::{Either, HttpResponse};
use actix_web::web::{Data, Path as ActixPath, Redirect};

use crate::{Context, ContextPath, VideoParent};
use crate::editor::widgets::not_found;

pub async fn delete(
    context: Data<Arc<Context>>,
    path: ActixPath<ContextPath>
) -> Either<Redirect, HttpResponse> {
    let video_parent = match context.delete_video(&path.path) {
        Ok(video_parent) => video_parent,
        Err(()) => {
            let language = context.get_editor_language();
            return Either::Right(not_found(&context, &language, &path.path))
        }
    };

    let redirect_url = match video_parent {
        Some(VideoParent::Collection(collection)) => {
            Path::new("/collection").join(&collection.path).to_str().unwrap().to_string()
        }
        Some(VideoParent::Playlist(playlist)) => {
            Path::new("/playlist").join(&playlist.path).to_str().unwrap().to_string()
        }
        None => "/".to_string()
    };

    Either::Left(Redirect::to(redirect_url).see_other())
}
