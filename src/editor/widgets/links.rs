// SPDX-FileCopyrightText: 2024 Simon Repp
// SPDX-License-Identifier: AGPL-3.0-or-later

use indoc::formatdoc;

use crate::{Link, Translations};
use crate::editor::{Input, Select};
use crate::icons;
use crate::util::{html_escape_inside_attribute, html_escape_outside_attribute};

/// Generic add link form, submit_url should be something like "/playlist/add-link/[site-path]"
pub fn add_link_form(submit_url: &str, translations: &Translations) -> String {
    let input_label = Input::text("label", translations.label_optional);
    let mut input_url = Input::url("url", translations.url);

    input_url.required();

    let t_add_link = translations.add_link;
    let t_hidden_link_with_rel_me_attribute = translations.hidden_link_with_rel_me_attribute;
    let t_regular_link = translations.regular_link;
    let t_link_type = translations.link_type;
    let t_link_with_rel_me_attribute = translations.link_with_rel_me_attribute;
    let t_new_link = translations.new_link;

    let mut select_verification = Select::new("verification", t_link_type);

    select_verification.option("", t_regular_link);
    select_verification.option("rel-me", t_link_with_rel_me_attribute);
    select_verification.option("rel-me-hidden", t_hidden_link_with_rel_me_attribute);

    formatdoc!(r#"
        <div class="form_group">
            <h2>{t_new_link}</h2>

            <form action="{submit_url}" autocomplete="off" data-warn-discard method="post">
                <div class="form_split even">
                    {input_url}
                    {input_label}
                </div>
                {select_verification}
                <button class="action">{t_add_link}</button>
            </form>
        </div>
    "#)
}

/// Generic links listing with possibility to delete items,
/// submit_url should be something like "/playlist/delete-link/[site-path]"
pub fn links(links: &[Link], submit_url: &str, translations: &Translations) -> String {
    if links.is_empty() {
        return String::new();
    }

    let t_delete = translations.delete;
    let t_hidden = translations.hidden;
    let t_really_delete = translations.really_delete;
    let delete_icon = icons::failure(t_delete);
    let links_rendered = links
        .iter()
        .map(|link| {
            let (e_label, label_value) = match &link.label {
                Some(label) => (
                    html_escape_outside_attribute(label),
                    format!(r#"value="{}""#, html_escape_inside_attribute(label))
                ),
                None => (String::new(), String::new())
            };

            let url = link.url.to_string();
            let e_url = html_escape_outside_attribute(&url);
            let url_value = html_escape_inside_attribute(&url);

            let delete_form = formatdoc!(r#"
                <form
                    action="{submit_url}"
                    data-confirm
                    data-confirm-label="{t_delete}"
                    data-confirm-text="{t_really_delete}"
                    method="post"
                    style="display: inline;">
                    <input name="label" type="hidden" {label_value}>
                    <input name="url" type="hidden" value="{url_value}">
                    <button class="danger delete_button">{delete_icon}</button>
                </form>
            "#);

            let hidden = if link.hidden { format!("[{t_hidden}]") } else { String::new() };
            let rel_me = if link.rel_me { r#"[rel="me"]"# } else { "" };

            formatdoc!(r#"
                <div class="button_group">
                    <span>
                        {e_label} {rel_me} {hidden}<br>
                        <small>
                            <a href="{url}">{e_url}</a>
                        </small>
                    </span>
                    {delete_form}
                </div>
            "#)
        })
        .collect::<Vec<String>>()
        .join("\n");

    let t_links = translations.links;
    formatdoc!(r#"
        <div class="form_group">
            <h2>{t_links}</h2>
            {links_rendered}
        </div>
    "#)
}