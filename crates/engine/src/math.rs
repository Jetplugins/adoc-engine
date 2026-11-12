//! Native STEM conversion shared by HTML and EPUB.
use anyhow::{Context, Result, ensure};
use asciidoc_html5::Document;
use asciidoc_parser::document::InterpretedValue;

pub(crate) enum Piece<'a> {
    Text(&'a str),
    Math(String),
}

pub(crate) fn native_html(doc: &Document<'_>) -> bool {
    matches!(doc.attribute_value("stem-renderer"), InterpretedValue::Value(value) if value == "native")
}

/// Recognize only the delimiters emitted by the AsciiDoc HTML backend.
/// Code, scripts and existing MathML are excluded by the DOM walkers.
pub(crate) fn pieces(mut text: &str, display: bool) -> Result<Vec<Piece<'_>>> {
    let mut out = Vec::new();
    while let Some((at, open, close, latex, block)) = [
        ("\\$", "\\$", false, false),
        ("\\(", "\\)", true, false),
        ("\\[", "\\]", true, true),
    ]
    .into_iter()
    .filter_map(|(open, close, latex, block)| {
        text.find(open).map(|at| (at, open, close, latex, block))
    })
    .min_by_key(|item| item.0)
    {
        let body = &text[at + open.len()..];
        let Some(end) = body.find(close) else {
            out.push(Piece::Text(&text[..at + open.len()]));
            text = body;
            continue;
        };
        out.push(Piece::Text(&text[..at]));
        out.push(Piece::Math(mathml(&body[..end], latex, display || block)?));
        text = &body[end + close.len()..];
    }
    out.push(Piece::Text(text));
    Ok(out)
}

fn mathml(source: &str, latex: bool, display: bool) -> Result<String> {
    ensure!(source.len() <= 32 * 1024, "STEM expression exceeds 32 KiB");
    let markup = if latex {
        let config = math_core::MathCoreConfig {
            xml_namespace: true,
            ..Default::default()
        };
        let converter = math_core::LatexToMathML::new(config, vec![])
            .map_err(|error| anyhow::anyhow!("LaTeX renderer configuration: {error:?}"))?;
        converter
            .convert_with_local_state(
                source,
                if display {
                    math_core::MathDisplay::Block
                } else {
                    math_core::MathDisplay::Inline
                },
            )
            .map_err(|error| anyhow::anyhow!("Cannot render LaTeX equation: {error}"))?
            .mathml
    } else {
        mathemascii::render_mathml(mathemascii::parse(source))
    };
    let rest = markup
        .strip_prefix("<math")
        .context("Native renderer returned no MathML")?;
    let alt = source
        .replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
        .replace('"', "&quot;");
    let mode = if latex {
        String::new()
    } else {
        format!(" display=\"{}\"", if display { "block" } else { "inline" })
    };
    Ok(format!("<math alttext=\"{alt}\"{mode}{rest}"))
}
