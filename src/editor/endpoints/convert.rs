use std::sync::Arc;

use actix_web::{Either, HttpResponse};
use actix_web::web::{Data, Path as ActixPath, Redirect};

use crate::{Context, ContextPath};
use crate::editor::widgets::error_page;

pub async fn to_collection(
    context: Data<Arc<Context>>,
    path: ActixPath<ContextPath>
) -> Redirect {
    context.convert_to_collection(&path.path);
    let redirect_url = format!("/collection/{}", path.path);
    Redirect::to(redirect_url).see_other()
}

pub async fn to_playlist(
    context: Data<Arc<Context>>,
    path: ActixPath<ContextPath>
) -> Either<Redirect, HttpResponse> {
    match context.convert_to_playlist(&path.path) {
        Ok(()) => {
            let redirect_url = format!("/playlist/{}", path.path);
            Either::Left(Redirect::to(redirect_url).see_other())
        }
        Err(err) => {
            let language = context.get_editor_language();
            // TODO: Return error in context of originating page (collection/video/empty)
            Either::Right(error_page(&context, &language, &err))
        }
    }
}

pub async fn to_video(
    context: Data<Arc<Context>>,
    path: ActixPath<ContextPath>
) -> Either<Redirect, HttpResponse> {
    match context.convert_to_video(&path.path) {
        Ok(()) => {
            let redirect_url = format!("/video/{}", path.path);
            Either::Left(Redirect::to(redirect_url).see_other())
        }
        Err(err) => {
            let language = context.get_editor_language();
            // TODO: Return error in context of originating page (collection/playlist/empty)
            Either::Right(error_page(&context, &language, &err))
        }
    }
}