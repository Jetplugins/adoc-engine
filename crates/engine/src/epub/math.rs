use super::dom::{Element, Node, parse};
use anyhow::{Context, Result, ensure};

/// Convert the HTML backend's AsciiMath delimiters into static, accessible MathML.
pub fn render(root: &mut Element, display: bool) -> Result<()> {
    if matches!(root.tag.as_str(), "pre" | "code" | "kbd" | "samp" | "math") {
        return Ok(());
    }
    let display = display || root.class("stemblock");
    let mut children = Vec::new();
    for node in std::mem::take(&mut root.children) {
        match node {
            Node::Element(mut e) => {
                render(&mut e, display)?;
                children.push(Node::Element(e));
            }
            Node::Text(text) => {
                let mut rest = text.as_str();
                while let Some(start) = rest.find("\\$") {
                    let Some(end) = rest[start + 2..].find("\\$") else {
                        break;
                    };
                    children.push(Node::Text(rest[..start].into()));
                    let source = &rest[start + 2..start + 2 + end];
                    ensure!(
                        source.len() <= 32 * 1024,
                        "EPUB math expression exceeds 32 KiB"
                    );
                    let markup = mathemascii::render_mathml(mathemascii::parse(source));
                    let mut fragment = parse(&markup)?;
                    let mut math = fragment
                        .children
                        .drain(..)
                        .find_map(|n| match n {
                            Node::Element(e) if e.tag == "math" => Some(e),
                            _ => None,
                        })
                        .context("AsciiMath renderer returned no MathML")?;
                    math.set("display", if display { "block" } else { "inline" });
                    math.set("alttext", source);
                    children.push(Node::Element(math));
                    rest = &rest[start + 2 + end + 2..];
                }
                children.push(Node::Text(rest.to_owned()));
            }
        }
    }
    root.children = children;
    Ok(())
}
