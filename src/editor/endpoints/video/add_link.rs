// SPDX-FileCopyrightText: 2025 Simon Repp
// SPDX-License-Identifier: AGPL-3.0-or-later

use std::sync::Arc;

use actix_web::{Either, HttpResponse};
use actix_web::web::{Data, Form, Path as ActixPath, Redirect};
use serde_derive::Deserialize;
use url::Url;

use crate::{
    Context,
    Link,
    SitePath,
    Video,
    VideoUpdate
};
use crate::editor::routes;
use crate::editor::widgets::{error_page, not_found};

#[derive(Deserialize)]
pub struct AddLinkForm {
    label: String,
    url: String,
    verification: String
}

pub async fn add_link(
    context: Data<Arc<Context>>,
    form: Form<AddLinkForm>,
    site_path: ActixPath<SitePath>
) -> Either<Redirect, HttpResponse> {
    let (hidden, rel_me) = match form.verification.as_str() {
        "rel-me" => (false, true),
        "rel-me-hidden" => (true, true),
        _ => (false, false)
    };
    let label_trimmed = form.label.trim();
    let label = match label_trimmed.is_empty() {
        true => None,
        false => Some(label_trimmed.to_string())
    };
    let url = match Url::parse(&form.url) {
        Ok(_) => form.url.clone(),
        Err(err) => {
            let language = context.get_editor_language();
            let t_invalid_url = language.translations.invalid_url;
            let message = format!("{t_invalid_url} ({err})");
            return Either::Right(error_page(&context, &language, &message))
        }
    };

    let mutation = |video_mut: &mut Video| {
        let link = Link::new(hidden, label, rel_me, url);
        video_mut.links.push(link);
        video_mut.write_manifest(&context);

        Ok(())
    };

    match context.update_video(mutation, &site_path) {
        VideoUpdate::Error(_) => unreachable!("The mutation is infallible"),
        VideoUpdate::NotFound => {
            let language = context.get_editor_language();
            Either::Right(not_found(&context, &language, &site_path))
        }
        VideoUpdate::Success { video_updated, .. } => {
            let redirect_url = routes::video_edit(&video_updated);
            Either::Left(Redirect::to(redirect_url).see_other())
        }
    }
}
