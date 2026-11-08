use super::dom::{Element, Node};
use anyhow::{Context, Result, bail, ensure};
use base64::{Engine, engine::general_purpose::STANDARD};
use percent_encoding::percent_decode_str;
use regex::Regex;
use std::{
    collections::BTreeMap,
    path::{Path, PathBuf},
    sync::LazyLock,
};

pub struct Asset {
    pub href: String,
    pub media: &'static str,
    pub bytes: Vec<u8>,
}
pub struct Assets<'a> {
    pub entries: Vec<Asset>,
    keys: BTreeMap<String, usize>,
    options: &'a crate::RenderOptions,
}
impl<'a> Assets<'a> {
    pub fn new(options: &'a crate::RenderOptions) -> Self {
        Self {
            entries: Vec::new(),
            keys: BTreeMap::new(),
            options,
        }
    }
    pub fn local(&self, source: &str, directory: &Path) -> Result<PathBuf> {
        ensure!(
            !source.starts_with("//") && !SCHEME.is_match(source),
            "EPUB requires a local asset, not {source}"
        );
        let decoded = percent_decode_str(source)
            .decode_utf8()
            .context("asset URL must be UTF-8")?;
        let path = directory
            .canonicalize()
            .with_context(|| format!("EPUB asset directory {}", directory.display()))?
            .join(decoded.as_ref());
        crate::assets::local_path(&path.to_string_lossy(), self.options)
    }
    pub fn read(&self, source: &str, directory: &Path) -> Result<(PathBuf, Vec<u8>)> {
        let path = self.local(source, directory)?;
        let bytes = std::fs::read(&path)
            .with_context(|| format!("reading EPUB asset {}", path.display()))?;
        Ok((path, bytes))
    }
    pub fn add(&mut self, source: &str, directory: &Path) -> Result<String> {
        let (key, bytes, media, extension) = if let Some(data) = source.strip_prefix("data:") {
            let (header, data) = data.split_once(',').context("malformed EPUB data image")?;
            ensure!(
                data.len() <= 45 * 1024 * 1024,
                "EPUB data asset exceeds 32 MiB"
            );
            let mime = header
                .strip_suffix(";base64")
                .context("EPUB data images must use base64")?;
            let extension = match mime {
                "image/png" => "png",
                "image/jpeg" => "jpg",
                "image/gif" => "gif",
                "image/svg+xml" => "svg",
                "font/ttf" => "ttf",
                "font/otf" => "otf",
                "font/woff" => "woff",
                "font/woff2" => "woff2",
                _ => bail!("unsupported EPUB data asset: {mime}"),
            };
            (
                source.to_owned(),
                STANDARD.decode(data).context("invalid base64 EPUB asset")?,
                media_type(extension)?,
                extension.to_owned(),
            )
        } else {
            let (path, bytes) = self.read(source, directory)?;
            let extension = path
                .extension()
                .and_then(|s| s.to_str())
                .unwrap_or("")
                .to_ascii_lowercase();
            (
                path.to_string_lossy().into_owned(),
                bytes,
                media_type(&extension)?,
                extension,
            )
        };
        if let Some(index) = self.keys.get(&key) {
            return Ok(self.entries[*index].href.clone());
        }
        ensure!(
            !bytes.is_empty() && bytes.len() <= 32 * 1024 * 1024,
            "EPUB asset must contain 1 byte to 32 MiB: {source}"
        );
        if media == "image/svg+xml" {
            validate_svg(&bytes)?;
        }
        let href = format!("assets/resource-{}.{}", self.entries.len() + 1, extension);
        self.keys.insert(key, self.entries.len());
        self.entries.push(Asset {
            href: href.clone(),
            media,
            bytes,
        });
        Ok(href)
    }
    pub fn rewrite(&mut self, root: &mut Element, directory: &Path) -> Result<()> {
        for node in &mut root.children {
            if let Node::Element(element) = node {
                if element.tag == "svg" {
                    element.set("xmlns:xlink", "http://www.w3.org/1999/xlink");
                    let source = format!(
                        "data:image/svg+xml;base64,{}",
                        STANDARD.encode(element.html())
                    );
                    let href = self.add(&source, directory)?;
                    let mut image = Element::new("img");
                    image.set("src", href);
                    image.set("alt", "Diagram");
                    *element = image;
                } else {
                    if element.tag == "img" {
                        let source = element.attr("src").to_owned();
                        ensure!(!source.is_empty(), "EPUB image has no source");
                        let href = self.add(&source, directory)?;
                        ensure!(
                            self.entries
                                .iter()
                                .any(|a| a.href == href && a.media.starts_with("image/")),
                            "EPUB image requires an image asset"
                        );
                        element.set("src", href);
                        element.attrs.remove("srcset");
                        element.attrs.entry("alt".into()).or_default();
                    }
                    if matches!(element.tag.as_str(), "audio" | "video" | "source")
                        && !element.attr("src").is_empty()
                    {
                        let href = self.add(element.attr("src"), directory)?;
                        ensure!(
                            self.entries.iter().any(|a| a.href == href
                                && (a.media.starts_with("audio/")
                                    || a.media.starts_with("video/"))),
                            "EPUB media requires MP3, M4A or MP4"
                        );
                        element.set("src", href);
                    }
                    if element.tag == "video" && !element.attr("poster").is_empty() {
                        let href = self.add(element.attr("poster"), directory)?;
                        element.set("poster", href);
                    }
                    self.rewrite(element, directory)?;
                }
            }
        }
        Ok(())
    }
    pub fn stylesheet(&mut self, css: &str, directory: &Path) -> Result<String> {
        self.stylesheet_inner(css, directory, &mut std::collections::BTreeSet::new())
    }
    fn stylesheet_inner(
        &mut self,
        css: &str,
        directory: &Path,
        active: &mut std::collections::BTreeSet<PathBuf>,
    ) -> Result<String> {
        ensure!(
            active.len() < 16,
            "EPUB stylesheet imports exceed 16 levels"
        );
        let mut expanded = String::new();
        let mut end = 0;
        let mut imports = Vec::new();
        for capture in CSS_IMPORT.captures_iter(css) {
            let found = capture.get(0).unwrap();
            expanded.push_str(&css[end..found.start()]);
            let source = capture
                .get(1)
                .or_else(|| capture.get(2))
                .or_else(|| capture.get(3))
                .unwrap()
                .as_str()
                .trim()
                .trim_matches(['\'', '"']);
            let (path, bytes) = self.read(source, directory)?;
            ensure!(
                active.insert(path.clone()),
                "cyclic EPUB stylesheet import: {}",
                path.display()
            );
            let contents = self.stylesheet_inner(
                std::str::from_utf8(&bytes)?,
                path.parent().context("stylesheet requires a directory")?,
                active,
            )?;
            active.remove(&path);
            let media = capture[4].trim();
            imports.push(if media.is_empty() {
                contents
            } else {
                format!("@media {media} {{ {contents} }}")
            });
            expanded.push_str(&format!("\0ADOCIMPORT{}\0", imports.len() - 1));
            end = found.end();
        }
        expanded.push_str(&css[end..]);
        let mut out = String::new();
        let mut end = 0;
        for capture in CSS_URL.captures_iter(&expanded) {
            let found = capture.get(0).unwrap();
            out.push_str(&expanded[end..found.start()]);
            let target = capture[1].trim().trim_matches(['\'', '"']);
            if target.starts_with('#') {
                out.push_str(found.as_str());
            } else {
                out.push_str(&format!("url(\"{}\")", self.add(target, directory)?));
            }
            end = found.end();
        }
        out.push_str(&expanded[end..]);
        for (i, imported) in imports.into_iter().enumerate() {
            out = out.replace(&format!("\0ADOCIMPORT{i}\0"), &imported);
        }
        Ok(out)
    }
}
fn media_type(extension: &str) -> Result<&'static str> {
    match extension {
        "png" => Ok("image/png"),
        "jpg" | "jpeg" => Ok("image/jpeg"),
        "gif" => Ok("image/gif"),
        "svg" => Ok("image/svg+xml"),
        "ttf" => Ok("font/ttf"),
        "otf" => Ok("font/otf"),
        "woff" => Ok("font/woff"),
        "woff2" => Ok("font/woff2"),
        "mp3" => Ok("audio/mpeg"),
        "m4a" => Ok("audio/mp4"),
        "mp4" => Ok("video/mp4"),
        _ => bail!("unsupported EPUB asset type: {extension}"),
    }
}
fn validate_svg(bytes: &[u8]) -> Result<()> {
    let text = std::str::from_utf8(bytes).context("SVG must be UTF-8")?;
    let svg = roxmltree::Document::parse(text).context("invalid SVG image")?;
    ensure!(
        svg.root_element().tag_name().name() == "svg",
        "EPUB SVG asset requires an SVG root"
    );
    for node in svg.descendants().filter(|n| n.is_element()) {
        ensure!(
            !matches!(
                node.tag_name().name(),
                "script" | "foreignObject" | "iframe" | "object" | "embed"
            ),
            "EPUB SVG assets must be static"
        );
        for attr in node.attributes() {
            ensure!(
                !attr.name().to_ascii_lowercase().starts_with("on"),
                "SVG event handlers are not supported"
            );
            if matches!(attr.name(), "href" | "src") {
                ensure!(
                    attr.value().starts_with('#'),
                    "SVG resources must be self-contained"
                );
            }
            check_css(attr.value())?;
        }
        if node.tag_name().name() == "style" {
            check_css(node.text().unwrap_or(""))?;
        }
    }
    Ok(())
}
fn check_css(value: &str) -> Result<()> {
    ensure!(
        !value.to_ascii_lowercase().contains("@import"),
        "SVG styles must be self-contained"
    );
    for url in CSS_URL.captures_iter(value) {
        ensure!(
            url[1].trim().trim_matches(['\'', '"']).starts_with('#'),
            "SVG styles must not load external assets"
        );
    }
    Ok(())
}
static SCHEME: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r"^[a-zA-Z][a-zA-Z0-9+.-]*:").unwrap());
static CSS_URL: LazyLock<Regex> = LazyLock::new(|| Regex::new(r"(?i)url\(\s*(.*?)\s*\)").unwrap());
static CSS_IMPORT: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(r#"(?i)@import\s+(?:url\(\s*([^)]*)\)|"([^"]*)"|'([^']*)')\s*([^;]*);"#).unwrap()
});
