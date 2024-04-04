use std::sync::Arc;

use actix_web::{Either, HttpResponse};
use actix_web::http::header::ContentType;
use actix_web::web::{Data, Redirect};

use crate::Context;

pub async fn collection(context: Data<Arc<Context>>) -> Either<Redirect, HttpResponse> {
    match context.convert_to_collection() {
        Ok(()) => Either::Left(Redirect::to("/collection/").see_other()),
        Err(err) => {
            // TODO: Return error gracefully in context of originating page (collection/video/empty)
            Either::Right(HttpResponse::Ok().content_type(ContentType::html()).body(err))
        }
    }
}

pub async fn playlist(context: Data<Arc<Context>>) -> Either<Redirect, HttpResponse> {
    match context.convert_to_playlist() {
        Ok(()) => Either::Left(Redirect::to("/playlist/").see_other()),
        Err(err) => {
            // TODO: Return error gracefully in context of originating page (collection/video/empty)
            Either::Right(HttpResponse::Ok().content_type(ContentType::html()).body(err))
        }
    }
}

pub async fn video(context: Data<Arc<Context>>) -> Either<Redirect, HttpResponse> {
    match context.convert_to_video() {
        Ok(()) => Either::Left(Redirect::to("/video/").see_other()),
        Err(err) => {
            // TODO: Return error gracefully in context of originating page (collection/playlist/empty)
            Either::Right(HttpResponse::Ok().content_type(ContentType::html()).body(err))
        }
    }
}