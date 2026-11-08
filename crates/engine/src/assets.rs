use crate::RenderOptions;
use anyhow::{Context, Result, bail};
use std::path::{Path, PathBuf};
pub fn local_path(target: &str, options: &RenderOptions) -> Result<PathBuf> {
    if options.safe_mode == "secure" {
        bail!("secure mode prevents embedding local assets; use safe mode for publication assets");
    }
    if target.contains("://") || target.starts_with("data:") {
        bail!("remote/data assets are not supported: {target}");
    }
    let base = options.base_dir.canonicalize()?;
    let path = base
        .join(Path::new(target))
        .canonicalize()
        .with_context(|| format!("reading asset {target}"))?;
    if options.safe_mode != "unsafe" && !path.starts_with(&base) {
        bail!("asset escapes base directory: {target}");
    }
    if std::fs::metadata(&path)?.len() > 32 * 1024 * 1024 {
        bail!("asset exceeds 32 MiB: {target}");
    }
    Ok(path)
}
pub fn media_type(path: &Path) -> Result<&'static str> {
    match path
        .extension()
        .and_then(|s| s.to_str())
        .unwrap_or("")
        .to_lowercase()
        .as_str()
    {
        "png" => Ok("image/png"),
        "jpg" | "jpeg" => Ok("image/jpeg"),
        "gif" => Ok("image/gif"),
        "svg" => Ok("image/svg+xml"),
        _ => bail!("unsupported image type: {}", path.display()),
    }
}
