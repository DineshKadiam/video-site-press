// SPDX-FileCopyrightText: 2025 Simon Repp
// SPDX-License-Identifier: AGPL-3.0-or-later

use std::sync::Arc;

use actix_web::{Either, HttpResponse};
use actix_web::web::{Data, Path as ActixPath, Redirect};

use crate::{Context, GetAnyResult, SitePath};
use crate::editor::widgets::not_found;

pub async fn delete(
    context: Data<Arc<Context>>,
    site_path: ActixPath<SitePath>
) -> Either<Redirect, HttpResponse> {
    if !context.delete_video(&site_path) {
        let language = context.get_editor_language();
        return Either::Right(not_found(&context, &language, &site_path));
    }

    let redirect_url = if site_path.is_root_path() {
        String::from("/")
    } else {
        // TODO: We could instead implement an "any/{site_path}" route that
        //       directly renders whatever is present at the given site path,
        //       or redirects to the appropriate page. (makes sense if we need
        //       this more often only).
        match context.get_any(&site_path.parent_site_path_unchecked()) {
            GetAnyResult::Collection(collection) => format!("/collection/{}", collection.site_path.normalized()),
            // None is a one-in-a-million edge case that only happens when in
            // between deleting the video and redirecting to its parent, its
            // parent was somehow removed.
            GetAnyResult::None => String::from("/"),
            GetAnyResult::Playlist(playlist) => format!("/playlist/{}", playlist.site_path.normalized()),
            // Video is a one-in-a-million edge case that only happens when in
            // between deleting the video and redirecting to its parent, its
            // parent was somehow replaced by a video.
            GetAnyResult::Video(video) => format!("/video/{}", video.site_path.normalized())
        }
    };

    Either::Left(Redirect::to(redirect_url).see_other())
}
