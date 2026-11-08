//! Small owned publication DOM, built by the HTML5 parser rather than regex repair.
use anyhow::{Result, bail};
use html5ever::tendril::TendrilSink;
use markup5ever_rcdom::{Handle, NodeData, RcDom};
use std::collections::BTreeMap;

#[derive(Clone, Debug)]
pub enum Node {
    Text(String),
    Element(Element),
}

#[derive(Clone, Debug)]
pub struct Element {
    pub tag: String,
    pub attrs: BTreeMap<String, String>,
    pub children: Vec<Node>,
}

impl Element {
    pub fn new(tag: &str) -> Self {
        Self {
            tag: tag.into(),
            attrs: BTreeMap::new(),
            children: Vec::new(),
        }
    }
    pub fn attr(&self, key: &str) -> &str {
        self.attrs.get(key).map(String::as_str).unwrap_or("")
    }
    pub fn set(&mut self, key: &str, value: impl Into<String>) {
        self.attrs.insert(key.into(), value.into());
    }
    pub fn class(&self, name: &str) -> bool {
        self.attr("class").split_whitespace().any(|s| s == name)
    }
    pub fn text(&self) -> String {
        self.children.iter().map(Node::text).collect()
    }
    pub fn walk(&self, f: &mut impl FnMut(&Element)) {
        f(self);
        for node in &self.children {
            if let Node::Element(child) = node {
                child.walk(f);
            }
        }
    }
    pub fn walk_mut(&mut self, f: &mut impl FnMut(&mut Element) -> Result<()>) -> Result<()> {
        f(self)?;
        for node in &mut self.children {
            if let Node::Element(child) = node {
                child.walk_mut(f)?;
            }
        }
        Ok(())
    }
    pub fn html(&self) -> String {
        let mut out = String::new();
        self.emit(&mut out);
        out
    }
    pub fn inner_html(&self) -> String {
        let mut out = String::new();
        for child in &self.children {
            child.emit(&mut out);
        }
        out
    }
    fn emit(&self, out: &mut String) {
        out.push('<');
        out.push_str(&self.tag);
        for (key, value) in &self.attrs {
            out.push(' ');
            out.push_str(key);
            out.push_str("=\"");
            out.push_str(&xml(value));
            out.push('"');
        }
        if matches!(
            self.tag.as_str(),
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
                | "track"
        ) {
            out.push_str(" />");
        } else {
            out.push('>');
            for child in &self.children {
                child.emit(out);
            }
            out.push_str("</");
            out.push_str(&self.tag);
            out.push('>');
        }
    }
}
impl Node {
    pub fn text(&self) -> String {
        match self {
            Self::Text(t) => t.clone(),
            Self::Element(e) => e.text(),
        }
    }
    pub fn emit(&self, out: &mut String) {
        match self {
            Self::Text(t) => out.push_str(&xml(t)),
            Self::Element(e) => e.emit(out),
        }
    }
}
pub fn xml(text: &str) -> String {
    text.replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
        .replace('"', "&quot;")
}
pub fn parse(html: &str) -> Result<Element> {
    fn convert(handle: &Handle) -> Vec<Node> {
        match &handle.data {
            NodeData::Text { contents } => vec![Node::Text(contents.borrow().to_string())],
            NodeData::Element { name, attrs, .. } => {
                let mut element = Element::new(&name.local);
                for a in attrs.borrow().iter() {
                    let key = a
                        .name
                        .prefix
                        .as_ref()
                        .filter(|p| !p.is_empty())
                        .map(|p| format!("{p}:{}", a.name.local))
                        .unwrap_or_else(|| a.name.local.to_string());
                    element.set(&key, a.value.to_string());
                }
                if element.tag == "svg" {
                    element.set("xmlns", "http://www.w3.org/2000/svg");
                }
                if element.tag == "math" {
                    element.set("xmlns", "http://www.w3.org/1998/Math/MathML");
                }
                element.children = handle.children.borrow().iter().flat_map(convert).collect();
                vec![Node::Element(element)]
            }
            _ => handle.children.borrow().iter().flat_map(convert).collect(),
        }
    }
    let dom = html5ever::parse_document(RcDom::default(), Default::default()).one(html);
    let nodes = convert(&dom.document);
    fn body(nodes: Vec<Node>) -> Option<Element> {
        for node in nodes {
            if let Node::Element(e) = node {
                if e.tag == "body" {
                    return Some(e);
                }
                if let Some(e) = body(e.children) {
                    return Some(e);
                }
            }
        }
        None
    }
    body(nodes).ok_or_else(|| anyhow::anyhow!("missing publication body"))
}

pub fn clean(root: &mut Element, in_svg: bool) -> Result<()> {
    if root.tag == "code"
        && !root.attr("data-lang").is_empty()
        && !root
            .attr("class")
            .split_whitespace()
            .any(|s| s.starts_with("language-"))
    {
        root.set(
            "class",
            format!("{} language-{}", root.attr("class"), root.attr("data-lang"))
                .trim()
                .to_owned(),
        );
    }
    if root.children.iter().any(
        |n| matches!(n, Node::Element(e) if matches!(e.tag.as_str(), "iframe"|"object"|"embed")),
    ) {
        bail!("EPUB cannot embed browser players; use a local media file or an external link");
    }
    // Preserve SVG's intrinsic presentation, but remove preview scripts and controls.
    root.children.retain(|n| !matches!(n, Node::Element(e) if matches!(e.tag.as_str(), "script"|"iframe"|"object"|"embed"|"form"|"input"|"button"|"select"|"textarea"|"base"|"link") || (!in_svg && e.tag == "style")));
    root.attrs.retain(|key, _| {
        !key.to_ascii_lowercase().starts_with("on")
            && !key.starts_with("data-")
            && (in_svg || key != "style")
    });
    for key in ["href", "src", "xlink:href"] {
        if root
            .attr(key)
            .trim_start()
            .to_ascii_lowercase()
            .starts_with("javascript:")
        {
            bail!("JavaScript links are not supported in EPUB");
        }
    }
    if root.tag == "track" {
        bail!("EPUB subtitle tracks are not supported; use local MP3, M4A or MP4 media");
    }
    for node in &mut root.children {
        if let Node::Element(e) = node {
            clean(e, in_svg || e.tag == "svg")?;
        }
    }
    Ok(())
}

/// Section wrappers are layout containers, and must not prevent heading-based chunking.
pub fn flatten_sections(nodes: Vec<Node>) -> Vec<Node> {
    let mut result = Vec::new();
    for node in nodes {
        if let Node::Element(mut e) = node {
            let wrapper = e.tag == "div"
                && (e.class("sectionbody")
                    || (1..=5).any(|n| e.class(&format!("sect{n}")))
                    || e.attr("id") == "preamble");
            if wrapper {
                // IDs on the wrapper remain addressable after it is unwrapped.
                if !e.attr("id").is_empty() && e.attr("id") != "preamble" {
                    let mut anchor = Element::new("span");
                    anchor.set("id", e.attr("id"));
                    result.push(Node::Element(anchor));
                }
                result.extend(flatten_sections(e.children));
            } else {
                e.children = e
                    .children
                    .into_iter()
                    .map(|n| match n {
                        Node::Element(mut child) => {
                            child.children = flatten_sections(child.children);
                            Node::Element(child)
                        }
                        other => other,
                    })
                    .collect();
                result.push(Node::Element(e));
            }
        } else {
            result.push(node);
        }
    }
    result
}
