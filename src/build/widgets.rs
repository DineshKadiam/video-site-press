use indoc::formatdoc;

use crate::{Site, Video};
use crate::util::casual_duration;

pub fn copy_link(site: &Site, path: &str) -> String {
    let data_url = match &site.base_url {
        Some(base_url) => {
            let url = base_url.join(path).unwrap().to_string();
            format!(r#"data-url="{url}""#)
        }
        None => String::new()
    };

    let t_copy_link = &site.language.translations.copy_link;

    format!(r#"
        <div>
            <button {data_url} id="copy-link">{t_copy_link}</button>
        </div>
    "#)
}


pub fn thumbnail(video: &Video, video_prefix: &str) -> String {
    let duration = match &video.highest_resolution_version().and_then(|version| version.video_meta.as_ref()) {
        Some(video_meta) => {
            let duration = casual_duration(video_meta.duration);
            formatdoc!(r#"<span class="duration">{duration}</span>"#)
        }
        None => String::new()
    };

    match &video.poster {
        Some(poster) => {
            let alt = if let Some(description) = &poster.description {
                format!(r#"alt="{description}""#)
            } else {
                String::new()
            };

            let img = match &poster.assets {
                Some(poster_assets) => {
                    let image_attributes = poster_assets.img_attributes_up_to_480("poster", video_prefix);
                    let src = image_attributes.src;
                    let srcset = image_attributes.srcset;

                    format!(r#"<img {alt} class="thumbnail" src="{src}" srcset="{srcset}">"#)
                }
                None => {
                    format!(r#"<img {alt} class="thumbnail" src="{video_prefix}poster.jpg">"#)
                }
            };

            formatdoc!(r#"
                <div class="thumbnail_wrapper">
                    {img}
                    {duration}
                </div>
            "#)
        },
        None => String::new()
    }
}
