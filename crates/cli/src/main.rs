mod lsp;
use adoc_engine::{RenderOptions, export, render};
use anyhow::{Result, bail};
use clap::{Parser, Subcommand};
use std::{
    io::{self, BufRead, Read, Write},
    path::PathBuf,
    time::Instant,
};
#[derive(Parser)]
#[command(version, about = "Native AsciiDoc renderer and language server")]
struct Args {
    #[command(subcommand)]
    command: Command,
}
#[derive(Subcommand)]
enum Command {
    Render {
        input: Option<PathBuf>,
        #[arg(short, long)]
        output: Option<PathBuf>,
        #[arg(short, long, default_value = "html")]
        format: String,
        #[arg(long)]
        standalone: bool,
        #[arg(long)]
        base_dir: Option<PathBuf>,
        #[arg(short = 'a', long = "attribute")]
        attributes: Vec<String>,
        #[arg(long, default_value = "secure")]
        safe_mode: String,
    },
    Check {
        input: PathBuf,
    },
    Lsp,
    /// Persistent JSON-lines worker used by the benchmark harness and simple clients.
    Worker,
}
fn main() {
    if let Err(e) = run() {
        eprintln!("{e:#}");
        std::process::exit(1);
    }
}
fn run() -> Result<()> {
    match Args::parse().command {
        Command::Lsp => lsp::run(),
        Command::Worker => worker(),
        Command::Check { input } => {
            let source = std::fs::read_to_string(&input)?;
            let o = RenderOptions {
                base_dir: input.parent().unwrap_or(std::path::Path::new(".")).into(),
                ..Default::default()
            };
            let result = render(&source, &o)?;
            println!("{}", serde_json::to_string(&result.diagnostics)?);
            if !result.diagnostics.is_empty() {
                bail!("document has diagnostics");
            }
            Ok(())
        }
        Command::Render {
            input,
            output,
            format,
            standalone,
            base_dir,
            attributes,
            safe_mode,
        } => {
            let mut source = String::new();
            if let Some(path) = &input {
                source = std::fs::read_to_string(path)?;
            } else {
                io::stdin().read_to_string(&mut source)?;
            }
            let o = RenderOptions {
                input_file: input.clone(),
                standalone,
                safe_mode,
                base_dir: base_dir.unwrap_or_else(|| {
                    input
                        .as_ref()
                        .and_then(|p| p.parent())
                        .unwrap_or(std::path::Path::new("."))
                        .into()
                }),
                attributes: attributes
                    .into_iter()
                    .map(|a| {
                        let (k, v) = a.split_once('=').unwrap_or((&a, ""));
                        (k.to_string(), v.to_string())
                    })
                    .collect(),
                ..Default::default()
            };
            let artifacts = export(&source, &format, &o)?;
            if let Some(path) = output {
                if artifacts.len() == 1 && !matches!(format.as_str(), "svg" | "png") {
                    write_atomically(&path, &artifacts[0].1)?;
                } else {
                    std::fs::create_dir_all(&path)?;
                    for (name, bytes) in artifacts {
                        std::fs::write(path.join(name), bytes)?;
                    }
                }
            } else if artifacts.len() == 1 {
                io::stdout().write_all(&artifacts[0].1)?;
            } else {
                bail!("multipage export requires --output directory");
            }
            Ok(())
        }
    }
}
/// Stage a completed single-file export next to its destination before replacement.
fn write_atomically(path: &std::path::Path, bytes: &[u8]) -> Result<()> {
    use std::{
        fs::OpenOptions,
        sync::atomic::{AtomicU64, Ordering},
    };
    static SEQUENCE: AtomicU64 = AtomicU64::new(0);
    let directory = path
        .parent()
        .filter(|p| !p.as_os_str().is_empty())
        .unwrap_or(std::path::Path::new("."));
    let mut pending = None;
    for _ in 0..16 {
        let name = format!(
            ".adoc-export-{}-{}.tmp",
            std::process::id(),
            SEQUENCE.fetch_add(1, Ordering::Relaxed)
        );
        let candidate = directory.join(name);
        match OpenOptions::new()
            .write(true)
            .create_new(true)
            .open(&candidate)
        {
            Ok(file) => {
                pending = Some((candidate, file));
                break;
            }
            Err(error) if error.kind() == std::io::ErrorKind::AlreadyExists => continue,
            Err(error) => return Err(error.into()),
        }
    }
    let (pending, mut file) =
        pending.ok_or_else(|| anyhow::anyhow!("could not create a temporary export file"))?;
    let result = (|| {
        file.write_all(bytes)?;
        file.sync_all()?;
        drop(file);
        std::fs::rename(&pending, path)?;
        Ok(())
    })();
    let _ = std::fs::remove_file(pending);
    result
}

fn worker() -> Result<()> {
    for line in io::stdin().lock().lines() {
        let response = (|| -> Result<serde_json::Value> {
            let v: serde_json::Value = serde_json::from_str(&line?)?;
            let source = v["source"]
                .as_str()
                .ok_or_else(|| anyhow::anyhow!("source required"))?;
            if source.len() > 16 * 1024 * 1024 {
                bail!("source exceeds 16 MiB");
            }
            let options: RenderOptions =
                serde_json::from_value(v.get("options").cloned().unwrap_or(serde_json::json!({})))?;
            let started = Instant::now();
            if v["action"] == "analyze" {
                return Ok(serde_json::to_value(adoc_engine::analyze(
                    source, &options,
                )?)?);
            }
            if v["action"] == "export" {
                let format = v["format"].as_str().unwrap_or("html");
                let artifacts = export(source, format, &options)?;
                return Ok(serde_json::json!({"artifacts":artifacts}));
            }
            let r = render(source, &options)?;
            let nanos = started.elapsed().as_nanos() as u64;
            Ok(
                serde_json::json!({"html":r.html,"title":r.title,"diagnostics":r.diagnostics,"sourceMap":r.source_map,"dependencies":r.dependencies,"nanos":nanos}),
            )
        })();
        let value = response.unwrap_or_else(|e| serde_json::json!({"error":e.to_string()}));
        println!("{value}");
        io::stdout().flush()?;
    }
    Ok(())
}
