// SPDX-FileCopyrightText: 2024-2025 Simon Repp
// SPDX-License-Identifier: AGPL-3.0-or-later

use std::env;
use std::fs;
use std::fs::DirEntry;
use std::hash::{DefaultHasher, Hash, Hasher};
use std::ops::Deref;
use std::path::{Path, PathBuf};

use base64::Engine;
use base64::engine::general_purpose::URL_SAFE_NO_PAD;
use enolib::HtmlPrinter;
use indoc::formatdoc;
use pulldown_cmark::{CodeBlockKind, Event, Parser, Tag, TagEnd};
use pulldown_cmark::html;
use slug::slugify;

struct Docs {
	examples: Vec<Page>,
	index: Page,
	reference: Vec<Page>,
	topics: Vec<Page>,
    version: &'static str
}

#[derive(PartialEq)]
struct Page {
	content: String,
	slug: String,
	title: String
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

	render_page(
		&docs_out_dir,
		&docs,
		&docs.index,
		docs.topics.first()
	);

	let mut topics_iter = docs.topics.iter().peekable();
	while let Some(page) = topics_iter.next() {
		render_page(
			&docs_out_dir,
			&docs,
			page,
			topics_iter.peek().copied().or_else(|| docs.examples.first())
		);
	}

	let mut examples_iter = docs.examples.iter().peekable();
	while let Some(page) = examples_iter.next() {
		render_page(
			&docs_out_dir,
			&docs,
			page,
			examples_iter.peek().copied().or_else(|| docs.reference.first())
		);
	}

	let mut reference_iter = docs.reference.iter().peekable();
	while let Some(page) = reference_iter.next() {
		render_page(
			&docs_out_dir,
			&docs,
			page,
			reference_iter.peek().copied()
		);
	}

	fs::copy(
		"assets/fira-mono-v14-latin_latin-ext-regular.woff2",
		docs_out_dir.join("fira-mono-v14-latin_latin-ext-regular.woff2")
	).unwrap();

	fs::copy(
		"assets/titillium-web-v15-latin_latin-ext-regular.woff2",
		docs_out_dir.join("titillium-web-v15-latin_latin-ext-regular.woff2")
	).unwrap();

	fs::copy(
		"assets/titillium-web-v15-latin_latin-ext-italic.woff2",
		docs_out_dir.join("titillium-web-v15-latin_latin-ext-italic.woff2")
	).unwrap();

	fs::copy(
		"assets/styles.css",
		docs_out_dir.join("styles.css")
	).unwrap();
}

fn layout(body: &str, docs: &Docs, active_page: &Page) -> String {
	let section_links = |pages: &[Page]| {
		pages
			.iter()
			.map(|page| {
				let active = if page == active_page { r#"class="active" "# } else { "" };
				let slug = &page.slug;
				let title = &page.title;

				format!(r#"<a {active}href="{slug}.html">{title}</a>"#)
			})
			.collect::<Vec<String>>()
			.join("\n")
	};

	let examples = section_links(&docs.examples);
	let reference = section_links(&docs.reference);
	let topics = section_links(&docs.topics);
	let index_active = if active_page == &docs.index { r#" class="active""# } else { "" };

	let styles_css_hash = url_safe_hash_base64(include_bytes!("../assets/styles.css"));

	let title = &active_page.title;
	let version = docs.version;

	formatdoc!(r##"
		<!doctype html>
		<html>
			<head>
				<title>{title}</title>
				<meta charset="utf-8">
		        <meta name="description" content="{title}">
		        <meta name="viewport" content="width=device-width, initial-scale=1">
		        <link href="styles.css?{styles_css_hash}" rel="stylesheet">
			</head>
			<body>
				<header>
					<a class="title" href="index.html">
						<span{index_active}>Hyper 8 Video System {version}</span>
					</a>
					<a class="open_nav" href="#nav">☰</a>
				</header>

				<div class="split">
					<nav id="nav">
						<div class="nav_inner">
							<a class="close_nav" href="#">✕</a>

							<span>Topics</span>
							{topics}

							<span>Examples</span>
							{examples}

							<span>Reference</span>
							{reference}
						</div>
					</nav>
					<main>
						{body}
					</main>
				</div>
			</body>
		</html>
	"##)
}

fn read_docs() -> Docs {
	let index_content = markdown_to_html(include_str!("../index.md"));

	let version_display = env!("HYPER8_VERSION_DISPLAY");

	let index = Page {
		content: index_content,
		slug: String::from("index"),
		title: format!("Hyper 8 Video System {version_display}")
	};

	let examples = read_pages(&Path::new("examples"));
	let reference = read_pages(&Path::new("reference"));
	let topics = read_pages(&Path::new("topics"));

	Docs {
		examples,
		index,
		reference,
		topics,
		version: version_display
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
	docs_out_dir: &Path,
	docs: &Docs,
	page: &Page,
	next_page: Option<&Page>
) {
	let content = &page.content;

	let body = if let Some(next_page) = next_page {
		let next_page_slug = &next_page.slug;
		let next_page_title = &next_page.title;

		formatdoc!(r#"
			{content}

			<div class="next_page">
				Next page: <a href="{next_page_slug}.html">{next_page_title}</a>
			</div>
		"#)
	} else {
		content.clone()
	};

	let html = layout(&body, docs, page);

	let out_path = docs_out_dir.join(&page.slug).with_extension("html");

	fs::write(out_path, html).unwrap();
}

pub fn url_safe_hash_base64(hashable: &impl Hash) -> String {
    let mut hasher = DefaultHasher::new();
    hashable.hash(&mut hasher);
    let hash = hasher.finish();
    URL_SAFE_NO_PAD.encode(hash.to_le_bytes())
}
