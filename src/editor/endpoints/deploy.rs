use std::sync::Arc;

use actix_web::web::{Data, Redirect};

use crate::{Context, SiteContent};
use crate::build::perform_build;
use crate::deploy::{config_probe, perform_deploy};

pub async fn configure(context: Data<Arc<Context>>) -> Redirect {
    let site = context.get_site();

    let redirect_url = match site.content {
        SiteContent::Collection(_) => "/collection/#edit-site",
        SiteContent::Playlist(_) => "/playlist/#edit-site",
        SiteContent::Video(_) => "/video/#edit-site",
        // TODO: Is there any point to implementing a site form for an uninitialized site?
        //       (at this point offering a link to configure deployment, when there is no
        //       way to configure anything, is anyhow definitely wrong).
        _ => "/"
    };

    Redirect::to(redirect_url).see_other()
}

pub async fn deploy(context: Data<Arc<Context>>) -> Redirect {
    let site = context.get_site();

    // TODO: All of these errors (currently: panics) should be gracefully communicated through the browser interface

    if !site.base_url.is_some() {
        panic!("Base url is not set, therefore the deploy config cannot be probed, therefore deploying is not permitted for safety reasons.");
    }

    if !site.deploy_config.complete() {
        panic!("Deploy config is not complete, can not deploy.");
    }

    if let Err(err) = config_probe(&context, &site) {
        panic!("The deployment configuration could not be verified: {err}");
    }

    perform_build(&context);
    perform_deploy(&context, &site);

    Redirect::to("/").see_other()
}