// SPDX-FileCopyrightText: 2025 Simon Repp
// SPDX-License-Identifier: AGPL-3.0-or-later

use std::fs;
use std::fs::File;
use std::io::Write;
use std::path::Path;
use std::sync::Arc;

use actix_multipart::Multipart;
use actix_web::{Either, Error, HttpResponse};
use actix_web::web;
use actix_web::web::{Data, Path as ActixPath, Redirect};
use futures_util::TryStreamExt as _;
use slug::slugify;

use crate::{
    Collection,
    CollectionUpdate,
    Context,
    FileMeta,
    GetAnyResult,
    HyperDir,
    JobKind,
    Playlist,
    PlaylistUpdate,
    SitePath,
    Video,
    VideoFile
};
use crate::editor::widgets::not_found;
use crate::util::{nanoid, supported_video_extension};

// TODO: Implementation wise this is very messy, refactoring candidate
pub async fn upload_video(
    context: Data<Arc<Context>>,
    mut payload: Multipart,
    site_path: ActixPath<SitePath>
) -> Result<Either<Redirect, HttpResponse>, Error> {
    let language = context.get_editor_language();

    let container = context.get_any(&site_path);

    if let GetAnyResult::None = container {
        let http_response = not_found(&context, &language, &site_path);
        return Ok(Either::Right(http_response));
    }

    while let Some(mut field) = payload.try_next().await? {
        if let Some(content_disposition) = field.content_disposition() {
            if let Some(filename) = content_disposition.get_filename() {
                // TODO: Support audio formats as well?
                //       Or should this be a separate endpoint/feature?
                if supported_video_extension(filename) {
                    let original_filename = filename.to_string();

                    let storage_path = match &container {
                        GetAnyResult::Collection(collection) => {
                            let mut candidate_dir_name = slugify(Path::new(&original_filename).with_extension("").to_string_lossy());
                            while context.site_dir.join(collection.site_path.filesystem_relative()).join(&candidate_dir_name).exists() {
                                candidate_dir_name.push_str("-duplicate");
                            }
                            let candidate_dir = context.site_dir.join(collection.site_path.filesystem_relative()).join(&candidate_dir_name);
                            fs::create_dir_all(&candidate_dir).unwrap();
                            candidate_dir.join(original_filename)
                        }
                        GetAnyResult::Playlist(playlist) => {
                            let mut candidate_dir_name = slugify(Path::new(&original_filename).with_extension("").to_string_lossy());
                            while context.site_dir.join(playlist.site_path.filesystem_relative()).join(&candidate_dir_name).exists() {
                                candidate_dir_name.push_str("-duplicate");
                            }
                            let candidate_dir = context.site_dir.join(playlist.site_path.filesystem_relative()).join(&candidate_dir_name);
                            fs::create_dir_all(&candidate_dir).unwrap();
                            candidate_dir.join(original_filename)
                        }
                        GetAnyResult::Video(video) => {
                            let mut candidate_filename = original_filename.clone();
                            while context.site_dir.join(video.site_path.filesystem_relative()).join(&candidate_filename).exists() {
                                candidate_filename = Path::new(&candidate_filename).with_file_name(format!("{}.mp4", nanoid())).to_string_lossy().to_string();
                            }
                            context.site_dir.join(video.site_path.filesystem_relative()).join(&candidate_filename)
                        }
                        GetAnyResult::None => unreachable!()
                    };

                    let storage_path_moving = storage_path.clone();
                    // File::create is blocking operation, use threadpool
                    let mut f = web::block(|| File::create(storage_path_moving)).await??;

                    // Field in turn is stream of *Bytes* object
                    while let Some(chunk) = field.try_next().await? {
                        // filesystem operations are blocking, we have to use threadpool
                        f = web::block(move || f.write_all(&chunk).map(|_| f)).await??;
                    }

                    let site = context.get_site();

                    let video = match &container {
                        GetAnyResult::Collection(collection) => {
                            let hyper_dir = HyperDir::read(storage_path.parent().unwrap());
                            let video = Video::read_dir(&context, &hyper_dir, collection.mutate_inherited_options(), &site);

                            let video_moving = Arc::clone(&video);
                            let mutation = |collection: &mut Collection| {
                                collection.videos.push(video_moving);
                            };

                            if let CollectionUpdate::NotFound = context.update_collection(mutation, &site_path) {
                                // TODO: Can only happen in weird time of check vs. time of use edge cases
                                return Ok(Either::Right(not_found(&context, &language, &site_path)));
                            }

                            video
                        }
                        GetAnyResult::Playlist(playlist) => {
                            let hyper_dir = HyperDir::read(storage_path.parent().unwrap());
                            let video = Video::read_dir(&context, &hyper_dir, playlist.mutate_inherited_options(), &site);

                            let video_moving = Arc::clone(&video);
                            let mutation = |playlist: &mut Playlist| {
                                playlist.videos.push(video_moving);
                            };

                            if let PlaylistUpdate::NotFound = context.update_playlist(mutation, &site_path) {
                                // TODO: Can only happen in weird time of check vs. time of use edge cases
                                return Ok(Either::Right(not_found(&context, &language, &site_path)));
                            }

                            video
                        }
                        GetAnyResult::Video(_) => {
                            let file_meta = FileMeta::new(&storage_path);
                            let file_name = storage_path.file_name().unwrap().to_string_lossy().to_string();

                            let mutation = |video: &mut Video| {
                                video.versions.push(VideoFile::new(file_meta, file_name, None));
                            };

                            match context.update_video(mutation, &site_path) {
                                Ok(video) => video,
                                // TODO: Can only happen in weird time of check vs. time of use edge cases
                                Err(()) => return Ok(Either::Right(not_found(&context, &language, &site_path)))
                            }
                        }
                        GetAnyResult::None => unreachable!()
                    };

                    let job_kind = JobKind::VideoFastTasks {
                        poster_aspect: site.poster_aspect.numeric(),
                        video: Arc::clone(&video)
                    };
                    context.push_job_queue(job_kind);

                    let redirect_url = format!("/video/{}", video.site_path.normalized());
                    return Ok(Either::Left(Redirect::to(redirect_url).see_other()))
                }
            }
        }
    }

    unimplemented!()
}
