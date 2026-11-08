use anyhow::{Result, bail};
use html5ever::{
    serialize::{SerializeOpts, serialize},
    tendril::TendrilSink,
};
use markup5ever_rcdom::{Handle, NodeData, RcDom, SerializableHandle};
use std::collections::BTreeMap;
fn tree(html: &str) -> RcDom {
    html5ever::parse_document(RcDom::default(), Default::default()).one(html)
}
fn text(node: &Handle) -> String {
    match &node.data {
        NodeData::Text { contents } => contents.borrow().to_string(),
        _ => node.children.borrow().iter().map(text).collect(),
    }
}
fn find(node: &Handle, name: &str) -> Option<Handle> {
    if matches!(&node.data,NodeData::Element{name:n,..}if &*n.local==name) {
        return Some(node.clone());
    }
    node.children.borrow().iter().find_map(|c| find(c, name))
}
pub fn validate_diagrams(diagrams: &BTreeMap<String, String>) -> Result<()> {
    for svg in diagrams.values() {
        if svg.len() > 4 * 1024 * 1024 {
            bail!("diagram exceeds 4 MiB");
        }
        let doc = roxmltree::Document::parse(svg)?;
        if doc.root_element().tag_name().name() != "svg"
            || doc
                .descendants()
                .any(|n| n.is_element() && n.tag_name().name() == "script")
        {
            bail!("diagram must be passive SVG");
        }
    }
    Ok(())
}
pub fn inject(html: String, diagrams: &BTreeMap<String, String>) -> Result<String> {
    if diagrams.is_empty() {
        return Ok(html);
    }
    validate_diagrams(diagrams)?;
    fn replace(node: &Handle, diagrams: &BTreeMap<String, String>) {
        for child in node.children.borrow_mut().iter_mut() {
            let listing = match &child.data {
                NodeData::Element { name, attrs, .. } if &*name.local == "div" => {
                    attrs.borrow().iter().any(|a| {
                        &*a.name.local == "class"
                            && a.value
                                .split_whitespace()
                                .any(|s| matches!(s, "listingblock" | "literalblock"))
                    })
                }
                _ => false,
            };
            if listing
                && let Some(pre) = find(child, "pre")
                && let Some(svg) = diagrams.get(text(&pre).trim())
            {
                let line = match &child.data {
                    NodeData::Element { attrs, .. } => attrs
                        .borrow()
                        .iter()
                        .find(|a| &*a.name.local == "data-source-line")
                        .map(|a| format!(" data-source-line=\"{}\"", a.value))
                        .unwrap_or_default(),
                    _ => String::new(),
                };
                let dom = tree(&format!("<div class=\"imageblock\"{line}>{svg}</div>"));
                if let Some(body) = find(&dom.document, "body")
                    && !body.children.borrow().is_empty()
                {
                    *child = body.children.borrow_mut().remove(0);
                }
                continue;
            }
            replace(child, diagrams);
        }
    }
    let standalone = html
        .trim_start()
        .to_ascii_lowercase()
        .starts_with("<!doctype")
        || html.trim_start().starts_with("<html");
    let dom = tree(&html);
    replace(&dom.document, diagrams);
    let mut bytes = Vec::new();
    serialize(
        &mut bytes,
        &SerializableHandle::from(if standalone {
            dom.document.clone()
        } else {
            find(&dom.document, "body").unwrap_or(dom.document.clone())
        }),
        SerializeOpts::default(),
    )?;
    Ok(String::from_utf8(bytes)?)
}
pub fn image_sources(html: &str) -> Vec<String> {
    fn walk(n: &Handle, images: &mut Vec<String>) {
        if let NodeData::Element { name, attrs, .. } = &n.data
            && &*name.local == "img"
        {
            for attr in attrs.borrow().iter() {
                if &*attr.name.local == "src" {
                    images.push(attr.value.to_string());
                }
            }
        }
        for child in n.children.borrow().iter() {
            walk(child, images);
        }
    }
    let mut images = Vec::new();
    walk(&tree(html).document, &mut images);
    images
}
