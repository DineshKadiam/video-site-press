use crate::Site;

pub fn copy_link(site: &Site, path: &str) -> String {
    let data_url = match &site.base_url {
        Some(base_url) => {
            let url = base_url.join(path).unwrap().to_string();
            format!(r#"data-url="{url}""#)
        }
        None => String::new()
    };

    format!(r#"
        <div>
            <button {data_url} id="copy-link">Copy link</button>
        </div>
    "#)
}