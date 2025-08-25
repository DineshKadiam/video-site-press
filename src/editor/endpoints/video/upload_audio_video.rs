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
    AudioFile,
    Collection,
    CollectionUpdate,
    Context,
    FileMeta,
    GetAnyResult,
    HyperDir,
    JobDesc,
    Playlist,
    PlaylistUpdate,
    SitePath,
    Video,
    VideoFile,
    VideoUpdate
};
use crate::editor::endpoints::collection::pending_manifest_errors as pending_collection_manifest_errors;
use crate::editor::endpoints::playlist::pending_manifest_errors as pending_playlist_manifest_errors;
use crate::editor::routes;
use crate::editor::widgets::{error_page, not_found};
use crate::infallible_mutation;
use crate::util::{
    deduplicate_filename,
    deduplicate_filestem,
    supported_audio_extension,
    supported_video_extension
};

use super::pending_manifest_errors as pending_video_manifest_errors;

pub async fn upload_audio_video(
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

                let is_supported_audio = supported_audio_extension(filename);
                let is_supported_video = supported_video_extension(filename);

                if is_supported_audio || is_supported_video {
                    let original_filename = filename.to_string();

                    let storage_path = match &container {
                        GetAnyResult::Collection(collection) => {
                            let file_stem = Path::new(&original_filename)
                                .file_stem()
                                .unwrap()
                                .to_string_lossy();

                            let mut candidate_dir_name = slugify(file_stem);
                            let mut candidate_path = context.site_dir
                                .join(collection.site_path.filesystem_relative())
                                .join(&candidate_dir_name);

                            while candidate_path.exists() {
                                candidate_dir_name = deduplicate_filestem(&candidate_dir_name);
                                candidate_path.set_file_name(&candidate_dir_name);
                            }

                            fs::create_dir_all(&candidate_path).unwrap();
                            candidate_path.join(original_filename)
                        }
                        GetAnyResult::Playlist(playlist) => {
                            let file_stem = Path::new(&original_filename)
                                .file_stem()
                                .unwrap()
                                .to_string_lossy();

                            let mut candidate_dir_name = slugify(file_stem);
                            let mut candidate_path = context.site_dir
                                .join(playlist.site_path.filesystem_relative())
                                .join(&candidate_dir_name);

                            while candidate_path.exists() {
                                candidate_dir_name = deduplicate_filestem(&candidate_dir_name);
                                candidate_path.set_file_name(&candidate_dir_name);
                            }

                            fs::create_dir_all(&candidate_path).unwrap();
                            candidate_path.join(original_filename)
                        }
                        GetAnyResult::Video(video) => {
                            let mut candidate_filename = original_filename.clone();
                            let mut candidate_path = context.site_dir
                                .join(video.site_path.filesystem_relative())
                                .join(&candidate_filename);

                            while candidate_path.exists() {
                                candidate_filename = deduplicate_filename(&candidate_filename);
                                candidate_path.set_file_name(&candidate_filename);
                            }

                            candidate_path
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

                            let mutation = |collection: &mut Collection| {
                                collection.videos.push(Arc::clone(&video));
                                video
                            };

                            match context.update_collection(mutation, &site_path) {
                                CollectionUpdate::InvalidManifest { collection } => {
                                    return Ok(Either::Right(pending_collection_manifest_errors(&collection, &context, &language)))
                                }
                                CollectionUpdate::NotFound => {
                                    // TODO: Can only happen in weird time of check vs. time of use edge cases
                                    return Ok(Either::Right(not_found(&context, &language, &site_path)));
                                }
                                CollectionUpdate::Updated { optional_return_value: video, .. } => video
                            }
                        }
                        GetAnyResult::Playlist(playlist) => {
                            let hyper_dir = HyperDir::read(storage_path.parent().unwrap());
                            let video = Video::read_dir(&context, &hyper_dir, playlist.mutate_inherited_options(), &site);

                            let mutation = |playlist: &mut Playlist| {
                                playlist.videos.push(Arc::clone(&video));
                                video
                            };

                            match context.update_playlist(mutation, &site_path) {
                                PlaylistUpdate::InvalidManifest { playlist } => {
                                    return Ok(Either::Right(pending_playlist_manifest_errors(&context, &language, &playlist)))
                                }
                                PlaylistUpdate::NotFound => {
                                    // TODO: Can only happen in weird time of check vs. time of use edge cases
                                    return Ok(Either::Right(not_found(&context, &language, &site_path)));
                                }
                                PlaylistUpdate::Updated { optional_return_value: video, .. } => video
                            }
                        }
                        GetAnyResult::Video(_) => {
                            let file_meta = FileMeta::new(&storage_path);
                            let file_name = storage_path.file_name().unwrap().to_string_lossy().to_string();

                            let mutation = |video: &mut Video| {
                                if is_supported_audio {
                                    video.audio_files.push(AudioFile::new(None, file_meta, file_name));
                                    Ok(())
                                } else /* if is_supported_video */ {
                                    video.video_files.push(VideoFile::new(file_meta, file_name, None));
                                    Ok(())
                                }
                            };

                            match context.update_video(mutation, &site_path) {
                                VideoUpdate::Error(_) => infallible_mutation!(),
                                VideoUpdate::InvalidManifest { video } => {
                                    return Ok(Either::Right(pending_video_manifest_errors(&context, &language, &video)))
                                }
                                // TODO: Can only happen in weird time of check vs. time of use edge cases
                                VideoUpdate::NotFound => {
                                    return Ok(Either::Right(not_found(&context, &language, &site_path)));
                                }
                                VideoUpdate::Success { video_updated, .. } => video_updated
                            }
                        }
                        GetAnyResult::None => unreachable!()
                    };

                    let job_desc = if is_supported_audio {
                        JobDesc::AudioMeta {
                            video: Arc::clone(&video)
                        }
                    } else /* if is_supported_video */ {
                        JobDesc::VideoFastTasks {
                            poster_aspect: site.poster_aspect.numeric(),
                            video: Arc::clone(&video)
                        }
                    };

                    context.queue_job(job_desc);

                    let redirect_url = routes::video_edit(&video);
                    return Ok(Either::Left(Redirect::to(redirect_url).see_other()))
                } else {
                    let message = language.translations.unsupported_file_extension_in_xxx(filename);
                    return Ok(Either::Right(error_page(&context, &language, &message)));
                }
            }
        }
    }

    unimplemented!()
}
