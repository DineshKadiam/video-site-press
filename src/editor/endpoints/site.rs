// SPDX-FileCopyrightText: 2024 Simon Repp
// SPDX-License-Identifier: AGPL-3.0-or-later

use std::sync::Arc;

use actix_web::HttpResponse;
use actix_web::web::{Data, Either, Form, Redirect};
use indoc::formatdoc;
use serde_derive::Deserialize;

use crate::{
    AspectRatio,
    HyperUrl,
    Context,
    Field,
    Language,
    Site,
    SiteContent,
    Theme,
    Translations
};
use crate::editor::{Checkbox, Input, Select};
use crate::editor::endpoints;

#[derive(Deserialize)]
pub struct SiteForm {
    base_url: String,
    clean_urls: Option<String>,
    deployment_path: String,
    deployment_server: String,
    deployment_user: String,
    language: String,
    poster_aspect: String,
    theme: String
}

pub struct SiteFormFeedback {
    pub base_url: Field,
    pub clean_urls: bool,
    pub deployment_path: Field,
    pub deployment_server: Field,
    pub deployment_user: Field,
    pub language: Field,
    pub poster_aspect: Field,
    pub theme: Theme
}

fn edit_page(
    form_feedback: Option<&SiteFormFeedback>,
    site: &Site,
    translations: &Translations
) -> String {
    let errors = if site.errors.is_empty() {
        String::new()
    } else {
        site.errors
            .iter()
            .map(|error| {
                formatdoc!(r#"
                    <pre class="error">{error}</pre>
                "#)
            })
            .collect::<Vec<String>>()
            .join("")
    };

    let mut clean_urls_checkbox = Checkbox::new("clean_urls", translations.clean_urls_label);
    let mut input_base_url = Input::url("base_url", translations.public_address_label);
    let mut input_deployment_path = Input::text("deployment_path", translations.deployment_path_label);
    let mut input_deployment_server = Input::text("deployment_server", translations.deployment_server_label);
    let mut input_deployment_user = Input::text("deployment_user", translations.deployment_user_label);
    let mut input_language = Input::text("language", translations.language_label);
    let mut input_poster_aspect = Input::text("poster_aspect", translations.poster_aspect_label);

    let mut select_theme = Select::new("theme", translations.theme);
    for (key, label) in Theme::variants(translations) {
        select_theme.option(key, label);
    }

    // TODO: For better usability, include detailed hints (help texts, possibly toggeable, hidden by default)
    //       that give very clear indication about what base url, deployment path/server/user etc. are and
    //       what to pay attention to.
    let t_not_saved = translations.not_saved;
    let feedback = if let Some(unsaved) = &form_feedback {
        clean_urls_checkbox.checked(unsaved.clean_urls);
        input_base_url.validated_value(&unsaved.base_url);
        input_deployment_path.validated_value(&unsaved.deployment_path);
        input_deployment_server.validated_value(&unsaved.deployment_server);
        input_deployment_user.validated_value(&unsaved.deployment_user);
        input_language.validated_value(&unsaved.language);
        input_poster_aspect.validated_value(&unsaved.poster_aspect);
        select_theme.selected(unsaved.theme.key);

        format!(r#"<span class="feedback">{t_not_saved}</span>"#)
    } else {
        let language = if site.language.default { None } else { Some(&site.language.code) };
        let poster_aspect = match &site.poster_aspect {
            AspectRatio::Custom { representation, .. } => Some(representation),
            AspectRatio::Default => None
        };

        clean_urls_checkbox.checked(site.clean_urls);
        input_base_url.value_option(&site.base_url.as_ref().map(|url| url.representation.clone()));
        input_deployment_path.value_option(&site.deploy_config.path);
        input_deployment_server.value_option(&site.deploy_config.server);
        input_deployment_user.value_option(&site.deploy_config.user);
        input_language.value_option(&language);
        input_poster_aspect.value_option(&poster_aspect);
        select_theme.selected(site.theme.key);

        String::new()
    };

    let t_deployment_is_currently_supported_text = translations.deployment_is_currently_supported_text;
    let t_site_settings = translations.site_settings;
    let t_update_site = translations.update_site;
    formatdoc!(r#"
        <div class="form_group">
            <a id="edit-site"></a>
            <h2>{t_site_settings}</h2>

            {errors}

            <p>{t_deployment_is_currently_supported_text}</p>
            
            <form action="/site" autocomplete="off" data-warn-discard method="post">
                {input_base_url}
                {input_deployment_path}
                {input_deployment_server}
                {input_deployment_user}
                {input_poster_aspect}
                {input_language}
                {select_theme}
                {clean_urls_checkbox}

                <div style="align-items: center; column-gap: .5rem; display: flex;">
                    <button class="action">{t_update_site}</button> {feedback}
                </div>
            </form>
        </div>
    "#)
}

pub fn edit(
    context: &Context,
    form_feedback: Option<&SiteFormFeedback>,
    path: &str,
    translations: &Translations
) -> String {
    if path.is_empty() {
        let site = context.get_site();
        edit_page(form_feedback, &site, translations)
    } else {
        String::new()
    }
}

// pub async fn formats(context: Data<Arc<Context>>, payload: Bytes) -> Redirect {
//     let mut formats = parse(&payload)
//         .into_iter()
//         .filter_map(|(format, _checked_value)|
//             VideoFormat::parse(&format)
//         )
//         .collect();

//     let site = {
//         let mut site_mut = context.site.lock().unwrap();

//         site_mut.formats.clear();
//         site_mut.formats.append(&mut formats);

//         site_mut.clone()
//     };

//     site.persist_in_site(&context.site_dir);

//     Redirect::to("/site").see_other()
// }

pub async fn update(
    context: Data<Arc<Context>>,
    form: Form<SiteForm>
) -> Either<Redirect, HttpResponse> {
    let language = context.get_editor_language();

    let base_url_trimmed = form.base_url.trim();
    let clean_urls = form.clean_urls.is_some();
    let deployment_path_trimmed = form.deployment_path.trim();
    let deployment_server_trimmed = form.deployment_server.trim();
    let deployment_user_trimmed = form.deployment_user.trim();
    let language_trimmed = form.language.trim();
    let poster_aspect_trimmed = form.poster_aspect.trim();
    let theme = match Theme::from_key(&form.theme) {
        Ok(theme) => theme,
        Err(_) => Theme::default()
    };

    let base_url = if base_url_trimmed.is_empty() {
        None
    } else {
        Some(HyperUrl::parse(base_url_trimmed))
    };

    let poster_aspect = if poster_aspect_trimmed.is_empty() {
        Ok(AspectRatio::Default)
    } else {
        AspectRatio::from_representation(poster_aspect_trimmed)
            .map_err(|err| err.to_string())
    };

    if base_url.as_ref().is_some_and(|base_url| base_url.is_err()) || poster_aspect.is_err() {
        let site = context.get_site();

        let site_form_feedback = SiteFormFeedback {
            base_url: Field::validated(base_url_trimmed, base_url.and_then(|base_url| base_url.err().map(|err| err.to_string()))),
            clean_urls,
            deployment_path: Field::valid(deployment_path_trimmed),
            deployment_server: Field::valid(deployment_server_trimmed),
            deployment_user: Field::valid(deployment_user_trimmed),
            language: Field::valid(language_trimmed),
            poster_aspect: Field::validated(poster_aspect_trimmed, poster_aspect.err()),
            theme
        };

        let http_response = match &site.content {
            SiteContent::Collection(collection) => {
                let form_feedback = endpoints::collection::CollectionFormFeedback::Site(site_form_feedback);
                endpoints::collection::edit_page(collection, &context, form_feedback, &language)
            }
            SiteContent::Playlist(playlist) => {
                let form_feedback = endpoints::playlist::PlaylistFormFeedback::Site(site_form_feedback);
                endpoints::playlist::edit_page(&context, Some(form_feedback), &language, playlist)
            }
            SiteContent::Video(video) => {
                let form_feedback = endpoints::video::VideoFormFeedback::Site(site_form_feedback);
                endpoints::video::edit_page(&context, Some(form_feedback), &language, video)
            }
            // TODO: At best a time of check/time of use edge case possibility?
            _ => unreachable!()
        };

        Either::Right(http_response)
    } else {
        let deployment_path = if deployment_path_trimmed.is_empty() {
            None
        } else {
            Some(deployment_path_trimmed.to_string())
        };

        let deployment_server = if deployment_server_trimmed.is_empty() {
            None
        } else {
            Some(deployment_server_trimmed.to_string())
        };

        let deployment_user = if deployment_user_trimmed.is_empty() {
            None
        } else {
            Some(deployment_user_trimmed.to_string())
        };

        let language = if language_trimmed.is_empty() {
            Language::default()
        } else {
            Language::from_code(language_trimmed)
        };

        let mutation = |site_mut: &mut Site| {
            site_mut.base_url = base_url.map(|base_url| base_url.unwrap());
            site_mut.clean_urls = clean_urls;
            site_mut.deploy_config.path = deployment_path;
            site_mut.deploy_config.server = deployment_server;
            site_mut.deploy_config.user = deployment_user;
            site_mut.language = language;
            site_mut.poster_aspect = poster_aspect.unwrap();
            site_mut.theme = theme;
            site_mut.write_manifest(&context);
        };

        let site = context.update_site(mutation);

        let redirect_url = match site.content {
            SiteContent::Collection(_) => "/collection/",
            SiteContent::Playlist(_) => "/playlist/",
            SiteContent::Video(_) => "/video/",
            _ => todo!()
        };

        // Banner/Poster assets need recomputation when site.poster_aspect
        // changes (which could have happened here).
        context.compute_job_queue();

        Either::Left(Redirect::to(redirect_url).see_other())
    }
}