use super::dom::{parse, xml};
use anyhow::{Context, Result, bail};
use asciidoc_html5::Document;
use asciidoc_parser::document::InterpretedValue;
use sha2::{Digest, Sha256};
use std::collections::BTreeMap;

pub struct Metadata {
    pub title: String,
    pub language: String,
    pub identifier: String,
    pub authors: Vec<String>,
    pub attributes: BTreeMap<String, String>,
}
impl Metadata {
    pub fn new(doc: &Document<'_>, options: &crate::RenderOptions) -> Self {
        let mut attributes = BTreeMap::new();
        for key in [
            "doctype",
            "imagesdir",
            "lang",
            "uuid",
            "author",
            "authors",
            "producer",
            "publisher",
            "creator",
            "description",
            "copyright",
            "rights",
            "source",
            "keywords",
            "revdate",
            "series-name",
            "series-volume",
            "series-id",
            "front-cover-image",
            "epub-chapter-level",
            "epub-properties",
            "epub3-frontmatterdir",
            "epub3-stylesdir",
            "toc",
            "toc-title",
            "toclevels",
            "outlinelevels",
            "username",
            "source-highlighter",
            "epub-modified",
            "listing-caption",
            "epub-fonts",
            "scripts",
        ] {
            match doc.attribute_value(key) {
                InterpretedValue::Value(value) => {
                    attributes.insert(key.into(), value);
                }
                InterpretedValue::Set => {
                    attributes.insert(key.into(), String::new());
                }
                InterpretedValue::Unset => {}
            }
        }
        let title = doc
            .doctitle_sanitized()
            .filter(|s| !s.trim().is_empty())
            .or_else(|| {
                options
                    .input_file
                    .as_ref()
                    .and_then(|p| p.file_stem())
                    .map(|s| s.to_string_lossy().into_owned())
            })
            .unwrap_or_else(|| "Untitled".into());
        let language = attributes
            .get("lang")
            .filter(|s| !s.is_empty())
            .cloned()
            .unwrap_or_else(|| "en".into());
        let identifier = attributes
            .get("uuid")
            .filter(|s| !s.trim().is_empty())
            .cloned()
            .unwrap_or_else(|| {
                let digest = Sha256::digest(title.as_bytes());
                let hex: String = digest.iter().map(|byte| format!("{byte:02x}")).collect();
                format!("urn:sha256:{hex}")
            });
        let authors = doc
            .authors()
            .iter()
            .map(|a| a.raw_name().to_owned())
            .collect();
        Self {
            title,
            language,
            identifier,
            authors,
            attributes,
        }
    }
    pub fn attr(&self, name: &str) -> &str {
        self.attributes.get(name).map(String::as_str).unwrap_or("")
    }
    pub fn is_book(&self) -> bool {
        self.attr("doctype") != "article" && self.attr("doctype") != "manpage"
    }
    pub fn depth(&self, key: &str, default: usize) -> usize {
        self.attr(key)
            .parse::<usize>()
            .ok()
            .filter(|n| (1..=6).contains(n))
            .unwrap_or(default)
    }
    pub fn visible_toc(&self) -> bool {
        self.attributes.contains_key("toc")
            && !matches!(self.attr("toc"), "false" | "none" | "no" | "0")
    }
    pub fn package(&self) -> Result<String> {
        let mut out = format!(
            "<dc:identifier id=\"book-id\">{}</dc:identifier><dc:title>{}</dc:title><dc:language>{}</dc:language>",
            xml(&self.identifier),
            xml(&self.title),
            xml(&self.language)
        );
        // Keep creator metadata useful to readers while also honoring the reference's contributor mapping.
        for (i, author) in self.authors.iter().enumerate() {
            out.push_str(&format!("<dc:creator id=\"author-{i}\">{}</dc:creator><meta refines=\"#author-{i}\" property=\"role\" scheme=\"marc:relators\">aut</meta><dc:contributor>{}</dc:contributor>", xml(author), xml(author)));
        }
        let creator = if self.attr("creator").is_empty() {
            "adoc-engine"
        } else {
            self.attr("creator")
        };
        out.push_str(&format!("<dc:creator id=\"manufacturer\">{}</dc:creator><meta refines=\"#manufacturer\" property=\"role\" scheme=\"marc:relators\">mfr</meta>", xml(creator)));
        for (element, names) in [
            ("publisher", ["producer", "publisher"]),
            ("rights", ["copyright", "rights"]),
            ("description", ["description", "description"]),
            ("source", ["source", "source"]),
            ("date", ["revdate", "revdate"]),
        ] {
            if let Some(value) = names.iter().map(|n| self.attr(n)).find(|s| !s.is_empty()) {
                out.push_str(&format!(
                    "<dc:{element}>{}</dc:{element}>",
                    xml(&plain(value))
                ));
            }
        }
        for subject in csv(self.attr("keywords"))? {
            out.push_str(&format!("<dc:subject>{}</dc:subject>", xml(&subject)));
        }
        if !self.attr("series-name").is_empty() {
            out.push_str(&format!("<meta property=\"belongs-to-collection\" id=\"series\">{}</meta><meta refines=\"#series\" property=\"collection-type\">series</meta>", xml(self.attr("series-name"))));
            if !self.attr("series-volume").is_empty() {
                let volume: f64 = self
                    .attr("series-volume")
                    .parse()
                    .context("series-volume must be a nonnegative number")?;
                if !volume.is_finite() || volume < 0.0 {
                    bail!("series-volume must be a nonnegative number");
                }
                out.push_str(&format!(
                    "<meta refines=\"#series\" property=\"group-position\">{}</meta>",
                    xml(self.attr("series-volume"))
                ));
            }
            if !self.attr("series-id").is_empty() {
                out.push_str(&format!(
                    "<meta refines=\"#series\" property=\"dcterms:identifier\">{}</meta>",
                    xml(self.attr("series-id"))
                ));
            }
        }
        let modified = if !self.attr("epub-modified").is_empty() {
            let value = self.attr("epub-modified");
            time::OffsetDateTime::parse(value, &time::format_description::well_known::Rfc3339)
                .context("epub-modified must be a UTC ISO timestamp")?;
            if !value.ends_with('Z') {
                bail!("epub-modified must end in Z");
            }
            value.to_owned()
        } else {
            // SOURCE_DATE_EPOCH enables byte-reproducible builds, without inventing a publication date.
            let timestamp = std::env::var("SOURCE_DATE_EPOCH")
                .ok()
                .map(|s| s.parse::<i64>())
                .transpose()
                .context("invalid SOURCE_DATE_EPOCH")?;
            let now = match timestamp {
                Some(t) => time::OffsetDateTime::from_unix_timestamp(t)?,
                None => time::OffsetDateTime::now_utc(),
            };
            now.replace_nanosecond(0)?
                .format(&time::format_description::well_known::Rfc3339)?
        };
        out.push_str(&format!("<meta property=\"dcterms:modified\">{}</meta><meta property=\"rendition:layout\">reflowable</meta>", xml(&modified)));
        Ok(out)
    }
}
fn plain(value: &str) -> String {
    parse(&format!("<p>{value}</p>"))
        .map(|e| e.text())
        .unwrap_or_else(|_| value.to_owned())
}

pub fn csv(value: &str) -> Result<Vec<String>> {
    let mut entries = Vec::new();
    let mut current = String::new();
    let mut quoted = false;
    let mut chars = value.chars().peekable();
    while let Some(c) = chars.next() {
        match c {
            '"' if quoted && chars.peek() == Some(&'"') => {
                current.push('"');
                chars.next();
            }
            '"' => quoted = !quoted,
            ',' if !quoted => {
                if !current.trim().is_empty() {
                    entries.push(current.trim().to_owned());
                }
                current.clear();
            }
            _ => current.push(c),
        }
    }
    if quoted {
        bail!("keywords contains an unterminated quoted CSV value");
    }
    if !current.trim().is_empty() {
        entries.push(current.trim().to_owned());
    }
    Ok(entries)
}
