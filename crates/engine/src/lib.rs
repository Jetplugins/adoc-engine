//! Native AsciiDoc conversion. No JavaScript, Ruby, or external executable is used.
mod analysis;
mod assets;
mod math;
mod presentation;
mod resolver;
pub use analysis::{Analysis, Symbol, analyze};
mod epub;
mod publication;
use anyhow::{Result, bail};
use asciidoc_html5::{Options, SafeMode};
pub use asciidoc_parser::Document;
use serde::{Deserialize, Serialize};
use std::{collections::BTreeMap, path::PathBuf};

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(default, rename_all = "camelCase")]
pub struct RenderOptions {
    pub base_dir: PathBuf,
    pub input_file: Option<PathBuf>,
    pub overlays: BTreeMap<PathBuf, String>,
    pub include_aliases: BTreeMap<String, PathBuf>,
    pub safe_mode: String,
    pub standalone: bool,
    pub source_locations: bool,
    pub attributes: BTreeMap<String, String>,
    pub diagrams: BTreeMap<String, String>,
}
impl Default for RenderOptions {
    fn default() -> Self {
        Self {
            base_dir: PathBuf::from("."),
            input_file: None,
            overlays: BTreeMap::new(),
            include_aliases: BTreeMap::new(),
            safe_mode: "secure".into(),
            standalone: false,
            source_locations: false,
            attributes: BTreeMap::new(),
            diagrams: BTreeMap::new(),
        }
    }
}
impl RenderOptions {
    pub fn parser_options(&self) -> Result<Options> {
        let safe = match self.safe_mode.as_str() {
            "unsafe" => SafeMode::Unsafe,
            "safe" => SafeMode::Safe,
            "server" => SafeMode::Server,
            "secure" => SafeMode::Secure,
            _ => bail!("unknown safe mode: {}", self.safe_mode),
        };
        let mut o = Options::new()
            .base_dir(&self.base_dir)
            .safe_mode(safe)
            .standalone(self.standalone)
            .source_locations(self.source_locations)
            .catalog_assets(true);
        if let Some(path) = &self.input_file {
            o = o.input_file(path);
        }
        o = o.include_file_handler(resolver::Resolver::new(self.clone())?);
        for (k, v) in &self.attributes {
            let soft = v.ends_with('@') || k.ends_with('@');
            let key = k.trim_end_matches('@');
            let unset = key.starts_with('!') || key.ends_with('!');
            let name = key.trim_start_matches('!').trim_end_matches('!');
            let value = if soft {
                v.strip_suffix('@').unwrap_or(v)
            } else {
                v.as_str()
            };
            o = match (unset, soft) {
                (true, true) => o.unset_default(name),
                (true, false) => o.unset(name),
                (false, true) => o.attribute_default(name, value),
                (false, false) => o.attribute(name, value),
            };
        }
        Ok(o)
    }
}
#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Diagnostic {
    pub file: Option<String>,
    pub line: usize,
    pub message: String,
}
#[derive(Debug, Serialize)]
pub struct SourceMapping {
    pub output_line: usize,
    pub file: Option<String>,
    pub line: usize,
}
#[derive(Debug, Serialize)]
pub struct RenderResult {
    pub html: String,
    pub title: String,
    pub diagnostics: Vec<Diagnostic>,
    pub source_map: Vec<SourceMapping>,
    pub dependencies: Vec<String>,
}
pub fn render(source: &str, options: &RenderOptions) -> Result<RenderResult> {
    let opts = options.parser_options()?;
    let doc = asciidoc_html5::load_with(source, &opts);
    let diagnostics = doc
        .warnings()
        .map(|w| Diagnostic {
            file: doc.origin_of(w.source).file.map(str::to_owned),
            line: doc.origin_of(w.source).line,
            message: w.warning.to_string(),
        })
        .collect();
    let html = presentation::inject(
        asciidoc_html5::convert_document_with(&doc, &opts),
        &options.diagrams,
    )?;
    let html = if math::native_html(&doc) {
        presentation::render_math(html)?
    } else {
        html
    };
    let source_map = if options.source_locations {
        let re = regex::Regex::new(r#"data-source-line="(\d+)""#)?;
        re.captures_iter(&html)
            .filter_map(|c| {
                let output_line = c[1].parse::<usize>().ok()?;
                let origin = doc.source_map().original_file_and_line(output_line)?;
                Some(SourceMapping {
                    output_line,
                    file: origin.0,
                    line: origin.1,
                })
            })
            .collect()
    } else {
        Vec::new()
    };
    let mut dependencies = std::collections::BTreeSet::new();
    for line in 1..=doc.span().data().lines().count() {
        if let Some(origin) = doc.source_map().original_file_and_line(line)
            && let Some(file) = origin.0
        {
            dependencies.insert(file);
        }
    }
    Ok(RenderResult {
        html,
        source_map,
        dependencies: dependencies.into_iter().collect(),
        title: doc.doctitle().unwrap_or("Untitled").into(),
        diagnostics,
    })
}
/// One or more named output artifacts. SVG/PNG use one artifact per page.
pub fn export(
    source: &str,
    format: &str,
    options: &RenderOptions,
) -> Result<Vec<(String, Vec<u8>)>> {
    let mut opts = options.parser_options()?;
    if format == "epub" {
        opts = opts.doctype_default("book").processing_backend("epub3");
        if !options.attributes.contains_key("listing-caption") {
            opts = opts.attribute_default("listing-caption", "Listing");
        }
    }
    let doc = asciidoc_html5::load_with(source, &opts);
    match format {
        "html" => {
            let html = presentation::inject(
                asciidoc_html5::convert_document_with(&doc, &opts),
                &options.diagrams,
            )?;
            let html = if math::native_html(&doc) {
                presentation::render_math(html)?
            } else {
                html
            };
            Ok(vec![("document.html".into(), html.into_bytes())])
        }
        "json" => Ok(vec![(
            "document.json".into(),
            serde_json::to_vec_pretty(&render(source, options)?)?,
        )]),
        "epub" => Ok(vec![(
            "document.epub".into(),
            epub::export(&doc, &opts, options)?,
        )]),
        "pdf" | "svg" | "svg-merged" | "png" => publication::export(&doc, format, options),
        _ => bail!("unsupported format: {format}"),
    }
}
