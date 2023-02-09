use std::sync::Arc;

use actix_web::web::{Data, Redirect};

use crate::Context;

pub async fn collection(context: Data<Arc<Context>>) -> Redirect {
    context.convert_to_collection();

    Redirect::to("/").see_other()
}

pub async fn playlist(context: Data<Arc<Context>>) -> Redirect {
    context.convert_to_playlist();

    Redirect::to("/").see_other()
}

pub async fn video(context: Data<Arc<Context>>) -> Redirect {
    context.convert_to_video();

    Redirect::to("/").see_other()
}