use std::sync::Arc;

use actix_web::web::{Data, Json};

use crate::{Context, Processing};

pub async fn processing(context: Data<Arc<Context>>) -> Json<Processing> {
    Json(context.processing.lock().unwrap().clone())
}