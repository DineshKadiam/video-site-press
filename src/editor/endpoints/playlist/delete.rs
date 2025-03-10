// SPDX-FileCopyrightText: 2025 Simon Repp
// SPDX-License-Identifier: AGPL-3.0-or-later

use std::sync::Arc;

use actix_web::{Either, HttpResponse};
use actix_web::web::{Data, Path as ActixPath, Redirect};

use crate::{Context, SitePath};
use crate::editor::widgets::not_found;

pub async fn delete(
    context: Data<Arc<Context>>,
    site_path: ActixPath<SitePath>
) -> Either<Redirect, HttpResponse> {
    if !context.delete_playlist(&site_path) {
        let language = context.get_editor_language();
        return Either::Right(not_found(&context, &language, &site_path))
    };

    let redirect_url = if site_path.is_root_path() {
        String::from("/")
    } else {
        format!("/collection/{}", site_path.parent_site_path_unchecked().normalized())
    };

    Either::Left(Redirect::to(redirect_url).see_other())
}
