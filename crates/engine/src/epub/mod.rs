//! EPUB 3 book assembly. Parsing, assets and metadata are isolated from packaging.
mod assets;
mod dom;
mod math;
mod metadata;
use anyhow::{Context, Result, ensure};
use asciidoc_html5::{Document, Options};
use asciidoc_parser::blocks::{Block, FindBlocks, IsBlock};
use assets::Assets;
use dom::{Element, Node, clean, flatten_sections, parse, xml};
use metadata::Metadata;
use percent_encoding::{NON_ALPHANUMERIC, percent_decode_str, utf8_percent_encode};
use std::{
    collections::{BTreeMap, BTreeSet},
    io::{Cursor, Write},
};
use zip::{CompressionMethod, ZipWriter, write::SimpleFileOptions};

struct Chapter {
    href: String,
    title: String,
    body: Element,
    kind: String,
}
struct Heading {
    level: usize,
    title: String,
    href: String,
}

pub fn export(
    doc: &Document<'_>,
    opts: &Options,
    options: &crate::RenderOptions,
) -> Result<Vec<u8>> {
    let html = crate::presentation::inject(
        asciidoc_html5::convert_document_with(doc, &opts.clone().standalone(false)),
        &options.diagrams,
    )?;
    let meta = Metadata::new(doc, options);
    let mut body = parse(&html)?;
    math::render(&mut body, false)?;
    let kinds: BTreeMap<_, _> = doc
        .descendant_blocks()
        .filter_map(|block| {
            if !matches!(block, Block::Section(_)) {
                return None;
            }
            let kind = block.resolved_style()?;
            matches!(
                kind,
                "preface"
                    | "appendix"
                    | "bibliography"
                    | "glossary"
                    | "index"
                    | "abstract"
                    | "dedication"
                    | "acknowledgments"
            )
            .then(|| block.id().map(|id| (id.to_owned(), kind.to_owned())))
            .flatten()
        })
        .collect();
    body.walk_mut(&mut |e| {
        if let Some(kind) = kinds.get(e.attr("id")) {
            e.set("epub:type", kind);
        }
        Ok(())
    })?;
    clean(&mut body, false)?;
    remove_toc(&mut body);
    body.children = flatten_sections(body.children);
    if !body
        .children
        .iter()
        .any(|n| matches!(n, Node::Element(e) if e.tag == "h1"))
    {
        let mut heading = Element::new("h1");
        heading.children.push(Node::Text(meta.title.clone()));
        body.children.insert(0, Node::Element(heading));
    }
    let mut assets = Assets::new(options);
    assets.rewrite(&mut body, &options.base_dir)?;
    if !meta.is_book() && !meta.authors.is_empty() {
        let mut byline = Element::new("p");
        byline.set("class", "byline");
        byline.children.push(Node::Text(meta.authors.join(", ")));
        if !meta.attr("username").is_empty() {
            let source = format!("avatars/{}.jpg", meta.attr("username"));
            let href = assets.add(&source, &options.base_dir.join(meta.attr("imagesdir")))?;
            let mut image = Element::new("img");
            image.set("src", href);
            image.set("alt", "Author avatar");
            image.set("class", "avatar");
            byline.children.insert(0, Node::Element(image));
        }
        body.children.insert(1, Node::Element(byline));
        if let Some(Node::Element(last)) = body.children.last_mut() {
            last.set("class", format!("{} article-end", last.attr("class")));
        }
    }
    let mut styles = include_str!("epub3.css").to_owned();
    let mut css3 = include_str!("epub3-css3-only.css").to_owned();
    if !meta.attr("epub3-stylesdir").is_empty() {
        let directory = options.base_dir.join(meta.attr("epub3-stylesdir"));
        let (_, bytes) = assets.read("epub3.css", &directory)?;
        styles = assets.stylesheet(
            std::str::from_utf8(&bytes).context("epub3.css must be UTF-8")?,
            &directory,
        )?;
        if directory.join("epub3-css3-only.css").exists() {
            let (_, bytes) = assets.read("epub3-css3-only.css", &directory)?;
            css3 = assets.stylesheet(
                std::str::from_utf8(&bytes).context("epub3-css3-only.css must be UTF-8")?,
                &directory,
            )?;
        }
    }
    styles.push_str(include_str!("../math.css"));
    let mut chapters = front_matter(&meta, &mut assets, options)?;
    let front_count = chapters.len();
    allocate_ids(&mut body)?;
    chapters.extend(split(body, &meta));
    let cover = if !meta.attr("front-cover-image").is_empty() {
        let value = meta.attr("front-cover-image");
        let source = value
            .strip_prefix("image:")
            .map(|s| s.split('[').next().unwrap_or(s))
            .unwrap_or(value);
        let href = assets.add(source, &options.base_dir.join(meta.attr("imagesdir")))?;
        ensure!(
            assets
                .entries
                .iter()
                .any(|a| a.href == href && a.media.starts_with("image/")),
            "cover requires an image asset"
        );
        Some(href)
    } else {
        None
    };
    let mut targets = BTreeMap::new();
    for chapter in &chapters {
        chapter.body.walk(&mut |e| {
            if !e.attr("id").is_empty() {
                targets
                    .entry(e.attr("id").to_owned())
                    .or_insert_with(|| chapter.href.clone());
            }
        });
    }
    let known: BTreeSet<_> = chapters
        .iter()
        .map(|c| c.href.clone())
        .chain(assets.entries.iter().map(|a| a.href.clone()))
        .collect();
    let mut headings = Vec::new();
    for chapter in &mut chapters {
        rewrite_links(&mut chapter.body, &chapter.href, &targets, &known)?;
        semantics(&mut chapter.body)?;
        highlight(&mut chapter.body, &meta)?;
        chapter.body.walk(&mut |e| {
            if let Some(level) = heading_level(e)
                && level
                    <= meta
                        .depth("outlinelevels", meta.depth("toclevels", 2))
                        .max(meta.depth("toclevels", 2))
            {
                headings.push(Heading {
                    level,
                    title: e.text(),
                    href: format!("{}#{}", chapter.href, fragment(e.attr("id"))),
                });
            }
        });
    }
    let toc_title = if meta.attr("toc-title").is_empty() {
        "Contents"
    } else {
        meta.attr("toc-title")
    };
    let outline: Vec<_> = headings
        .iter()
        .filter(|h| h.level <= meta.depth("outlinelevels", meta.depth("toclevels", 2)))
        .collect();
    let visible: Vec<_> = headings
        .iter()
        .filter(|h| h.level <= meta.depth("toclevels", 2))
        .collect();
    let nav = navigation(&outline, toc_title);
    let mut nav_body = format!(
        "<nav epub:type=\"toc\" id=\"toc\" role=\"doc-toc\"><h1>{}</h1>{nav}</nav>",
        xml(toc_title)
    );
    nav_body.push_str("<nav epub:type=\"landmarks\" hidden=\"hidden\"><h2>Landmarks</h2><ol>");
    if cover.is_some() {
        nav_body.push_str("<li><a epub:type=\"cover\" href=\"cover.xhtml\">Cover</a></li>");
    }
    nav_body.push_str(&format!(
        "<li><a epub:type=\"bodymatter\" href=\"{}\">Start reading</a></li></ol></nav>",
        xml(&chapters[front_count].href)
    ));
    let mut manifest = String::from(
        "<item id=\"nav\" href=\"nav.xhtml\" media-type=\"application/xhtml+xml\" properties=\"nav\"/><item id=\"ncx\" href=\"toc.ncx\" media-type=\"application/x-dtbncx+xml\"/><item id=\"style\" href=\"styles.css\" media-type=\"text/css\"/><item id=\"css3\" href=\"epub3-css3-only.css\" media-type=\"text/css\"/>",
    );
    let mut spine = String::new();
    if meta.visible_toc() {
        manifest
            .push_str("<item id=\"toc\" href=\"toc.xhtml\" media-type=\"application/xhtml+xml\"/>");
    }
    if cover.is_some() {
        manifest.push_str(
            "<item id=\"cover\" href=\"cover.xhtml\" media-type=\"application/xhtml+xml\"/>",
        );
        spine.push_str("<itemref idref=\"cover\"/>");
    }
    for (i, chapter) in chapters.iter().enumerate() {
        if i == front_count && meta.visible_toc() {
            spine.push_str("<itemref idref=\"toc\"/>");
        }
        let mut properties = BTreeSet::new();
        chapter.body.walk(&mut |e| {
            if e.tag == "math" {
                properties.insert("mathml".to_owned());
            }
            if e.tag == "svg" {
                properties.insert("svg".to_owned());
            }
        });
        for property in meta.attr("epub-properties").split_whitespace() {
            ensure!(
                matches!(property, "svg" | "mathml"),
                "unsupported static EPUB property: {property}"
            );
            properties.insert(property.to_owned());
        }
        let props = if properties.is_empty() {
            String::new()
        } else {
            format!(
                " properties=\"{}\"",
                properties.into_iter().collect::<Vec<_>>().join(" ")
            )
        };
        manifest.push_str(&format!(
            "<item id=\"chapter-{i}\" href=\"{}\" media-type=\"application/xhtml+xml\"{props}/>",
            xml(&chapter.href)
        ));
        spine.push_str(&format!("<itemref idref=\"chapter-{i}\"/>"));
    }
    let font_css = fonts(&mut assets, &meta)?;
    for (i, asset) in assets.entries.iter().enumerate() {
        let property = if cover.as_ref() == Some(&asset.href) {
            " properties=\"cover-image\""
        } else {
            ""
        };
        manifest.push_str(&format!(
            "<item id=\"asset-{i}\" href=\"{}\" media-type=\"{}\"{property}/>",
            xml(&asset.href),
            asset.media
        ));
    }
    manifest
        .push_str("<item id=\"fonts-style\" href=\"epub3-fonts.css\" media-type=\"text/css\"/>");
    let metadata = meta.package()?;
    let package = format!(
        "<?xml version=\"1.0\" encoding=\"UTF-8\"?><package xmlns=\"http://www.idpf.org/2007/opf\" version=\"3.0\" unique-identifier=\"book-id\" xml:lang=\"{}\"><metadata xmlns:dc=\"http://purl.org/dc/elements/1.1/\">{metadata}</metadata><manifest>{manifest}</manifest><spine toc=\"ncx\">{spine}</spine></package>",
        xml(&meta.language)
    );
    let mut files = vec![
        (
            "META-INF/container.xml".to_owned(),
            CONTAINER.as_bytes().to_vec(),
        ),
        ("EPUB/package.opf".into(), package.into_bytes()),
        ("EPUB/styles.css".into(), styles.into_bytes()),
        ("EPUB/epub3-css3-only.css".into(), css3.into_bytes()),
        (
            "EPUB/nav.xhtml".into(),
            xhtml(toc_title, &meta.language, &nav_body).into_bytes(),
        ),
        ("EPUB/toc.ncx".into(), ncx(&meta, &outline).into_bytes()),
    ];
    if meta.visible_toc() {
        let content = format!(
            "<nav epub:type=\"toc\" role=\"doc-toc\"><h1>{}</h1>{}</nav>",
            xml(toc_title),
            navigation(&visible, toc_title)
        );
        files.push((
            "EPUB/toc.xhtml".into(),
            xhtml(toc_title, &meta.language, &content).into_bytes(),
        ));
    }
    if let Some(cover) = cover {
        let content = format!(
            "<section epub:type=\"cover\" class=\"cover\"><img src=\"{}\" alt=\"{}\"/></section>",
            xml(&cover),
            xml(&meta.title)
        );
        files.push((
            "EPUB/cover.xhtml".into(),
            xhtml("Cover", &meta.language, &content).into_bytes(),
        ));
    }
    for chapter in chapters {
        let content = format!(
            "<section epub:type=\"{}\">{}</section>",
            chapter.kind,
            chapter.body.inner_html()
        );
        files.push((
            format!("EPUB/{}", chapter.href),
            xhtml(&chapter.title, &meta.language, &content).into_bytes(),
        ));
    }
    files.push(("EPUB/epub3-fonts.css".into(), font_css.into_bytes()));
    files.extend(
        assets
            .entries
            .into_iter()
            .map(|a| (format!("EPUB/{}", a.href), a.bytes)),
    );
    archive(files)
}

