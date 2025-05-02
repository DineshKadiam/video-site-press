// SPDX-FileCopyrightText: 2025 Simon Repp
// SPDX-License-Identifier: AGPL-3.0-or-later

use std::fs;
use std::sync::Arc;

use actix_web::{Either, HttpResponse};
use actix_web::web::{Data, Path as ActixPath, Redirect};

use crate::{
    Collection,
    CollectionUpdate,
    Context,
    SitePath
};
use crate::editor::routes;
use crate::editor::widgets::not_found;

pub async fn delete_banner(
    context: Data<Arc<Context>>,
    site_path: ActixPath<SitePath>
) -> Either<Redirect, HttpResponse> {
    let mutation = |collection_mut: &mut Collection| {
        if let Some(banner) = &collection_mut.banner {
            let banner_path = context.site_dir
                .join(collection_mut.site_path.filesystem_relative())
                .join(&banner.file_name);

            collection_mut.banner = None;
            collection_mut.write_manifest(&context);
            let _ = fs::remove_file(banner_path);
        }
    };

    match context.update_collection(mutation, &site_path) {
        CollectionUpdate::Updated { collection_updated, .. } => {
            let redirect_url = routes::collection_edit(&collection_updated);
            Either::Left(Redirect::to(redirect_url).see_other())
        }
        CollectionUpdate::NotFound => {
            let language = context.get_editor_language();
            Either::Right(not_found(&context, &language, &site_path))
        }
    }
}
