use std::sync::Arc;

use actix_web::HttpResponse;
use actix_web::web::{Data, Either, Redirect};

use crate::Context;
use crate::build::perform_build;
use crate::editor::widgets::error_page;

pub async fn build(context: Data<Arc<Context>>) -> Either<Redirect, HttpResponse> {
    let pending_deployment = context.deploying.lock().unwrap().active();

    if pending_deployment {
        let language = context.get_editor_language();
        // TODO: Localize
        let message = "Currently deploying, can only build and preview when the deployment is finished.";
        return Either::Right(error_page(&context, &language, message));
    }

    perform_build(&context);

    Either::Left(Redirect::to("/preview/").see_other())
}