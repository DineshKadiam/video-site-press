// SPDX-FileCopyrightText: 2024-2025 Simon Repp
// SPDX-License-Identifier: AGPL-3.0-or-later

use std::env;
use std::fs;
use std::fs::DirEntry;
use std::ops::Deref;
use std::path::{Path, PathBuf};

use enolib::HtmlPrinter;
use hyper8_version::VERSION_WITHOUT_PATCH;
use indoc::formatdoc;
use pulldown_cmark::{CodeBlockKind, Event, Parser, Tag, TagEnd};
use pulldown_cmark::html;
use slug::slugify;

mod layout;

use layout::layout;

const SITE_DIRECTORY_EXAMPLES_TITLE: &str = "Site directory examples";
const SITE_DIRECTORY_TITLE: &str = "The site directory";
const TOPICS_TITLE: &str = "Topics";

struct Docs {
    index: Page,
    site_directory: Vec<Page>,
    site_directory_examples: Vec<Page>,
    topics: Vec<Page>
}

struct NextPage<'a> {
    next_section: Option<&'static str>,
    page: &'a Page
}

#[derive(PartialEq)]
struct Page {
    content: String,
    slug: String,
    title: String
}

impl<'a> NextPage<'a> {
    pub fn next_section(
        next_section: &'static str,
        page: &'a Page
    ) -> NextPage<'a> {
        NextPage {
            next_section: Some(next_section),
            page
        }
    }

    pub fn same_section(page: &'a Page) -> NextPage<'a> {
        NextPage {
            next_section: None,
            page
        }
    }
}

pub fn markdown_to_html(markdown: &str) -> String {
    let mut html_output = String::new();
    let parser = Parser::new(markdown);

    let mut inside_eno_codeblock = false;

    let parser = parser.map(|event| {
        if let Event::Start(Tag::CodeBlock(CodeBlockKind::Fenced(ref language))) = event {
            if language.deref() == "eno" {
                inside_eno_codeblock = true;
            }
        } else if let Event::End(TagEnd::CodeBlock) = event {
            inside_eno_codeblock = false;
        } else if let Event::Text(ref text) = event {
            if inside_eno_codeblock {
                // Fenced code comes with a trailing line break here, we trim it away
                let eno_source = text.trim_end();
                let document = match enolib::parse(eno_source) {
                    Ok(document) => document,
                    Err(err) => panic!("Syntax error in {} ({})", text, err)
                };
                let syntax_highlighted = document.snippet_with_options(&HtmlPrinter, false);
                return Event::Html(syntax_highlighted.into())
            }
        }

        event
    });

    html::push_html(&mut html_output, parser);

    html_output
}

