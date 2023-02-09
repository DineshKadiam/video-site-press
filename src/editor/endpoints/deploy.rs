use std::sync::Arc;

use actix_web::web::{Data, Redirect};

use crate::Context;
use crate::deploy::perform_deploy;

pub async fn deploy(context: Data<Arc<Context>>) -> Redirect {
    perform_deploy(&context);

    Redirect::to("/").see_other()
}