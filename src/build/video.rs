use std::fs;

use indoc::formatdoc;

use crate::{Context, Playlist, Site, Video};
use crate::build::Breadcrumb;
use crate::build::render::{layout, player, thumbnail};
use crate::build::widgets::copy_link;

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
    let player_rendered = player(video, "");

    let playlist_context = match playlist {
        Some(playlist) => {
            let position = playlist.videos
                .iter()
                .position(|video_iterated| video_iterated.path == video.path)
                .unwrap();

            let mut out = String::new();

            let adjacent_shown: usize = 2;

            // TODO: Below a certain threshold just show all videos (at least 6 or so)
            // TODO: Make the display a bit more compact too so that more videos can fit
            //       (but also need to determine what to do with different aspect ratios as that changes the visual proportions)

            let previous_videos = position;
            if previous_videos > adjacent_shown {
                let hidden_videos = previous_videos - adjacent_shown;
                let more = formatdoc!(r#"
                    <div class="more">
                        <a href="..{index_suffix}">{hidden_videos} previous videos</a>
                    </div>
                "#);

                out.push_str(&more);
            }

            for (index, video_iterated) in playlist.videos.iter().enumerate() {
                if (index as i16) < (position as i16) - (adjacent_shown as i16) || index > position + adjacent_shown { continue; }

                let label = video_iterated.label();
                let slug = video_iterated.slug();

                let video_prefix = if index == position {
                    String::new()
                } else {
                    format!("../{slug}/")
                };

                let thumbnail_rendered = thumbnail(video_iterated, &video_prefix);

                let number = index + 1;
                let (current, href) = if index == position {
                    ("current", String::new())
                } else {
                    ("", format!("../{slug}{index_suffix}"))
                };

                let line = formatdoc!(r#"
                    <div class="{current}">
                        <div>{thumbnail_rendered}</div>
                        <div class="description">
                            {number} <a href="{href}">{label}</a>
                        </div>
                    </div>
                "#);

                out.push_str(&line);
            }

            let remaining_videos = playlist.videos.len() - position - 1;
            if remaining_videos > adjacent_shown {
                let hidden_videos = remaining_videos - adjacent_shown;
                let more = formatdoc!(r#"
                    <div class="more">
                        <a href="..{index_suffix}">{hidden_videos} more videos</a>
                    </div>
                "#);

                out.push_str(&more);
            }

            out
        }
        None => String::new()
    };

    let copy_link_widget = copy_link(site, &video.path);

    let body = format!(r#"
        <h1>{title}</h1>
        {player_rendered}
        <p>
            {description}
        </p>
        <div class="playlist context">
            {playlist_context}
        </div>
        {copy_link_widget}
    "#);

    let root_prefix = if video.path == "" { "".to_string() } else { "../".repeat(video.path.split('/').count()) };
    let html = layout(&root_prefix, site, &body, title, breadcrumbs);

    let video_dir = context.build_dir.join(&video.path);
    fs::create_dir_all(&video_dir).unwrap();
    let index_path = video_dir.join("index.html");
    fs::write(index_path, html).unwrap();
}