pub fn main() {
    let mut args = env::args();

    let docs_out_dir = match args.nth(1) {
        Some(path) => PathBuf::from(&path),
        None => {
            eprintln!("A single argument is required (directory path to which to write the docs), aborting.");
            return;
        }
    };

    let docs = read_docs();

    if docs_out_dir.exists() {
        fs::remove_dir_all(&docs_out_dir).unwrap();
    }

    fs::create_dir(&docs_out_dir).unwrap();

    let next_page = NextPage::next_section(
        TOPICS_TITLE,
        docs.topics.first().unwrap()
    );

    render_page(
        &docs,
        &docs_out_dir,
        Some(next_page),
        &docs.index
    );

    let mut topics_iter = docs.topics.iter().peekable();
    while let Some(page) = topics_iter.next() {
        let next_page = topics_iter
            .peek()
            .map(|page| NextPage::same_section(page))
            .unwrap_or_else(|| {
                NextPage::next_section(
                    SITE_DIRECTORY_TITLE,
                    docs.site_directory.first().unwrap()
                )
            });

        render_page(
            &docs,
            &docs_out_dir,
            Some(next_page),
            page
        );
    }

    let mut site_directory_iter = docs.site_directory.iter().peekable();
    while let Some(page) = site_directory_iter.next() {
        let next_page = site_directory_iter
            .peek()
            .map(|page| NextPage::same_section(page))
            .unwrap_or_else(|| {
                NextPage::next_section(
                    SITE_DIRECTORY_EXAMPLES_TITLE,
                    docs.site_directory_examples.first().unwrap()
                )
            });

        render_page(
            &docs,
            &docs_out_dir,
            Some(next_page),
            page
        );
    }

    let mut site_directory_examples_iter = docs.site_directory_examples.iter().peekable();
    while let Some(page) = site_directory_examples_iter.next() {
        let next_page = site_directory_examples_iter
            .peek()
            .map(|page| NextPage::same_section(page));

        render_page(
            &docs,
            &docs_out_dir,
            next_page,
            page
        );
    }

    fs::copy(
        "docs/assets/fira-mono-v14-latin_latin-ext-regular.woff2",
        docs_out_dir.join("fira-mono-v14-latin_latin-ext-regular.woff2")
    ).unwrap();

    fs::copy(
        "docs/assets/titillium-web-v15-latin_latin-ext-regular.woff2",
        docs_out_dir.join("titillium-web-v15-latin_latin-ext-regular.woff2")
    ).unwrap();

    fs::copy(
        "docs/assets/titillium-web-v15-latin_latin-ext-italic.woff2",
        docs_out_dir.join("titillium-web-v15-latin_latin-ext-italic.woff2")
    ).unwrap();

    fs::copy(
        "docs/assets/docs.css",
        docs_out_dir.join("docs.css")
    ).unwrap();

    fs::copy(
        "docs/assets/favicon.png",
        docs_out_dir.join("favicon.png")
    ).unwrap();
}

fn read_docs() -> Docs {
    let index_content = markdown_to_html(include_str!("../index.md"));

    let index = Page {
        content: index_content,
        slug: String::from("index"),
        title: format!("Hyper 8 Video System {VERSION_WITHOUT_PATCH}")
    };

    let site_directory = read_pages(&Path::new("docs/site_directory"));
    let site_directory_examples = read_pages(&Path::new("docs/site_directory_examples"));
    let topics = read_pages(&Path::new("docs/topics"));

    Docs {
        index,
        site_directory,
        site_directory_examples,
        topics
    }
}

fn read_pages(dir: &Path) -> Vec<Page> {
    let mut pages: Vec<DirEntry> = fs::read_dir(dir)
        .unwrap()
        .flatten()
        .collect();

    pages.sort_by_key(|dir_entry| dir_entry.file_name());

    pages
        .into_iter()
        .map(|dir_entry| {
            let path = dir_entry.path();
            let file_stem = path.file_stem().unwrap().to_string_lossy();

            let title = match file_stem.split_once(' ') {
                Some((prefix, suffix)) => {
                    match prefix.parse::<usize>() {
                        Ok(_) => suffix.to_string(),
                        Err(_) => file_stem.to_string()
                    }
                }
                None => file_stem.to_string()
            };

            let content_markdown = fs::read_to_string(&path).unwrap();
            let content = markdown_to_html(&content_markdown);

            let slug = slugify(&title);

            Page { content, slug, title }
        })
        .collect()
}

fn render_page(
    docs: &Docs,
    docs_out_dir: &Path,
    next_page: Option<NextPage>,
    page: &Page
) {
    let content = &page.content;

    let body = if let Some(next_page) = next_page {
        let next_page_slug = &next_page.page.slug;
        let next_page_title = &next_page.page.title;

        let next_page_or_section_label;
        let next_page_or_section_title;

        match next_page.next_section {
            Some(next_section_title) => {
                next_page_or_section_label = "Next section";
                next_page_or_section_title = next_section_title;
            }
            None => {
                next_page_or_section_label = "Next page";
                next_page_or_section_title = next_page_title;
            }
        }

        formatdoc!(r#"
            {content}

            <div class="next_page">
                {next_page_or_section_label}: <a href="{next_page_slug}.html">{next_page_or_section_title}</a>
            </div>
        "#)
    } else {
        content.clone()
    };

    let html = layout(page, &body, docs);

    let out_path = docs_out_dir.join(&page.slug).with_extension("html");

    fs::write(out_path, html).unwrap();
}
