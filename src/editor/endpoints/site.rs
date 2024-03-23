use std::sync::Arc;

use actix_web::HttpResponse;
use actix_web::web::{Data, Either, Form, Redirect};
use indoc::formatdoc;
use serde_derive::Deserialize;

use crate::{
    AspectRatio,
    Context,
    Field,
    Site,
    SiteContent,
    Theme
};
use crate::editor::endpoints;
use crate::editor::widgets::form_field;
use crate::util::validate_url;

#[derive(Deserialize)]
pub struct SiteForm {
    base_url: String,
    poster_aspect: String,
    theme: String
}

pub struct SiteFormFeedback {
    pub base_url: Field,
    pub poster_aspect: Field,
    pub theme: Theme
}

fn edit_page(
    form_feedback: Option<&SiteFormFeedback>,
    site: &Site
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
    //         encoding_approach: EncodingApproach::Quality,
    //         quality: 0.5,
    //         width: 480
    //     },
    //     VideoFormat::Video {
    //         codec: Codec::H264,
    //         container: Container::Mp4,
    //         encoding_approach: EncodingApproach::Quality,
    //         quality: 0.75,
    //         width: 720
    //     },
    //     VideoFormat::Video {
    //         codec: Codec::H264,
    //         container: Container::Mp4,
    //         encoding_approach: EncodingApproach::Quality,
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

    let (
        theme_key,
        input_base_url,
        input_poster_aspect,
        feedback
    ) = if let Some(SiteFormFeedback { base_url, poster_aspect, theme }) = &form_feedback {
        (
            &theme.key,
            form_field(base_url, "base_url", "Base URL", "Base URL"),
            form_field(poster_aspect, "poster_aspect", "Custom Poster Aspect Ratio", "Custom Poster Aspect Ratio"),
            r#"<span class="feedback">Not Saved</span>"#
        )
    } else {
        let poster_aspect = match &site.poster_aspect {
            AspectRatio::Custom { representation, .. } => representation,
            AspectRatio::Default => ""
        };

        (
            &site.theme.key,
            form_field(&Field::valid(site.base_url.as_deref().unwrap_or("")), "base_url", "Base URL", "Base URL"),
            form_field(&Field::valid(poster_aspect), "poster_aspect", "Custom Poster Aspect Ratio", "Custom Poster Aspect Ratio"),
            ""
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

    let radio_button_adaptive = radio_button(theme_key, "adaptive", "Adaptive");
    let radio_button_dark = radio_button(theme_key, "dark", "Dark");
    let radio_button_light = radio_button(theme_key, "light", "Light");

    formatdoc!(r#"
        <h2>Site settings</h2>

        {errors}
        
        <form action="/site" method="post">
            {input_base_url}
            {input_poster_aspect}
            <div class="form_field">
                <span>Theme</span>
                <div>
                    {radio_button_adaptive}
                    {radio_button_dark}
                    {radio_button_light}
                </div>
            </div>
            
            <div style="align-items: center; column-gap: .5rem; display: flex;">
                <button>Update</button> {feedback}
            </div>
        </form>
    "#)
}

pub fn edit(context: &Context, form_feedback: Option<&SiteFormFeedback>, path: &str) -> String {
    if path == "" {
        let site = context.get_site();
        edit_page(form_feedback, &site)
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
    let base_url_trimmed = form.base_url.trim();
    let poster_aspect_trimmed = form.poster_aspect.trim();
    let theme = match Theme::from_key(&form.theme) {
        Ok(theme) => theme,
        Err(_) => Theme::default()
    };

    let base_url_error = if !base_url_trimmed.is_empty() {
        if let Err(message) = validate_url(base_url_trimmed) {
            Some(format!("Not a valid URL: {message}"))
        } else {
            None
        }
    } else {
        None
    };

    let poster_aspect = if poster_aspect_trimmed.is_empty() {
        Ok(AspectRatio::Default)
    } else {
        AspectRatio::from_representation(poster_aspect_trimmed)
            .map_err(|err| err.to_string())
    };

    if base_url_error.is_some() || poster_aspect.is_err() {
        let site = context.get_site();

        let site_form_feedback = SiteFormFeedback {
            base_url: Field::validated(base_url_trimmed, base_url_error),
            poster_aspect: Field::validated(poster_aspect_trimmed, poster_aspect.err()),
            theme
        };

        let http_response = match &site.content {
            SiteContent::Collection(collection) => {
                let form_feedback = endpoints::collection::CollectionFormFeedback::Site(site_form_feedback);
                endpoints::collection::edit_page(collection, &context, Some(form_feedback))
            }
            SiteContent::Playlist(playlist) => {
                let form_feedback = endpoints::playlist::PlaylistFormFeedback::Site(site_form_feedback);
                endpoints::playlist::edit_page(&context, Some(form_feedback), playlist)
            }
            SiteContent::Video(video) => {
                let form_feedback = endpoints::video::VideoFormFeedback::Site(site_form_feedback);
                endpoints::video::edit_page(&context, Some(form_feedback), video)
            }
            // TODO: At best a time of check/time of use edge case possibility?
            _ => unreachable!()
        };

        Either::Right(http_response)
    } else {
        let mutation = |site_mut: &mut Site| {
            site_mut.base_url = if base_url_trimmed.is_empty() { None } else { Some(base_url_trimmed.to_owned()) };
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