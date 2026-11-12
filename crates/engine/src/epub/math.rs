use super::dom::{Element, Node, parse};
use crate::math::{Piece, pieces};
use anyhow::Result;

/// Convert both AsciiMath and LaTeX into static, accessible MathML.
pub fn render(root: &mut Element, display: bool) -> Result<()> {
    if matches!(
        root.tag.as_str(),
        "pre" | "code" | "kbd" | "samp" | "math" | "script" | "style" | "textarea" | "svg"
    ) || root.class("nostem")
    {
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
                for piece in pieces(&text, display)? {
                    match piece {
                        Piece::Text(text) => children.push(Node::Text(text.into())),
                        Piece::Math(markup) => {
                            let mut fragment = parse(&markup)?;
                            for node in &mut fragment.children {
                                if let Node::Element(math) = node {
                                    math.set(
                                        "display",
                                        if display || math.attr("display") == "block" {
                                            "block"
                                        } else {
                                            "inline"
                                        },
                                    );
                                    // MathML Core permits scriptlevel on mtable; EPUB's
                                    // MathML 3 profile requires the equivalent mstyle wrapper.
                                    math.walk_mut(&mut |element| {
                                        if element.tag == "mtable"
                                            && let Some(level) = element.attrs.remove("scriptlevel")
                                        {
                                            let table =
                                                std::mem::replace(element, Element::new("mstyle"));
                                            element.set("scriptlevel", level);
                                            element.children.push(Node::Element(table));
                                        }
                                        Ok(())
                                    })?;
                                }
                            }
                            children.extend(fragment.children);
                        }
                    }
                }
            }
        }
    }
    root.children = children;
    Ok(())
}
