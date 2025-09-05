// SPDX-FileCopyrightText: 2025 Simon Repp
// SPDX-License-Identifier: AGPL-3.0-or-later

use std::sync::Arc;

use actix_session::Session as ActixSession;
use actix_web::HttpResponse;
use actix_web::web::{Data, Path as ActixPath};
use indoc::formatdoc;

use hyper8_core::{Collection, SitePath};

use crate::{
    Context,
    Location,
    Session,
    Textarea
};
use crate::actix::html_response;
use crate::routes;
use crate::widgets::{
    layout,
    not_found,
    upload_widget
};

pub async fn banner(
    context: Data<Arc<Context>>,
    session: ActixSession,
    site_path: ActixPath<SitePath>
) -> HttpResponse {
    let session = Session::new(&session);

    match context.get_collection(&site_path) {
        Some(collection) => edit_banner_page(&collection, &context, &session),
        None => not_found(&context, &session, &site_path)
    }
}

pub fn edit_banner_page(
    collection: &Collection,
    context: &Context,
    session: &Session
) -> HttpResponse {
    let site = context.get_site();
    let translations = &session.language.translations;

    let t_upload_banner = translations.upload_banner;
    let banner_upload_button = upload_widget(&routes::collection_upload_banner(collection), t_upload_banner);
    let banner_form = match &collection.banner {
        Some(banner) => {
            let alt = if let Some(description) = &banner.description {
                format!(r#"alt="{description}""#)
            } else {
                String::new()
            };

            // TODO: Could/should also use computed assets (also elsewhere, videos etc.)
            let file_name = &banner.file_name;
            let src = routes::poster(file_name, &collection.site_path);

            let mut description = Textarea::image_description(
                "description",
                translations.image_description_for_blind_or_visually_impaired_people
            );

            description
                .id("banner_description")
                .value_option(&banner.description);

            let t_delete_banner = translations.delete_banner;
            let t_really_delete = translations.really_delete;
            let t_update_description = translations.update_description;

            let delete_banner_action = routes::collection_delete_banner(collection);
            let update_banner_action = routes::collection_update_banner(collection);

            format!(r#"
                <img {alt} class="thumbnail" src="{src}">
                <div class="button_group">
                    {banner_upload_button}
                    <form action="{delete_banner_action}" data-confirm data-confirm-label="{t_delete_banner}" data-confirm-text="{t_really_delete}" method="post">
                        <button class="danger">{t_delete_banner}</button>
                    </form>
                </div>
                <form action="{update_banner_action}" data-warn-discard method="post">
                    {description}
                    <button class="action">{t_update_description}</button>
                </form>
            "#)
        }
        None => banner_upload_button
    };

    let t_banner_image = translations.banner_image;
    let body = formatdoc!(r#"
        <div class="form_group">
            <h2>{t_banner_image}</h2>
            <p style="background: var(--warning-bg); border-radius: .5rem; color: var(--warning-fg); padding: 1rem;">
                Banner images are not yet used, but this will be included in a future version of Hyper 8
            </p>
            {banner_form}
        </div>
    "#);

    let collection_title = collection.title_or_slug_or_generic_label(translations);
    let head_title = format!("{t_banner_image} – {collection_title}");
    let html = layout(
        &body,
        context,
        Location::poster(&collection.site_path),
        session,
        &site,
        &head_title
    );

    html_response(html)
}