fn remove_toc(root: &mut Element) {
    root.children
        .retain(|n| !matches!(n, Node::Element(e) if e.attr("id") == "toc"));
    for node in &mut root.children {
        if let Node::Element(e) = node {
            remove_toc(e);
        }
    }
}
fn allocate_ids(root: &mut Element) -> Result<()> {
    let mut reserved = BTreeSet::new();
    root.walk(&mut |e| {
        if !e.attr("id").is_empty() {
            reserved.insert(e.attr("id").to_owned());
        }
    });
    let mut used = BTreeSet::new();
    let mut number = 0;
    root.walk_mut(&mut |e| {
        if e.attr("id").is_empty() && heading_level(e).is_none() {
            return Ok(());
        }
        let original = if e.attr("id").is_empty() {
            number += 1;
            format!("heading-{number}")
        } else {
            e.attr("id").to_owned()
        };
        ensure!(
            !original.chars().any(char::is_whitespace),
            "EPUB IDs cannot contain whitespace: {original}"
        );
        let mut id = original.clone();
        let mut suffix = 2;
        while used.contains(&id) || (e.attr("id").is_empty() && reserved.contains(&id)) {
            id = format!("{original}-{suffix}");
            suffix += 1;
        }
        used.insert(id.clone());
        e.set("id", id);
        Ok(())
    })
}
fn heading_level(e: &Element) -> Option<usize> {
    match e.tag.as_str() {
        "h1" | "h2" => Some(1),
        "h3" => Some(2),
        "h4" => Some(3),
        "h5" => Some(4),
        "h6" => Some(5),
        _ => None,
    }
}
fn split(body: Element, meta: &Metadata) -> Vec<Chapter> {
    let split_tag = format!("h{}", meta.depth("epub-chapter-level", 1).min(5) + 1);
    let mut chapters = Vec::new();
    let mut current = Element::new("body");
    let mut title = meta.title.clone();
    let mut href = if meta.is_book() {
        "_preamble.xhtml"
    } else {
        "content.xhtml"
    }
    .to_owned();
    let mut names = BTreeSet::from([
        "_preamble.xhtml".to_owned(),
        "content.xhtml".into(),
        "cover.xhtml".into(),
        "nav.xhtml".into(),
        "toc.xhtml".into(),
    ]);
    for node in body.children {
        if meta.is_book() && matches!(&node, Node::Element(e) if e.tag == split_tag) {
            if !current.children.is_empty() {
                let kind = chapter_kind(&current);
                chapters.push(Chapter {
                    href,
                    title,
                    body: current,
                    kind,
                });
                current = Element::new("body");
            }
            let Node::Element(heading) = &node else {
                unreachable!()
            };
            title = heading.text();
            let base: String = heading
                .attr("id")
                .chars()
                .map(|c| {
                    if c.is_ascii_alphanumeric() || matches!(c, '-' | '_') {
                        c.to_string()
                    } else {
                        format!("-{:x}", c as u32)
                    }
                })
                .collect();
            let base = if base.is_empty() {
                format!("chapter-{}", chapters.len() + 1)
            } else {
                base
            };
            href = format!("{base}.xhtml");
            let mut suffix = 2;
            while !names.insert(href.clone()) {
                href = format!("{base}-{suffix}.xhtml");
                suffix += 1;
            }
        }
        current.children.push(node);
    }
    let kind = if meta.is_book() {
        chapter_kind(&current)
    } else {
        "bodymatter".into()
    };
    chapters.push(Chapter {
        href,
        title,
        body: current,
        kind,
    });
    chapters
}
fn chapter_kind(body: &Element) -> String {
    body.children
        .iter()
        .find_map(|n| match n {
            Node::Element(e) if heading_level(e).is_some() && !e.attr("epub:type").is_empty() => {
                Some(e.attr("epub:type").to_owned())
            }
            _ => None,
        })
        .unwrap_or_else(|| "chapter".into())
}
fn fragment(id: &str) -> String {
    const FRAGMENT: &percent_encoding::AsciiSet = &NON_ALPHANUMERIC
        .remove(b'-')
        .remove(b'_')
        .remove(b'.')
        .remove(b'~');
    utf8_percent_encode(id, FRAGMENT).to_string()
}
fn rewrite_links(
    root: &mut Element,
    current: &str,
    targets: &BTreeMap<String, String>,
    known: &BTreeSet<String>,
) -> Result<()> {
    let mut local = BTreeSet::new();
    root.walk(&mut |e| {
        if !e.attr("id").is_empty() {
            local.insert(e.attr("id").to_owned());
        }
    });
    root.walk_mut(&mut |e| {
        if e.tag != "a" || e.attr("href").is_empty() {
            return Ok(());
        }
        let href = e.attr("href").to_owned();
        if href == "#" {
            e.set("href", current);
            return Ok(());
        }
        if href.starts_with('#')
            || (href.split('#').next().is_some_and(|p| p.ends_with(".adoc")) && href.contains('#'))
        {
            let encoded = href
                .split_once('#')
                .map(|(_, id)| id.trim_start_matches('#'))
                .unwrap_or("");
            let id = percent_decode_str(encoded)
                .decode_utf8()
                .context("invalid UTF-8 fragment")?;
            let destination = if local.contains(id.as_ref()) {
                current
            } else {
                targets
                    .get(id.as_ref())
                    .with_context(|| format!("unresolved EPUB cross-reference: {href}"))?
            };
            e.set("href", format!("{destination}#{}", fragment(&id)));
        } else if !href.contains(':') && !href.starts_with("//") {
            let (file, id) = href.split_once('#').unwrap_or((&href, ""));
            ensure!(
                known.contains(file),
                "EPUB link is not part of this book: {href}"
            );
            if !id.is_empty() {
                ensure!(
                    targets
                        .get(percent_decode_str(id).decode_utf8()?.as_ref())
                        .is_some_and(|s| s == file),
                    "unresolved EPUB link: {href}"
                );
            }
        } else {
            ensure!(
                !href.to_ascii_lowercase().starts_with("file:"),
                "EPUB links must not reference local files: {href}"
            );
        }
        Ok(())
    })
}
fn semantics(root: &mut Element) -> Result<()> {
    root.walk_mut(&mut |e| {
        if e.class("conum")
            && let Ok(number) = e.text().trim_matches(['(', ')', ' ']).parse::<u32>()
            && let Some(label) = (1..=20)
                .contains(&number)
                .then(|| char::from_u32(0x245f + number))
                .flatten()
        {
            e.children = vec![Node::Text(label.to_string())];
        }
        if e.tag == "ul" && e.class("checklist") {
            for node in &mut e.children {
                if let Node::Element(item) = node
                    && item.tag == "li"
                    && item.text().trim_start().starts_with(['✓', '❏'])
                {
                    item.set(
                        "class",
                        format!("{} task-list-item", item.attr("class"))
                            .trim()
                            .to_owned(),
                    );
                }
            }
        }
        if e.class("colist") {
            for node in &mut e.children {
                if let Node::Element(list) = node {
                    if list.tag != "ol" {
                        continue;
                    }
                    for (i, item) in list
                        .children
                        .iter_mut()
                        .filter_map(|n| {
                            if let Node::Element(li) = n {
                                Some(li)
                            } else {
                                None
                            }
                        })
                        .enumerate()
                    {
                        let number = item.attr("value").parse::<u32>().unwrap_or(i as u32 + 1);
                        let label = if (1..=20).contains(&number) {
                            char::from_u32(0x245f + number).unwrap().to_string()
                        } else {
                            format!("({number})")
                        };
                        let paragraph = item.children.iter_mut().find_map(|n| match n {
                            Node::Element(p) if p.tag == "p" => Some(p),
                            _ => None,
                        });
                        if let Some(p) = paragraph {
                            p.children.insert(0, Node::Text(format!("{label} ")));
                        } else {
                            item.children.insert(0, Node::Text(format!("{label} ")));
                        }
                    }
                }
            }
        }
        if e.tag == "a" && (e.class("footnote") || e.attr("id").starts_with("_footnoteref_")) {
            e.set("epub:type", "noteref");
            e.set("role", "doc-noteref");
        }
        if e.class("footnote") && e.tag == "div" {
            e.set("epub:type", "footnote");
            e.set("role", "doc-footnote");
        }
        if e.attr("id") == "footnotes" {
            e.set("epub:type", "footnotes");
        }
        for kind in [
            "preface",
            "appendix",
            "bibliography",
            "glossary",
            "index",
            "abstract",
            "dedication",
            "acknowledgments",
        ] {
            if e.class(kind) {
                e.set("epub:type", kind);
            }
        }
        Ok(())
    })
}
fn front_matter(
    meta: &Metadata,
    assets: &mut Assets<'_>,
    options: &crate::RenderOptions,
) -> Result<Vec<Chapter>> {
    let mut files = Vec::new();
    if !meta.attr("epub3-frontmatterdir").is_empty() {
        let directory = assets.local(meta.attr("epub3-frontmatterdir"), &options.base_dir)?;
        for entry in std::fs::read_dir(&directory)? {
            let file = entry?.path();
            let name = file.file_name().unwrap_or_default().to_string_lossy();
            if name.starts_with("front-matter") && name.ends_with(".html") {
                files.push(file);
            }
        }
    } else if options.safe_mode != "secure" && options.base_dir.join("front-matter.html").exists() {
        files.push(options.base_dir.join("front-matter.html"));
    }
    files.sort();
    let mut result = Vec::new();
    for file in &files {
        let directory = file.parent().context("front matter requires a directory")?;
        let (_, bytes) = assets.read(&file.file_name().unwrap().to_string_lossy(), directory)?;
        let mut body = parse(std::str::from_utf8(&bytes).context("front matter must be UTF-8")?)?;
        clean(&mut body, false)?;
        assets.rewrite(&mut body, directory)?;
        allocate_ids(&mut body)?;
        let mut title = None;
        body.walk(&mut |e| {
            if title.is_none() && heading_level(e).is_some() {
                title = Some(e.text());
            }
        });
        let href = format!("{}.xhtml", file.file_stem().unwrap().to_string_lossy());
        result.push(Chapter {
            href,
            title: title.unwrap_or_else(|| "Front matter".into()),
            body,
            kind: "frontmatter".into(),
        });
    }
    Ok(result)
}
fn navigation(headings: &[&Heading], _title: &str) -> String {
    fn level(headings: &[&Heading], index: &mut usize, minimum: usize) -> String {
        let mut out = String::from("<ol>");
        while *index < headings.len() && headings[*index].level >= minimum {
            let h = &headings[*index];
            let depth = h.level;
            out.push_str(&format!(
                "<li><a href=\"{}\">{}</a>",
                xml(&h.href),
                xml(&h.title)
            ));
            *index += 1;
            if *index < headings.len() && headings[*index].level > depth {
                out.push_str(&level(headings, index, headings[*index].level));
            }
            out.push_str("</li>");
        }
        out.push_str("</ol>");
        out
    }
    level(headings, &mut 0, 1)
}
fn ncx(meta: &Metadata, headings: &[&Heading]) -> String {
    fn points(headings: &[&Heading], index: &mut usize, minimum: usize) -> String {
        let mut out = String::new();
        while *index < headings.len() && headings[*index].level >= minimum {
            let h = headings[*index];
            let order = *index + 1;
            *index += 1;
            out.push_str(&format!("<navPoint id=\"nav-{order}\" playOrder=\"{order}\"><navLabel><text>{}</text></navLabel><content src=\"{}\"/>", xml(&h.title), xml(&h.href)));
            if *index < headings.len() && headings[*index].level > h.level {
                out.push_str(&points(headings, index, headings[*index].level));
            }
            out.push_str("</navPoint>");
        }
        out
    }
    let points = points(headings, &mut 0, 1);
    let depth = headings.iter().map(|h| h.level).max().unwrap_or(1);
    format!(
        "<?xml version=\"1.0\" encoding=\"UTF-8\"?><ncx xmlns=\"http://www.daisy.org/z3986/2005/ncx/\" version=\"2005-1\"><head><meta name=\"dtb:uid\" content=\"{}\"/><meta name=\"dtb:depth\" content=\"{depth}\"/><meta name=\"dtb:totalPageCount\" content=\"0\"/><meta name=\"dtb:maxPageNumber\" content=\"0\"/></head><docTitle><text>{}</text></docTitle><navMap>{points}</navMap></ncx>",
        xml(&meta.identifier),
        xml(&meta.title)
    )
}
fn xhtml(title: &str, language: &str, body: &str) -> String {
    format!(
        "<?xml version=\"1.0\" encoding=\"UTF-8\"?><html xmlns=\"http://www.w3.org/1999/xhtml\" xmlns:epub=\"http://www.idpf.org/2007/ops\" lang=\"{}\" xml:lang=\"{}\"><head><title>{}</title><link rel=\"stylesheet\" type=\"text/css\" href=\"epub3-fonts.css\"/><link rel=\"stylesheet\" type=\"text/css\" href=\"styles.css\"/><link rel=\"stylesheet\" type=\"text/css\" href=\"epub3-css3-only.css\"/></head><body>{body}</body></html>",
        xml(language),
        xml(language),
        xml(title)
    )
}
fn fonts(assets: &mut Assets<'_>, meta: &Metadata) -> Result<String> {
    ensure!(
        matches!(
            meta.attr("scripts"),
            "" | "latin" | "latin-ext" | "latin-cyrillic" | "multilingual"
        ),
        "scripts must be latin, latin-ext, latin-cyrillic or multilingual"
    );
    if meta.attr("epub-fonts") == "none" {
        return Ok(String::new());
    }
    let mut css = String::new();
    for (name, bytes, family, weight, style) in [
        (
            "LibertinusSerif-Regular.otf",
            &include_bytes!("fonts/LibertinusSerif-Regular.otf")[..],
            "Libertinus Serif",
            "normal",
            "normal",
        ),
        (
            "LibertinusSerif-Italic.otf",
            &include_bytes!("fonts/LibertinusSerif-Italic.otf")[..],
            "Libertinus Serif",
            "normal",
            "italic",
        ),
        (
            "LibertinusSerif-Bold.otf",
            &include_bytes!("fonts/LibertinusSerif-Bold.otf")[..],
            "Libertinus Serif",
            "bold",
            "normal",
        ),
        (
            "LibertinusSerif-BoldItalic.otf",
            &include_bytes!("fonts/LibertinusSerif-BoldItalic.otf")[..],
            "Libertinus Serif",
            "bold",
            "italic",
        ),
        (
            "DejaVuSansMono.ttf",
            &include_bytes!("fonts/DejaVuSansMono.ttf")[..],
            "DejaVu Sans Mono",
            "normal",
            "normal",
        ),
        (
            "DejaVuSansMono-Bold.ttf",
            &include_bytes!("fonts/DejaVuSansMono-Bold.ttf")[..],
            "DejaVu Sans Mono",
            "bold",
            "normal",
        ),
        (
            "DejaVuSansMono-Oblique.ttf",
            &include_bytes!("fonts/DejaVuSansMono-Oblique.ttf")[..],
            "DejaVu Sans Mono",
            "normal",
            "italic",
        ),
        (
            "DejaVuSansMono-BoldOblique.ttf",
            &include_bytes!("fonts/DejaVuSansMono-BoldOblique.ttf")[..],
            "DejaVu Sans Mono",
            "bold",
            "italic",
        ),
    ] {
        let href = format!("fonts/{name}");
        css.push_str(&format!("@font-face {{ font-family: '{family}'; font-weight: {weight}; font-style: {style}; src: url('{href}'); }}\n"));
        assets.entries.push(assets::Asset {
            href,
            bytes: bytes.to_vec(),
            media: if name.ends_with(".otf") {
                "font/otf"
            } else {
                "font/ttf"
            },
        });
    }
    css.push_str("p, li, dd, td { font-family: 'Libertinus Serif', serif; } pre, code { font-family: 'DejaVu Sans Mono', monospace; }");
    assets.entries.push(assets::Asset {
        href: "font-licenses.txt".into(),
        media: "text/plain",
        bytes: include_bytes!("fonts/LICENSE.txt").to_vec(),
    });
    Ok(css)
}
fn archive(files: Vec<(String, Vec<u8>)>) -> Result<Vec<u8>> {
    let mut zip = ZipWriter::new(Cursor::new(Vec::new()));
    zip.start_file(
        "mimetype",
        SimpleFileOptions::default().compression_method(CompressionMethod::Stored),
    )?;
    zip.write_all(b"application/epub+zip")?;
    for (name, bytes) in files {
        if [".xml", ".xhtml", ".opf", ".ncx"]
            .iter()
            .any(|ext| name.ends_with(ext))
        {
            roxmltree::Document::parse(std::str::from_utf8(&bytes)?)
                .with_context(|| format!("{name} is not valid XML"))?;
        }
        zip.start_file(
            name,
            SimpleFileOptions::default().compression_method(CompressionMethod::Deflated),
        )?;
        zip.write_all(&bytes)?;
    }
    Ok(zip.finish()?.into_inner())
}
fn highlight(root: &mut Element, meta: &Metadata) -> Result<()> {
    if matches!(meta.attr("source-highlighter"), "" | "none") {
        return Ok(());
    }
    use std::sync::LazyLock;
    use syntect::{
        html::{ClassStyle, ClassedHTMLGenerator},
        parsing::SyntaxSet,
        util::LinesWithEndings,
    };
    static SYNTAX: LazyLock<SyntaxSet> = LazyLock::new(SyntaxSet::load_defaults_newlines);
    root.walk_mut(&mut |element| {
        if element.tag != "code" {
            return Ok(());
        }
        let language = element
            .attr("class")
            .split_whitespace()
            .find_map(|s| s.strip_prefix("language-"))
            .unwrap_or(element.attr("data-lang"));
        let Some(syntax) = SYNTAX.find_syntax_by_token(language) else {
            return Ok(());
        };
        let mut generator =
            ClassedHTMLGenerator::new_with_class_style(syntax, &SYNTAX, ClassStyle::Spaced);
        for line in LinesWithEndings::from(&element.text()) {
            generator.parse_html_for_line_which_includes_newline(line)?;
        }
        element.children = parse(&generator.finalize())?.children;
        Ok(())
    })
}
const CONTAINER: &str = "<?xml version=\"1.0\"?><container version=\"1.0\" xmlns=\"urn:oasis:names:tc:opendocument:xmlns:container\"><rootfiles><rootfile full-path=\"EPUB/package.opf\" media-type=\"application/oebps-package+xml\"/></rootfiles></container>";
