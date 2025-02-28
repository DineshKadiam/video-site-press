// SPDX-FileCopyrightText: 2025 Simon Repp
// SPDX-License-Identifier: AGPL-3.0-or-later

use std::fs;
use std::fs::File;
use std::io::Write;
use std::sync::Arc;

use actix_multipart::Multipart;
use actix_web::{Either, Error, HttpResponse};
use actix_web::web;
use actix_web::web::{Data, Path as ActixPath, Redirect};
use futures_util::TryStreamExt as _;

use crate::{
    Banner,
    Collection,
    CollectionUpdate,
    Context,
    ContextPath,
    FileMeta,
    JobKind
};
use crate::editor::widgets::{error_page, not_found};
use crate::util::{nanoid, supported_image_extension};

pub async fn upload_banner(
    context: Data<Arc<Context>>,
    mut payload: Multipart,
    path: ActixPath<ContextPath>
) -> Result<Either<Redirect, HttpResponse>, Error> {
    let language = context.get_editor_language();

    while let Some(mut field) = payload.try_next().await? {
        if let Some(content_disposition) = field.content_disposition() {
            if let Some(file_name) = content_disposition.get_filename() {
                if !supported_image_extension(file_name) {
                    let message = language.translations.unsupported_file_extension_in_xxx(file_name);
                    return Ok(Either::Right(error_page(&context, &language, &message)))
                }

                let original_filename = file_name.to_string();
                // TODO: Generate a prettier filename - this goes into the site_dir!
                //       1) original_filename (if not taken) 2) only otherwise add nanoid component to filename
                let storage_filename = format!("{}_{original_filename}", nanoid());
                let storage_path = context.site_dir.join(&path.path).join(&storage_filename);
                let storage_path_moving = storage_path.clone();

                // File::create is blocking operation, use threadpool
                let mut f = web::block(|| File::create(storage_path_moving)).await??;

                while let Some(chunk) = field.try_next().await? {
                    f = web::block(move || f.write_all(&chunk).map(|_| f)).await??;
                }

                let file_meta = FileMeta::new(&storage_path);

                // TODO: Although unlikely to result in anything, most of the time,
                //       we could fetch assets from the cache, just in case?

                let banner = Banner::new(None, None, file_meta, storage_filename);

                let mutation = |collection_mut: &mut Collection| {
                    collection_mut.banner = Some(banner);
                    collection_mut.write_manifest(&context);
                };

                match context.update_collection(mutation, &path.path) {
                    CollectionUpdate::Updated { collection_updated, .. } => {
                        let job_kind = JobKind::CollectionFastTasks(Arc::clone(&collection_updated));
                        context.push_job_queue(job_kind);
                    }
                    CollectionUpdate::NotFound => {
                        let _ = fs::remove_file(&storage_path);
                        let http_response = not_found(&context, &language, &path.path);
                        return Ok(Either::Right(http_response));
                    }
                }
            }
        }
    }

    let redirect_url = format!("/collection/{}", path.path);

    Ok(Either::Left(Redirect::to(redirect_url).see_other()))
}
