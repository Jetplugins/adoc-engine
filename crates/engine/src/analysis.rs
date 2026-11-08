use crate::{RenderOptions, RenderResult, render};
use anyhow::Result;
use asciidoc_parser::HasSpan;
use asciidoc_parser::blocks::{Block, FindBlocks, IsBlock};
use serde::Serialize;
#[derive(Debug, Serialize)]
pub struct Symbol {
    pub name: String,
    pub detail: String,
    pub file: Option<String>,
    pub line: usize,
    pub kind: String,
}
#[derive(Debug, Serialize)]
pub struct Analysis {
    #[serde(flatten)]
    pub rendered: RenderResult,
    pub symbols: Vec<Symbol>,
}
pub fn analyze(source: &str, options: &RenderOptions) -> Result<Analysis> {
    let opts = options.parser_options()?;
    let doc = asciidoc_html5::load_with(source, &opts);
    let mut symbols = Vec::new();
    for block in doc.descendant_blocks() {
        if let Some(id) = block.id() {
            let origin = doc.origin_of(block.span());
            let detail = doc
                .catalog()
                .get_ref(id)
                .and_then(|e| e.reftext.clone())
                .unwrap_or_else(|| id.into());
            symbols.push(Symbol {
                name: id.into(),
                detail,
                file: origin.file.map(str::to_owned),
                line: origin.line.saturating_sub(1),
                kind: if matches!(block, Block::Section(_)) {
                    "section"
                } else {
                    "anchor"
                }
                .into(),
            });
        }
    }
    // Attribute definitions are collected only from parser attribute nodes, not arbitrary source lines.
    let mut raw_ranges = Vec::new();
    for block in doc.descendant_blocks() {
        if matches!(block, Block::RawDelimited(_)) {
            let span = block.span();
            raw_ranges.push((span.line(), span.line() + span.data().lines().count()));
        }
    }
    for (n, line) in doc.span().data().lines().enumerate() {
        if raw_ranges.iter().any(|(a, b)| n + 1 >= *a && n < *b) {
            continue;
        }
        if let Some(rest) = line.strip_prefix(':')
            && let Some((name, value)) = rest.split_once(':')
        {
            symbols.push(Symbol {
                name: name.into(),
                detail: value.trim().into(),
                file: doc
                    .source_map()
                    .original_file_and_line(n + 1)
                    .and_then(|s| s.0),
                line: doc
                    .source_map()
                    .original_file_and_line(n + 1)
                    .map_or(n, |s| s.1.saturating_sub(1)),
                kind: "attribute".into(),
            });
        }
    }
    Ok(Analysis {
        rendered: render(source, options)?,
        symbols,
    })
}
