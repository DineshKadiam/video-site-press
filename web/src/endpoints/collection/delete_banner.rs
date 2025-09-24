// SPDX-FileCopyrightText: 2025 Simon Repp
// SPDX-License-Identifier: AGPL-3.0-or-later

use std::fs;
use std::sync::Arc;

use actix_session::Session;
use actix_web::{Either, HttpResponse};
use actix_web::web::{Data, Path as ActixPath, Redirect};

use hyper8_core::{
    Collection,
    CollectionUpdate,
    SitePath
};

use crate::{Context, SessionState};
use crate::routes;
use crate::widgets::not_found;

use super::pending_manifest_errors;

pub async fn delete_banner(
    context: Data<Arc<Context>>,
    session: Session,
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
        CollectionUpdate::InvalidManifest { collection } => {
            let session_state = SessionState::all(&context.server_mode, &session);
            Either::Right(pending_manifest_errors(&collection, &context, &session_state))
        }
        CollectionUpdate::NotFound => {
            let session_state = SessionState::all(&context.server_mode, &session);
            Either::Right(not_found(&context, &session_state, &site_path))
        }
        CollectionUpdate::Updated { collection_updated, .. } => {
            let redirect_url = routes::collection_banner(&collection_updated);
            Either::Left(Redirect::to(redirect_url).see_other())
        }
    }
}
