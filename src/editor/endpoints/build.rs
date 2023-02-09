use std::sync::Arc;

use actix_web::web::{Data, Redirect};

use crate::Context;
use crate::build::perform_build;

pub async fn build(context: Data<Arc<Context>>) -> Redirect {
    perform_build(&context);

    Redirect::to("/preview/").see_other()
}