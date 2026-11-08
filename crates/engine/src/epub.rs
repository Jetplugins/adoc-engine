use anyhow::Result;
use asciidoc_html5::{Document, Options};
use std::io::{Cursor, Write};
use zip::{CompressionMethod, ZipWriter, write::SimpleFileOptions};
fn xml(s: &str) -> String {
    s.replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
        .replace('"', "&quot;")
}
pub fn export(
    doc: &Document<'_>,
    opts: &Options,
    options: &crate::RenderOptions,
) -> Result<Vec<u8>> {
    let fragment = opts.clone().standalone(false);
    let body = crate::presentation::inject(
        asciidoc_html5::convert_document_with(doc, &fragment),
        &options.diagrams,
    )?;
    let body = xhtml(&body)?;
    let images = regex::Regex::new(r#"<img\b[^>]*\bsrc="([^"]+)"[^>]*>"#)?;
    let mut assets = Vec::new();
    let mut replacements = std::collections::BTreeMap::new();
    let mut manifest = String::new();
    for cap in images.captures_iter(&body) {
        let src = &cap[1];
        if replacements.contains_key(src) {
            continue;
        }
        let path = crate::assets::local_path(src, options)?;
        let media = crate::assets::media_type(&path)?;
        let i = assets.len();
        let name = format!(
            "assets/image-{i}.{}",
            path.extension().unwrap().to_string_lossy()
        );
        manifest.push_str(&format!(
            r#"<item id="image-{i}" href="{name}" media-type="{media}"/>"#
        ));
        assets.push((format!("EPUB/{name}"), std::fs::read(path)?));
        replacements.insert(src.to_string(), name);
    }
    let mut body = body.to_string();
    for (src, name) in replacements {
        body = body.replace(&format!("src=\"{src}\""), &format!("src=\"{name}\""));
    }
    use sha2::{Digest, Sha256};
    let identifier = format!("urn:sha256:{:x}", Sha256::digest(body.as_bytes()));
    let language = xml(options
        .attributes
        .get("lang")
        .map(String::as_str)
        .unwrap_or("en"));
    let mut nav_items = String::new();
    let parsed = roxmltree::Document::parse(&format!("<root>{body}</root>"))?
        .root_element()
        .descendants()
        .filter(|n| {
            n.is_element() && matches!(n.tag_name().name(), "h1" | "h2" | "h3" | "h4" | "h5" | "h6")
        })
        .filter_map(|n| {
            n.attribute("id").map(|id| {
                (
                    id.to_string(),
                    n.descendants()
                        .filter_map(|d| d.text().filter(|_| d.is_text()))
                        .collect::<String>(),
                )
            })
        })
        .collect::<Vec<_>>();
    for (id, text) in parsed {
        nav_items.push_str(&format!(
            "<li><a href=\"content.xhtml#{}\">{}</a></li>",
            xml(&id),
            xml(&text)
        ));
    }
    let svg_property = if body.contains("<svg") {
        r#" properties="svg""#
    } else {
        ""
    };
    let title = xml(doc.doctitle().unwrap_or("Untitled"));
    let mut zip = ZipWriter::new(Cursor::new(Vec::new()));
    let stored = SimpleFileOptions::default().compression_method(CompressionMethod::Stored);
    let compressed = SimpleFileOptions::default().compression_method(CompressionMethod::Deflated);
    zip.start_file("mimetype", stored)?;
    zip.write_all(b"application/epub+zip")?;
    let files=[
    ("META-INF/container.xml",r#"<?xml version="1.0"?><container version="1.0" xmlns="urn:oasis:names:tc:opendocument:xmlns:container"><rootfiles><rootfile full-path="EPUB/package.opf" media-type="application/oebps-package+xml"/></rootfiles></container>"#.to_string()),
    ("EPUB/package.opf",format!(r#"<?xml version="1.0"?><package xmlns="http://www.idpf.org/2007/opf" version="3.0" unique-identifier="id"><metadata xmlns:dc="http://purl.org/dc/elements/1.1/"><dc:identifier id="id">{identifier}</dc:identifier><dc:title>{title}</dc:title><dc:language>{language}</dc:language><meta property="dcterms:modified">2000-01-01T00:00:00Z</meta></metadata><manifest><item id="content" href="content.xhtml" media-type="application/xhtml+xml"{svg_property}/><item id="nav" href="nav.xhtml" media-type="application/xhtml+xml" properties="nav"/>{manifest}</manifest><spine><itemref idref="content"/></spine></package>"#)),
    ("EPUB/nav.xhtml",format!(r#"<?xml version="1.0"?><html xml:lang="{language}" lang="{language}" xmlns="http://www.w3.org/1999/xhtml" xmlns:epub="http://www.idpf.org/2007/ops"><head><title>{title}</title></head><body><nav epub:type="toc"><h1>Contents</h1><ol><li><a href="content.xhtml">{title}</a></li>{nav_items}</ol></nav></body></html>"#)),
    ("EPUB/content.xhtml",format!(r#"<?xml version="1.0"?><html xml:lang="{language}" lang="{language}" xmlns="http://www.w3.org/1999/xhtml"><head><title>{title}</title></head><body>{body}</body></html>"#))];
    for (name, data) in files {
        roxmltree::Document::parse(&data)
            .map_err(|e| anyhow::anyhow!("{name} is not valid XML: {e}"))?;
        zip.start_file(name, compressed)?;
        zip.write_all(data.as_bytes())?;
    }
    for (name, bytes) in assets {
        zip.start_file(name, compressed)?;
        zip.write_all(&bytes)?;
    }
    Ok(zip.finish()?.into_inner())
}

/// Parse HTML5 rather than trying to repair HTML with regular expressions.
fn xhtml(html: &str) -> Result<String> {
    use html5ever::tendril::TendrilSink;
    use markup5ever_rcdom::{Handle, NodeData, RcDom};
    fn emit(node: &Handle, out: &mut String) -> Result<()> {
        match &node.data {
            NodeData::Text { contents } => out.push_str(&xml(&contents.borrow())),
            NodeData::Element { name, attrs, .. } => {
                let tag = &*name.local;
                if matches!(tag, "script" | "iframe" | "object" | "embed") {
                    anyhow::bail!("active HTML element {tag} is not supported in EPUB");
                }
                out.push('<');
                out.push_str(tag);
                if tag == "svg" {
                    out.push_str(" xmlns=\"http://www.w3.org/2000/svg\"");
                }
                if tag == "math" {
                    out.push_str(" xmlns=\"http://www.w3.org/1998/Math/MathML\"");
                }
                for a in attrs.borrow().iter() {
                    if &*a.name.local == "xmlns" {
                        continue;
                    }
                    out.push(' ');
                    if let Some(prefix) = &a.name.prefix
                        && !prefix.is_empty()
                    {
                        out.push_str(prefix);
                        out.push(':');
                    }
                    out.push_str(&a.name.local);
                    out.push_str("=\"");
                    out.push_str(&xml(&a.value));
                    out.push('"');
                }
                if matches!(
                    tag,
                    "img"
                        | "br"
                        | "hr"
                        | "col"
                        | "wbr"
                        | "input"
                        | "meta"
                        | "link"
                        | "source"
                        | "area"
                        | "base"
                ) {
                    out.push_str(" />");
                } else {
                    out.push('>');
                    for child in node.children.borrow().iter() {
                        emit(child, out)?;
                    }
                    out.push_str("</");
                    out.push_str(tag);
                    out.push('>');
                }
            }
            _ => {
                for child in node.children.borrow().iter() {
                    emit(child, out)?;
                }
            }
        }
        Ok(())
    }
    let dom = html5ever::parse_document(RcDom::default(), Default::default()).one(html);
    let html = dom
        .document
        .children
        .borrow()
        .iter()
        .find(|n| matches!(&n.data,NodeData::Element{name,..}if &*name.local=="html"))
        .cloned()
        .ok_or_else(|| anyhow::anyhow!("missing HTML root"))?;
    let body = html
        .children
        .borrow()
        .iter()
        .find(|n| matches!(&n.data,NodeData::Element{name,..}if &*name.local=="body"))
        .cloned()
        .ok_or_else(|| anyhow::anyhow!("missing body"))?;
    let mut out = String::new();
    for child in body.children.borrow().iter() {
        emit(child, &mut out)?;
    }
    Ok(out)
}
