use std::fs;

use crate::{Context, Site, Video};
use crate::build::render::{layout, player};

pub fn video_html(
    context: &Context,
    video: &Video,
    site: &Site
) {
    let description = if let Some(description) = &video.description { description } else { "" };
    let title = video.title.as_deref().unwrap_or(video.slug());
    let player_rendered = player(video, "");

    let body = format!(r#"
        <h1>{title}</h1>
        {player_rendered}
        <p>
            {description}
        </p>
        <div>
            <a href="TODO">Share</a>
        </div>
    "#);

    let root_prefix = if video.path == "" { "".to_string() } else { "../".repeat(video.path.split('/').count()) };
    let html = layout(&root_prefix, site, &body, title);

    let video_dir = context.build_dir.join(&video.path);
    fs::create_dir_all(&video_dir).unwrap();
    let index_path = video_dir.join("index.html");
    fs::write(index_path, html).unwrap();
}