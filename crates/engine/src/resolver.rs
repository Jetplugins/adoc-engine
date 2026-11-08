use crate::RenderOptions;
use anyhow::Result;
use asciidoc_parser::{
    Parser,
    attributes::Attrlist,
    parser::{IncludeContent, IncludeFileHandler, IncludeResolution},
};
use std::path::{Path, PathBuf};
#[derive(Debug)]
pub struct Resolver {
    options: RenderOptions,
    root: PathBuf,
}
impl Resolver {
    pub fn new(mut options: RenderOptions) -> Result<Self> {
        let root = options.base_dir.canonicalize()?;
        options.overlays = options
            .overlays
            .into_iter()
            .map(|(p, s)| {
                let normalized = p.canonicalize().unwrap_or_else(|_| {
                    p.parent()
                        .and_then(|d| d.canonicalize().ok())
                        .map(|d| d.join(p.file_name().unwrap_or_default()))
                        .unwrap_or(p)
                });
                (normalized, s)
            })
            .collect();
        Ok(Self { options, root })
    }
}
impl IncludeFileHandler for Resolver {
    fn resolve_target<'s>(
        &self,
        source: Option<&str>,
        target: &str,
        _attrs: &Attrlist<'s>,
        _parser: &Parser,
    ) -> IncludeResolution {
        if target.contains("://") {
            return IncludeResolution::NotReadable;
        }
        let source_path = source.map(|s| {
            let logical = Path::new(s);
            let alias = self.options.include_aliases.get(s).or_else(|| {
                self.options
                    .include_aliases
                    .iter()
                    .find(|(key, _)| !key.contains("::") && logical.ends_with(Path::new(key)))
                    .map(|(_, path)| path)
            });
            alias.cloned().unwrap_or_else(|| {
                if logical.is_absolute() {
                    logical.to_owned()
                } else {
                    self.root.join(logical)
                }
            })
        });
        let start = source_path
            .as_deref()
            .and_then(Path::parent)
            .unwrap_or(&self.root);
        let scoped = source_path
            .as_ref()
            .map(|p| format!("{}::{target}", p.display()));
        let path = scoped
            .as_ref()
            .and_then(|s| self.options.include_aliases.get(s))
            .or_else(|| self.options.include_aliases.get(target))
            .cloned()
            .unwrap_or_else(|| start.join(target));
        let normalized = path.components().fold(PathBuf::new(), |mut p, c| {
            if c == std::path::Component::ParentDir {
                p.pop();
            } else if c != std::path::Component::CurDir {
                p.push(c);
            }
            p
        });
        let real = normalized.canonicalize().unwrap_or_else(|_| {
            normalized
                .parent()
                .and_then(|d| d.canonicalize().ok())
                .map(|d| d.join(normalized.file_name().unwrap_or_default()))
                .unwrap_or(normalized.clone())
        });
        if self.options.safe_mode != "unsafe" && !real.starts_with(&self.root) {
            return IncludeResolution::NotReadable;
        }
        if let Some(text) = self
            .options
            .overlays
            .get(&real)
            .or_else(|| self.options.overlays.get(&normalized))
        {
            return if text.len() <= 16 * 1024 * 1024 {
                IncludeResolution::Found(IncludeContent::new(text.clone()))
            } else {
                IncludeResolution::NotReadable
            };
        }
        let bytes = match std::fs::metadata(&real) {
            Ok(m) if m.len() <= 16 * 1024 * 1024 => std::fs::read(&real),
            _ => return IncludeResolution::NotReadable,
        };
        match bytes {
            Ok(b) => match String::from_utf8(b) {
                Ok(s) => IncludeResolution::Found(IncludeContent::new(s)),
                Err(_) => IncludeResolution::NotDecodable,
            },
            Err(e) if e.kind() == std::io::ErrorKind::NotFound => IncludeResolution::NotFound,
            Err(_) => IncludeResolution::NotReadable,
        }
    }
}
