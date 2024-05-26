use std::fs;

use indoc::formatdoc;

use crate::{
    Collection,
    Context,
    MarkdownSubset,
    Playlist,
    Site
};
use crate::build::outline::Traversal;
use crate::util::localized_date_d_mmmm_yyyy;
use crate::build::widgets::{
    banner_html,
    layout,
    links,
    thumbnail,
    unlisted_badge
};

pub fn collection_html(
    context: &Context,
    collection: &Collection,
    site: &Site,
    traversal: &Traversal
) {
    let mut body = String::new();

    let title = if collection.path == "" {
        collection.title.as_deref().unwrap_or("Hyper 8")
    } else {
        collection.title.as_deref().unwrap_or_else(|| collection.slug())
    };

    // TODO: Hack to get margin to come out beneath header - replace with something proper soon
    body.push_str(r#"<div style="margin-top: 4rem;"></div>"#);

    if let Some(banner) = &collection.banner {
        let collection_prefix = "";
        let img = banner_html(banner, collection_prefix);
        body.push_str(&img);
    }

    let unlisted = unlisted_badge(collection.unlisted, &site.language.translations);

    body.push_str(&format!("<h1>{title} {unlisted}</h1>\n\n"));

    if let Some(description) = &collection.description {
        let description_html = MarkdownSubset::to_html(description);
        body.push_str(&format!(r#"<p class="description">{description_html}</p>"#));
    }

    if !collection.links.is_empty() {
        let links_rendered = links(&site.language, &collection.links);
        body.push_str(&links_rendered);
    }

    let videos = collection.public_videos_desc_by_release_date();

    if !videos.is_empty() {
        let videos = videos
            .iter()
            .map(|video| {
                let traversal_with_video = traversal.peek_video(video);
                let traversal_with_video_prefix = traversal_with_video.file_prefix_desc();

                let release_date = match video.release_date {
                    Some(release_date) => {
                        let formatted_date = localized_date_d_mmmm_yyyy(release_date, site.language.chrono_locale);
                        format!(r#"<span class="release_date">{formatted_date}</span>"#)
                    }
                    None => String::new()
                };
                let title = video.title.as_deref().unwrap_or_else(|| video.slug());

                let video_thumbnail = thumbnail(&site.language.translations, video, &traversal_with_video_prefix, true);

                let video_href = traversal_with_video.index_desc(site);
                formatdoc!(r#"
                    <a href="{video_href}">
                        {video_thumbnail}
                        <div class="caption">{title}</div>
                        {release_date}
                    </a>
                "#)
            })
            .collect::<Vec<String>>()
            .join("\n");

        let videos_rendered = formatdoc!(r#"
            <div class="videos_grid">
                {videos}
            </div>
        "#);

        body.push_str(&videos_rendered);
    }

    let playlists = collection.public_playlists_desc_by_release_date();
    for playlist in &playlists {
        let rendered = render_playlist(playlist, site, &traversal);
        body.push_str(&rendered);
    }

    let subcollections = collection.public_subcollections_desc_by_release_date();
    for subcollection in &subcollections {
        let rendered = render_collection(subcollection, site, &traversal);
        body.push_str(&rendered);
    }

    // if traversal.steps.len() > 1 {
    //     let outline_items = traversal.steps
    //         .iter()
    //         .filter(|step| !step.is_collection(collection))
    //         .enumerate()
    //         .map(|(index, step)| {
    //             let prefix = "../".repeat(traversal.steps.len() - 1 - index);
    //             step.context_for_collection(collection, &prefix, site)
    //         })
    //         .collect::<Vec<String>>()
    //         .join("\n");

    //     body.push_str(&format!(r#"<div class="outline">{outline_items}</div>"#));
    // }

    let html = layout(&body, site, title, traversal);

    let collection_dir = context.build_dir.join(&collection.path);
    fs::create_dir_all(&collection_dir).unwrap();
    let index_path = collection_dir.join("index.html");
    fs::write(index_path, html).unwrap();
}

fn render_collection(
    collection: &Collection,
    site: &Site,
    traversal: &Traversal
) -> String {
    let mut output = String::new();

    let traversal_with_collection = traversal.peek_collection(collection);
    let traversal_with_collection_prefix = traversal_with_collection.file_prefix_desc();

    let collection_href = traversal_with_collection.index_desc(site);

    if let Some(banner) = &collection.banner {
        let banner = banner_html(banner, &traversal_with_collection_prefix);

        let wrapped = format!(r#"<a href="{collection_href}">{banner}</a>"#);

        output.push_str(&wrapped);
    }

    let title = &collection.title.as_deref().unwrap_or_else(|| collection.slug());

    let videos = collection.public_videos_desc_by_release_date();

     if !videos.is_empty() {
        let videos = videos
            .iter()
            .map(|video| {
                let traversal_with_video = traversal_with_collection.peek_video(video);
                let traversal_with_video_prefix = traversal_with_video.file_prefix_desc();

                let release_date = match video.release_date {
                    Some(release_date) => {
                        let formatted_date = localized_date_d_mmmm_yyyy(release_date, site.language.chrono_locale);
                        format!(r#"<span class="release_date">{formatted_date}</span>"#)
                    }
                    None => String::new()
                };
                let title = video.title.as_deref().unwrap_or_else(|| video.slug());
                let video_thumbnail = thumbnail(&site.language.translations, video, &traversal_with_video_prefix, true);

                let video_href = traversal_with_video.index_desc(site);
                formatdoc!(r#"
                    <a href="{video_href}">
                        {video_thumbnail}
                        <div class="caption">{title}</div>
                        {release_date}
                    </a>
                "#)
            })
            .collect::<Vec<String>>()
            .join("\n");

        let videos_rendered = formatdoc!(r#"
            <div class="videos_grid">
                {videos}
            </div>
        "#);

        output.push_str(&videos_rendered);
    }

    let playlists = collection.public_playlists_desc_by_release_date();
    for playlist in &playlists {
        let rendered = render_playlist(playlist, site, &traversal_with_collection);
        output.push_str(&rendered);
    }

    let subcollections = collection.public_subcollections_desc_by_release_date();
    for subcollection in &subcollections {
        let rendered = render_collection(subcollection, site, &traversal_with_collection);
        output.push_str(&rendered);
    }

    formatdoc!(r#"
        <h2>
            <a href="{collection_href}">
                {title}
            </a>
        </h2>

        {output}
    "#)
}

fn render_playlist(
    playlist: &Playlist,
    site: &Site,
    traversal: &Traversal
) -> String {
    let title = playlist.title.as_deref().unwrap_or_else(|| playlist.slug());

    let traversal_with_playlist = traversal.peek_playlist(playlist);
    let traversal_with_playlist_prefix = traversal_with_playlist.file_prefix_desc();

    let banner = if let Some(banner) = &playlist.banner {
        banner_html(banner, &traversal_with_playlist_prefix)
    } else {
        String::new()
    };

    let videos = playlist.public_videos_asc_by_release_date()
        .iter()
        .enumerate()
        .map(|(index, video)| {
            let traversal_with_video = traversal_with_playlist.peek_video(video);
            let traversal_with_video_prefix = traversal_with_video.file_prefix_desc();

            let release_date = match video.release_date {
                Some(release_date) => {
                    let formatted_date = localized_date_d_mmmm_yyyy(release_date, site.language.chrono_locale);
                    format!(r#"<span class="release_date">{formatted_date}</span>"#)
                }
                None => String::new()
            };
            let number = index + 1;
            let title = video.title.as_deref().unwrap_or_else(|| video.slug());

            let video_thumbnail = thumbnail(&site.language.translations, video, &traversal_with_video_prefix, true);

            let video_href = traversal_with_video.index_desc(site);
            formatdoc!(r#"
                <a href="{video_href}">
                    {video_thumbnail}
                    <div class="caption">{title}</div>
                    {release_date} <span class="number"># {number}</span>
                </a>
            "#)
        })
        .collect::<Vec<String>>()
        .join("\n");

    let videos_rendered = formatdoc!(r#"
        <div class="videos_grid">
            {videos}
        </div>
    "#);

    let playlist_href = traversal_with_playlist.index_desc(site);
    formatdoc!(r#"
        <h2><a href="{playlist_href}">{title}</a></h2>
        <a href="{playlist_href}">{banner}</a>
        {videos_rendered}
    "#)
}