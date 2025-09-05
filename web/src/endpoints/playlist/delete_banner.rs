// SPDX-FileCopyrightText: 2025 Simon Repp
// SPDX-License-Identifier: AGPL-3.0-or-later

use std::fs;
use std::sync::Arc;

use actix_session::Session as ActixSession;
use actix_web::{Either, HttpResponse};
use actix_web::web::{Data, Path as ActixPath, Redirect};

use hyper8_core::{
    Playlist,
    PlaylistUpdate,
    SitePath
};

use crate::{Context, Session};
use crate::routes;
use crate::widgets::not_found;

use super::pending_manifest_errors;

pub async fn delete_banner(
    context: Data<Arc<Context>>,
    session: ActixSession,
    site_path: ActixPath<SitePath>
) -> Either<Redirect, HttpResponse> {
    let mutation = |playlist_mut: &mut Playlist| {
        if let Some(banner) = &playlist_mut.banner {
            let banner_path = context.site_dir
                .join(playlist_mut.site_path.filesystem_relative())
                .join(&banner.file_name);

            playlist_mut.banner = None;
            playlist_mut.write_manifest(&context);
            let _ = fs::remove_file(banner_path);
        }
    };

    match context.update_playlist(mutation, &site_path) {
        PlaylistUpdate::InvalidManifest { playlist } => {
            let session = Session::new(&session);
            Either::Right(pending_manifest_errors(&context, &playlist, &session))
        }
        PlaylistUpdate::NotFound => {
            let session = Session::new(&session);
            Either::Right(not_found(&context, &session, &site_path))
        }
        PlaylistUpdate::Updated { playlist_updated, .. } => {
            let redirect_url = routes::playlist_banner(&playlist_updated);
            Either::Left(Redirect::to(redirect_url).see_other())
        }
    }
}
