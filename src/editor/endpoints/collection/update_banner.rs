// SPDX-FileCopyrightText: 2025 Simon Repp
// SPDX-License-Identifier: AGPL-3.0-or-later

use std::sync::Arc;

use actix_web::{Either, HttpResponse};
use actix_web::web::{Data, Form, Path as ActixPath, Redirect};
use serde_derive::Deserialize;

use crate::{
    Collection,
    CollectionUpdate,
    Context,
    ContextPath
};
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
    let mutation = |collection_mut: &mut Collection| {
        if let Some(banner) = &mut collection_mut.banner {
            banner.description = Some(form.description.clone());
        }
        collection_mut.write_manifest(&context);
    };

    match context.update_collection(mutation, &path.path) {
        CollectionUpdate::Updated { collection_updated, .. } => {
            let redirect_url = format!("/collection/{}", collection_updated.path);
            Either::Left(Redirect::to(redirect_url).see_other())
        }
        CollectionUpdate::NotFound => {
            let language = context.get_editor_language();
            Either::Right(not_found(&context, &language, &path.path))
        }
    }
}
