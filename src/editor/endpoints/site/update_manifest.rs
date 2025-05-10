// SPDX-FileCopyrightText: 2025 Simon Repp
// SPDX-License-Identifier: AGPL-3.0-or-later

use std::fs;
use std::sync::Arc;

use actix_web::{Either, HttpResponse};
use actix_web::web::{Data, Form, Redirect};
use serde_derive::Deserialize;

use crate::Context;
use crate::editor::routes;

#[derive(Deserialize)]
pub struct UpdateForm {
    content: String
}

pub async fn update_manifest(
    context: Data<Arc<Context>>,
    form: Form<UpdateForm>
) -> Either<Redirect, HttpResponse> {
    let manifest_path = context.site_dir.join("site.eno");

    fs::write(manifest_path, &form.content).unwrap();

    let site = context.get_site();
    context.update_from_filesystem(&context.site_dir, &site);

    let redirect_url = routes::SITE_MANIFEST;
    Either::Left(Redirect::to(redirect_url).see_other())
}
