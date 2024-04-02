use std::fs;

use indoc::formatdoc;

use crate::{Context, Playlist, Site, Video};
use crate::build::Breadcrumb;
use crate::build::render::layout;
use crate::build::widgets::{copy_link, downloads, player, thumbnail};

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

    let playlist_context = match playlist {
        Some(playlist) => {
            let position = playlist.videos
                .iter()
                .position(|video_iterated| video_iterated.path == video.path)
                .unwrap();

            let mut out = String::new();

            let label = playlist.label();
            let t_videos = &site.language.translations.videos;
            let video_count = playlist.videos.len();
            let header = formatdoc!(r#"
                <div>
                    {label}<br>
                    {video_count} {t_videos}
                </div>
            "#);

            out.push_str(&header);

            let adjacent_shown: usize = 2;

            // TODO: Below a certain threshold just show all videos (at least 6 or so)
            // TODO: Make the display a bit more compact too so that more videos can fit
            //       (but also need to determine what to do with different aspect ratios as that changes the visual proportions)

            let previous_videos = position;
            if previous_videos > adjacent_shown {
                let hidden_videos = previous_videos - adjacent_shown;
                let more = formatdoc!(r#"
                    <div class="more">
                        {hidden_videos} previous videos<br>
                        <a href="..{index_suffix}">See entire playlist</a>
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

                let (current, href) = if index == position {
                    ("current", String::new())
                } else {
                    ("", format!("../{slug}{index_suffix}"))
                };

                let number = index + 1;
                let line = formatdoc!(r#"
                    <a class="{current}" href="{href}">
                        {thumbnail_rendered}
                        <div class="description">
                            {number} {label}
                        </div>
                    </a>
                "#);

                out.push_str(&line);
            }

            let remaining_videos = playlist.videos.len() - position - 1;
            if remaining_videos > adjacent_shown {
                let hidden_videos = remaining_videos - adjacent_shown;
                let more = formatdoc!(r#"
                    <div class="more">
                        {hidden_videos} more videos<br>
                        <a href="..{index_suffix}">See entire playlist</a>
                    </div>
                "#);

                out.push_str(&more);
            }

            out
        }
        None => String::new()
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
        <h1 class="video_title">{title}</h1>
        {release_date}
        <p>
            {description}
        </p>
        <div class="playlist context">
            {playlist_context}
        </div>
        {downloads_widget}
        {copy_link_widget}
    "#);

    let root_prefix = if video.path == "" { "".to_string() } else { "../".repeat(video.path.split('/').count()) };
    let html = layout(&root_prefix, site, &body, title, breadcrumbs);

    let video_dir = context.build_dir.join(&video.path);
    fs::create_dir_all(&video_dir).unwrap();
    let index_path = video_dir.join("index.html");
    fs::write(index_path, html).unwrap();
}
