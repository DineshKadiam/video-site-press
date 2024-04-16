use std::fs;
use std::sync::Arc;

use indoc::formatdoc;

use crate::{Context, Playlist, Site, Video};
use crate::build::Breadcrumb;
use crate::build::render::layout;
use crate::build::widgets::{copy_link, downloads, player, thumbnail};
use crate::util::casual_duration;

pub fn video_html(
    context: &Context,
    playlist: Option<&Playlist>,
    video: &Video,
    site: &Site,
    breadcrumbs: &Vec<Breadcrumb>
) {
    let index_suffix = context.index_suffix();
    let description = if let Some(description) = &video.description { description } else { "" };
    let title = video.title.as_deref().unwrap_or(video.slug());
    let player_rendered = player(site, video, "");

    let downloads_widget = downloads(video);

    let playlist_context = if video.unlisted {
        String::new()
    } else {
        match playlist {
            Some(playlist) => {
                let listed_videos: Vec<Arc<Video>> = playlist.videos
                    .iter()
                    .filter(|video| !video.unlisted)
                    .map(|video| Arc::clone(video))
                    .collect();

                let active_position = listed_videos
                    .iter()
                    .position(|video_iterated| video_iterated.path == video.path)
                    .unwrap();

                let mut out = String::new();

                out.push_str(r#"<div class="playlist_context">"#);

                let max_shown: usize = 20;
                let max_window: usize = max_shown - 2;

                let indices_shown = if listed_videos.len() > max_window {
                    let max_previous_shown = (max_window as f32 / 2.0).floor() as usize;

                    if max_previous_shown > active_position {
                        0..max_window
                    } else {
                        let max_remaining_shown = (max_window as f32 / 2.0).ceil() as usize;

                        if active_position + max_remaining_shown >= listed_videos.len() {
                            (listed_videos.len() - max_window)..listed_videos.len()
                        } else {
                            let from_index = active_position - max_previous_shown;
                            from_index..from_index + max_window
                        }
                    }
                } else {
                    0..listed_videos.len()
                };

                if indices_shown.start > 0 {
                    let hidden_videos = indices_shown.start;
                    let more = formatdoc!(r#"
                        <a class="more" href="..{index_suffix}">
                            <div class="thumbnail"></div>
                            <div class="description">{hidden_videos} previous videos</div>
                        </a>
                    "#);

                    out.push_str(&more);
                }

                for index in indices_shown.clone() {
                    let video_iterated = &listed_videos[index];

                    let label = video_iterated.label(context);
                    let slug = video_iterated.slug();

                    let video_prefix = if index == active_position {
                        String::new()
                    } else {
                        format!("../{slug}/")
                    };

                    let thumbnail_rendered = thumbnail(video_iterated, &video_prefix, false);

                    let (current, href) = if index == active_position {
                        ("current", String::new())
                    } else {
                        ("", format!("../{slug}{index_suffix}"))
                    };

                    let duration = match &video_iterated.highest_resolution_version().and_then(|version| version.video_meta.as_ref()) {
                        Some(video_meta) => casual_duration(video_meta.duration),
                        None => String::new()
                    };

                    let number = index + 1;
                    let line = formatdoc!(r#"
                        <a class="{current}" href="{href}">
                            <div class="number">{number}</div>
                            {thumbnail_rendered}
                            <div class="description">
                                <span class="label">{label}</span>
                                <span class="details">
                                    <span class="duration">{duration}</span>
                                </span>
                            </div>
                        </a>
                    "#);

                    out.push_str(&line);
                }

                if indices_shown.end <= listed_videos.len() - 1 {
                    let hidden_videos = listed_videos.len() - indices_shown.end;
                    let more = formatdoc!(r#"
                        <a class="more" href="..{index_suffix}">
                            <div class="thumbnail"></div>
                            <div class="description">{hidden_videos} more videos</div>
                        </a>
                    "#);

                    out.push_str(&more);
                }

                let label = playlist.label(context);
                let t_videos = &site.language.translations.videos;
                let video_count = listed_videos.len();
                let header = formatdoc!(r#"
                    <div class="playlist_header">
                        {label}<br>
                        {video_count} {t_videos}
                    </div>
                "#);

                // TODO: Reactivate
                // out.push_str(&header);

                out.push_str("</div>");

                out
            }
            None => String::new()
        }
    };

    let copy_link_widget = copy_link(site, &video.path);

    let release_date = match video.release_date {
        Some(release_date) => {
            use chrono::format::{Fixed, Item, Numeric, Pad};

            let format = [
                Item::Numeric(Numeric::Day, Pad::None),
                Item::Space(" "),
                Item::Fixed(Fixed::LongMonthName),
                Item::Space(" "),
                Item::Numeric(Numeric::Year, Pad::None)
            ];
            let formatted = release_date.format_localized_with_items(format.iter(), site.language.chrono_locale);
            format!(r#"<span class="release_date">{formatted}</span>"#)
        }
        None => String::new()
    };

    let body = format!(r#"
        {player_rendered}
        <div class="video_info">
            <div style="display: flex; align-items: center; justify-content: space-between;">
                <h1 class="video_title">{title}</h1>
                {copy_link_widget}
            </div>
            {release_date}
            <p>
                {description}
            </p>
            {downloads_widget}
        </div>
        {playlist_context}
    "#);

    let root_prefix = if video.path == "" { "".to_string() } else { "../".repeat(video.path.split('/').count()) };
    let html = layout(&root_prefix, site, &body, title, breadcrumbs);

    let video_dir = context.build_dir.join(&video.path);
    fs::create_dir_all(&video_dir).unwrap();
    let index_path = video_dir.join("index.html");
    fs::write(index_path, html).unwrap();
}
