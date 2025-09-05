// SPDX-FileCopyrightText: 2025 Simon Repp
// SPDX-License-Identifier: AGPL-3.0-or-later

use std::sync::Arc;

use actix_session::Session as ActixSession;
use actix_web::{Either, HttpResponse};
use actix_web::web::{Data, Form, Path as ActixPath, Redirect};
use serde_derive::Deserialize;

use hyper8_core::{
    Collection,
    CollectionUpdate,
    SitePath
};

use crate::{Context, Session};
use crate::routes;
use crate::widgets::not_found;

use super::pending_manifest_errors;

#[derive(Deserialize)]
pub struct BannerUpdateForm {
    description: String
}

pub async fn update_banner(
    context: Data<Arc<Context>>,
    form: Form<BannerUpdateForm>,
    session: ActixSession,
    site_path: ActixPath<SitePath>
) -> Either<Redirect, HttpResponse> {
    let mutation = |collection_mut: &mut Collection| {
        if let Some(banner) = &mut collection_mut.banner {
            banner.description = Some(form.description.clone());
        }
        collection_mut.write_manifest(&context);
    };

    match context.update_collection(mutation, &site_path) {
        CollectionUpdate::InvalidManifest { collection } => {
            let session = Session::new(&session);
            Either::Right(pending_manifest_errors(&collection, &context, &session))
        }
        CollectionUpdate::NotFound => {
            let session = Session::new(&session);
            Either::Right(not_found(&context, &session, &site_path))
        }
        CollectionUpdate::Updated { collection_updated, .. } => {
            let redirect_url = routes::collection_banner(&collection_updated);
            Either::Left(Redirect::to(redirect_url).see_other())
        }
    }
}
