// SPDX-FileCopyrightText: 2025 Simon Repp
// SPDX-License-Identifier: AGPL-3.0-or-later

use std::sync::Arc;

use actix_web::{Either, HttpResponse};
use actix_web::web::{Data, Form, Path as ActixPath, Redirect};
use serde_derive::Deserialize;

use crate::{Context, ContextPath, Video};
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
    let mutation = |video_mut: &mut Video| {
        if video_mut.links
            .iter()
            .position(|link|
                if form.label.is_empty()  {
                    link.label.is_none() && link.url == form.url
                } else {
                    link.label.as_ref().is_some_and(|label| label == &form.label) && link.url == form.url
                }
            )
            .map(|index| video_mut.links.remove(index))
            .is_some() {
            video_mut.write_manifest(&context);
        }
    };

    match context.update_video(mutation, &path.path) {
        Ok(video) => {
            let redirect_url = format!("/video/{}", video.path);
            Either::Left(Redirect::to(redirect_url).see_other())
        }
        Err(()) => {
            let language = context.get_editor_language();
            Either::Right(not_found(&context, &language, &path.path))
        }
    }
}
