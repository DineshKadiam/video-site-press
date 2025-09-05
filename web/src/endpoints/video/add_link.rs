// SPDX-FileCopyrightText: 2025 Simon Repp
// SPDX-License-Identifier: AGPL-3.0-or-later

use std::sync::Arc;

use actix_session::Session as ActixSession;
use actix_web::{Either, HttpResponse};
use actix_web::web::{Data, Form, Path as ActixPath, Redirect};
use serde_derive::Deserialize;
use url::Url;

use hyper8_core::{
    Link,
    SitePath,
    Video,
    VideoUpdate
};
use hyper8_core::infallible_mutation;

use crate::{Context, Session};
use crate::routes;
use crate::widgets::{error_page, not_found};

use super::pending_manifest_errors;

#[derive(Deserialize)]
pub struct AddLinkForm {
    label: String,
    url: String,
    verification: String
}

pub async fn add_link(
    context: Data<Arc<Context>>,
    form: Form<AddLinkForm>,
    session: ActixSession,
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
            let session = Session::new(&session);
            let t_invalid_url = session.language.translations.invalid_url;
            let message = format!("{t_invalid_url} ({err})");
            return Either::Right(error_page(&context, &message, &session))
        }
    };

    let mutation = |video_mut: &mut Video| {
        let link = Link::new(hidden, label, rel_me, url);
        video_mut.links.push(link);
        video_mut.write_manifest(&context);

        Ok(())
    };

    match context.update_video(mutation, &site_path) {
        VideoUpdate::Error(_) => infallible_mutation!(),
        VideoUpdate::InvalidManifest { video } => {
            let session = Session::new(&session);
            Either::Right(pending_manifest_errors(&context, &session, &video))
        }
        VideoUpdate::NotFound => {
            let session = Session::new(&session);
            Either::Right(not_found(&context, &session, &site_path))
        }
        VideoUpdate::Success { video_updated, .. } => {
            let redirect_url = routes::video_links(&video_updated);
            Either::Left(Redirect::to(redirect_url).see_other())
        }
    }
}
