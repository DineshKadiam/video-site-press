use std::sync::Arc;

use actix_web::HttpResponse;
use actix_web::web::{Data, Either, Form, Redirect};
use indoc::formatdoc;
use serde_derive::Deserialize;

use crate::{
    AspectRatio,
    BaseUrl,
    Context,
    Field,
    Language,
    Site,
    SiteContent,
    Theme,
    Translations
};
use crate::editor::endpoints;
use crate::editor::widgets::form_field;

#[derive(Deserialize)]
pub struct SiteForm {
    base_url: String,
    deployment_path: String,
    deployment_server: String,
    deployment_user: String,
    language: String,
    poster_aspect: String,
    theme: String
}

pub struct SiteFormFeedback {
    pub base_url: Field,
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

    // let active_formats = site.formats
    //     .iter()
    //     .map(|format| {
    //         let format_encoded = format.encode();
    //         formatdoc!(r#"
    //             <div>
    //                 <input checked id="{format_encoded}" name="{format_encoded}" type="checkbox">
    //                 <label for="{format_encoded}">{format_encoded}</label>
    //             </div>
    //         "#)
    //     })
    //     .collect::<Vec<String>>()
    //     .join("\n");

    // let proposed_formats = [
    //     VideoFormat::AudioOnly,
    //     VideoFormat::Video {
    //         codec: Codec::H264,
    //         container: Container::Mp4,
    //         encoding_approach: EncodingApproach::Speed,
    //         quality: 0.0,
    //         width: 240
    //     },
    //     VideoFormat::Video {
    //         codec: Codec::H264,
    //         container: Container::Mp4,
    //         encoding_approach: EncodingApproach::Efficiency,
    //         quality: 0.5,
    //         width: 480
    //     },
    //     VideoFormat::Video {
    //         codec: Codec::H264,
    //         container: Container::Mp4,
    //         encoding_approach: EncodingApproach::Efficiency,
    //         quality: 0.75,
    //         width: 720
    //     },
    //     VideoFormat::Video {
    //         codec: Codec::H264,
    //         container: Container::Mp4,
    //         encoding_approach: EncodingApproach::Efficiency,
    //         quality: 1.0,
    //         width: 1080
    //     }
    // ]
    //     .into_iter()
    //     .filter(|proposed| !settings.formats.iter().any(|format| format == proposed))
    //     .map(|format| {
    //         let format_encoded = format.encode();
    //         formatdoc!(r#"
    //             <div>
    //                 <input id="{format_encoded}" name="{format_encoded}" type="checkbox">
    //                 <label for="{format_encoded}">{format_encoded}</label>
    //             </div>
    //         "#)
    //     })
    //     .collect::<Vec<String>>()
    //     .join("\n");

    // <h2>Video formats to render for new videos</h2>

    // <p>
    //     When a new video gets uploaded, hyper8 will automatically
    //     encode it in the formats specified below. These settings, and
    //     any changes you are making to them now, do not retroactively
    //     apply to videos that were already added in the past. To
    //     re-encode existing videos in these formats, either request the
    //     re-encoding for specific videos from their own page, or use
    //     the batch re-encoding button/feature/page (not yet implemented).
    // </p>

    // <form action="/formats" method="post">
    //     {{active_formats}}
    //     {{proposed_formats}}
    //     <br>
    //     <button>Update formats</button>
    // </form>

    // TODO: For better usability, include detailed hints (help texts, possibly toggeable, hidden by default)
    //       that give very clear indication about what base url, deployment path/server/user etc. are and
    //       what to pay attention to.
    let t_deployment_path_label = translations.deployment_path_label;
    let t_deployment_server_label = translations.deployment_server_label;
    let t_deployment_user_label = translations.deployment_user_label;
    let t_language_label = translations.language_label;
    let t_not_saved = translations.not_saved;
    let t_poster_aspect_label = translations.poster_aspect_label;
    let t_public_address_label = translations.public_address_label;
    let (
        theme_key,
        input_base_url,
        input_deployment_path,
        input_deployment_server,
        input_deployment_user,
        input_language,
        input_poster_aspect,
        feedback
    ) = if let Some(SiteFormFeedback {
        base_url,
        deployment_path,
        deployment_server,
        deployment_user,
        language,
        poster_aspect,
        theme
    }) = &form_feedback {
        (
            &theme.key,
            form_field(base_url, "base_url", t_public_address_label),
            form_field(deployment_path, "deployment_path", t_deployment_path_label),
            form_field(deployment_server, "deployment_server", t_deployment_server_label),
            form_field(deployment_user, "deployment_user", t_deployment_user_label),
            form_field(language, "language", t_language_label),
            form_field(poster_aspect, "poster_aspect", t_poster_aspect_label),
            format!(r#"<span class="feedback">{t_not_saved}</span>"#)
        )
    } else {
        let language = match site.language.default {
            true => "",
            false => &site.language.code
        };

        let poster_aspect = match &site.poster_aspect {
            AspectRatio::Custom { representation, .. } => representation,
            AspectRatio::Default => ""
        };

        (
            &site.theme.key,
            form_field(&Field::valid(site.base_url.as_ref().map(|url| url.representation.as_str()).unwrap_or("")), "base_url", t_public_address_label),
            form_field(&Field::valid(site.deploy_config.path.as_ref().map(|path| path.as_str()).unwrap_or("")), "deployment_path", t_deployment_path_label),
            form_field(&Field::valid(site.deploy_config.server.as_ref().map(|server| server.as_str()).unwrap_or("")), "deployment_server", t_deployment_server_label),
            form_field(&Field::valid(site.deploy_config.user.as_ref().map(|user| user.as_str()).unwrap_or("")), "deployment_user", t_deployment_user_label),
            form_field(&Field::valid(language), "language", t_language_label),
            form_field(&Field::valid(poster_aspect), "poster_aspect", t_poster_aspect_label),
            String::new()
        )
    };

    let radio_button = |checked_key: &str, key: &str, label: &str| -> String {
        let checked = if key == checked_key { "checked " } else { "" };
        formatdoc!(r#"
            <span>
                <input {checked}name="theme" type="radio" value="{key}"> <label for="{key}">{label}</label>
            </span>
        "#)
    };

    let radio_button_adaptive = radio_button(theme_key, "adaptive", translations.adaptive);
    let radio_button_dark = radio_button(theme_key, "dark", translations.dark);
    let radio_button_light = radio_button(theme_key, "light", translations.light);

    let t_site_settings = translations.site_settings;
    let t_update_site = translations.update_site;
    formatdoc!(r#"
        <div class="form_group">
            <h2>{t_site_settings}</h2>

            {errors}
            
            <form action="/site" method="post">
                {input_base_url}
                {input_deployment_path}
                {input_deployment_server}
                {input_deployment_user}
                {input_poster_aspect}
                {input_language}
                <div class="form_field">
                    <span>Theme</span>
                    <div>
                        {radio_button_adaptive}
                        {radio_button_dark}
                        {radio_button_light}
                    </div>
                </div>

                <div style="align-items: center; column-gap: .5rem; display: flex;">
                    <button>{t_update_site}</button> {feedback}
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
    if path == "" {
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
        Some(BaseUrl::parse(&base_url_trimmed))
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
                endpoints::collection::edit_page(collection, &context, Some(form_feedback), &language)
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
            site_mut.deploy_config.path = deployment_path;
            site_mut.deploy_config.server = deployment_server;
            site_mut.deploy_config.user = deployment_user;
            site_mut.language = language;
            site_mut.poster_aspect = poster_aspect.unwrap();
            site_mut.theme = theme;
            site_mut.write_manifest(&context.site_dir);
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