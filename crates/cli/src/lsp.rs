//! Responsive stdio LSP. Two reusable native processes isolate cancellable work.
use adoc_engine::RenderOptions;
use anyhow::{Result, anyhow, bail};
use serde_json::{Value, json};
use std::{
    collections::HashMap,
    io::{self, BufRead, BufReader, Read, Write},
    process::{Child, ChildStdin, ChildStdout, Command, Stdio},
    sync::{
        Arc, Mutex,
        atomic::{AtomicBool, Ordering},
        mpsc,
    },
    thread,
    time::{Duration, Instant},
};
use url::Url;
#[derive(Clone)]
struct Document {
    version: i64,
    text: String,
}
#[derive(Default)]
struct Control {
    cancelled: AtomicBool,
    child: Mutex<Option<Arc<Mutex<Child>>>>,
}
impl Control {
    fn cancel(&self) {
        self.cancelled.store(true, Ordering::SeqCst);
        if let Some(child) = self.child.lock().unwrap().as_ref() {
            let _ = child.lock().unwrap().kill();
        }
    }
}
struct Pending {
    id: Value,
    uri: String,
    version: Option<i64>,
    method: String,
    params: Value,
    control: Arc<Control>,
    started: Instant,
    options: RenderOptions,
}
struct Job {
    key: String,
    request: Value,
    control: Arc<Control>,
}
enum Event {
    Message(Value),
    ParseError(String),
    Done(String, std::result::Result<Value, String>),
    Closed,
}
struct Worker {
    child: Arc<Mutex<Child>>,
    input: ChildStdin,
    output: BufReader<ChildStdout>,
}
impl Worker {
    fn new() -> Result<Self> {
        let mut child = Command::new(std::env::current_exe()?)
            .arg("worker")
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .stderr(Stdio::inherit())
            .spawn()?;
        let input = child.stdin.take().unwrap();
        let output = BufReader::new(child.stdout.take().unwrap());
        Ok(Self {
            child: Arc::new(Mutex::new(child)),
            input,
            output,
        })
    }
    fn request(&mut self, v: &Value) -> Result<Value> {
        serde_json::to_writer(&mut self.input, v)?;
        self.input.write_all(b"\n")?;
        self.input.flush()?;
        let mut line = String::new();
        if self.output.read_line(&mut line)? == 0 {
            bail!("native worker exited");
        }
        let v: Value = serde_json::from_str(&line)?;
        if let Some(error) = v.get("error") {
            bail!("{}", error.as_str().unwrap_or("worker failed"));
        }
        Ok(v)
    }
}
impl Drop for Worker {
    fn drop(&mut self) {
        if let Ok(mut child) = self.child.lock() {
            let _ = child.kill();
            let _ = child.wait();
        }
    }
}
fn send(out: &mut impl Write, v: Value) -> Result<()> {
    let bytes = serde_json::to_vec(&v)?;
    write!(out, "Content-Length: {}\r\n\r\n", bytes.len())?;
    out.write_all(&bytes)?;
    out.flush()?;
    Ok(())
}
fn error(out: &mut impl Write, id: Value, code: i64, message: &str) -> Result<()> {
    send(
        out,
        json!({"jsonrpc":"2.0","id":id,"error":{"code":code,"message":message}}),
    )
}
fn path(uri: &str) -> Result<std::path::PathBuf> {
    Url::parse(uri)?
        .to_file_path()
        .map_err(|_| anyhow!("expected file URI"))
}
fn uri_for(file: Option<&str>, options: &RenderOptions, root: &str) -> String {
    file.and_then(|f| Url::from_file_path(options.base_dir.join(f)).ok())
        .map(|u| u.to_string())
        .unwrap_or_else(|| root.to_string())
}
fn required<'a>(v: &'a Value, key: &str) -> Result<&'a str> {
    v[key].as_str().ok_or_else(|| anyhow!("{key} required"))
}
fn options(params: &Value, uri: &str, docs: &HashMap<String, Document>) -> Result<RenderOptions> {
    let mut options: RenderOptions =
        serde_json::from_value(params.get("options").cloned().unwrap_or(json!({})))?;
    if !uri.is_empty() {
        let file = path(uri)?;
        if params["options"].get("baseDir").is_none() {
            options.base_dir = file.parent().unwrap_or(std::path::Path::new(".")).into();
        }
        options.input_file = Some(file);
    }
    for (uri, doc) in docs {
        if let Ok(file) = path(uri) {
            options.overlays.insert(file, doc.text.clone());
        }
    }
    options.parser_options()?;
    Ok(options)
}
fn read_messages(tx: mpsc::Sender<Event>) {
    let mut input = io::stdin().lock();
    loop {
        let parsed = (|| -> Result<Option<Value>> {
            let mut length = None;
            let mut total = 0;
            loop {
                let mut line = String::new();
                if input.read_line(&mut line)? == 0 {
                    return Ok(None);
                }
                total += line.len();
                if total > 8192 {
                    bail!("oversized header");
                }
                if line == "\r\n" || line == "\n" {
                    break;
                }
                if let Some((key, value)) = line.split_once(':')
                    && key.eq_ignore_ascii_case("Content-Length")
                {
                    length = Some(value.trim().parse::<usize>()?);
                }
            }
            let length = length.ok_or_else(|| anyhow!("missing Content-Length"))?;
            if length > 16 * 1024 * 1024 {
                bail!("message exceeds 16 MiB");
            }
            let mut bytes = vec![0; length];
            input.read_exact(&mut bytes)?;
            match serde_json::from_slice(&bytes) {
                Ok(v) => Ok(Some(v)),
                Err(e) => {
                    let _ = tx.send(Event::ParseError(e.to_string()));
                    Ok(Some(Value::Null))
                }
            }
        })();
        match parsed {
            Ok(Some(v)) if !v.is_null() => {
                if tx.send(Event::Message(v)).is_err() {
                    return;
                }
            }
            Ok(Some(_)) => {}
            Ok(None) | Err(_) => {
                let _ = tx.send(Event::Closed);
                return;
            }
        }
    }
}
pub fn run() -> Result<()> {
    let (tx, rx) = mpsc::channel();
    let reader_tx = tx.clone();
    thread::spawn(move || read_messages(reader_tx));
    let (job_tx, job_rx) = mpsc::sync_channel::<Job>(32);
    let jobs = Arc::new(Mutex::new(job_rx));
    let mut threads = Vec::new();
    for _ in 0..2 {
        let jobs = jobs.clone();
        let tx = tx.clone();
        threads.push(thread::spawn(move || {
            let mut worker: Option<Worker> = None;
            loop {
                let job = match jobs.lock().unwrap().recv() {
                    Ok(job) => job,
                    Err(_) => break,
                };
                if job.control.cancelled.load(Ordering::SeqCst) {
                    continue;
                }
                let result = (|| -> Result<Value> {
                    if worker.is_none() {
                        worker = Some(Worker::new()?);
                    }
                    let w = worker.as_mut().unwrap();
                    *job.control.child.lock().unwrap() = Some(w.child.clone());
                    if job.control.cancelled.load(Ordering::SeqCst) {
                        bail!("cancelled");
                    }
                    w.request(&job.request)
                })();
                *job.control.child.lock().unwrap() = None;
                if result.is_err() || job.control.cancelled.load(Ordering::SeqCst) {
                    worker = None;
                }
                if tx
                    .send(Event::Done(job.key, result.map_err(|e| e.to_string())))
                    .is_err()
                {
                    break;
                }
            }
        }));
    }
    let mut out = io::stdout().lock();
    let mut docs: HashMap<String, Document> = HashMap::new();
    let mut pending: HashMap<String, Pending> = HashMap::new();
    let mut initialized = false;
    let mut shutdown = false;
    let mut serial = 0u64;
    loop {
        let event = match rx.recv_timeout(Duration::from_millis(100)) {
            Ok(e) => e,
            Err(mpsc::RecvTimeoutError::Timeout) => {
                let expired: Vec<_> = pending
                    .iter()
                    .filter(|(_, p)| p.started.elapsed() > Duration::from_secs(60))
                    .map(|(k, _)| k.clone())
                    .collect();
                for key in expired {
                    if let Some(p) = pending.remove(&key) {
                        p.control.cancel();
                        if !p.id.is_null() {
                            error(&mut out, p.id, -32800, "request timed out")?;
                        }
                    }
                }
                continue;
            }
            Err(_) => break,
        };
        match event {
            Event::ParseError(message) => error(&mut out, Value::Null, -32700, &message)?,
            Event::Closed => break,
            Event::Done(key, result) => {
                if let Some(p) = pending.remove(&key) {
                    if p.version != docs.get(&p.uri).map(|d| d.version) && !p.uri.is_empty() {
                        if !p.id.is_null() {
                            error(&mut out, p.id, -32801, "document changed")?;
                        }
                        continue;
                    }
                    match result {
                        Err(e) => {
                            if !p.id.is_null() {
                                error(&mut out, p.id, -32602, &e)?;
                            }
                        }
                        Ok(value) => {
                            if p.method == "diagnostics" {
                                let diagnostics=value["diagnostics"].as_array().into_iter().flatten().map(|d|{let n=d["line"].as_u64().unwrap_or(1).saturating_sub(1);json!({"range":{"start":{"line":n,"character":0},"end":{"line":n,"character":0}},"severity":2,"source":"adoc-engine","message":d["message"]})}).collect::<Vec<_>>();
                                send(
                                    &mut out,
                                    json!({"jsonrpc":"2.0","method":"textDocument/publishDiagnostics","params":{"uri":p.uri,"version":p.version,"diagnostics":diagnostics}}),
                                )?;
                            } else {
                                match finish(&p, value) {
                                    Ok(result) => send(
                                        &mut out,
                                        json!({"jsonrpc":"2.0","id":p.id,"result":result}),
                                    )?,
                                    Err(e) => error(&mut out, p.id, -32602, &e.to_string())?,
                                }
                            }
                        }
                    }
                }
            }
            Event::Message(v) => {
                let id = v.get("id").cloned().unwrap_or(Value::Null);
                let method = v["method"].as_str().unwrap_or("");
                let params = v.get("params").cloned().unwrap_or(json!({}));
                if v["jsonrpc"] != "2.0" || method.is_empty() {
                    error(&mut out, id, -32600, "invalid JSON-RPC request")?;
                    continue;
                }
                if method == "exit" {
                    break;
                }
                if method == "$/cancelRequest" {
                    let key = params["id"].to_string();
                    if let Some(p) = pending.remove(&key) {
                        p.control.cancel();
                        error(&mut out, p.id, -32800, "request cancelled")?;
                    }
                    continue;
                }
                if method == "initialize" && !initialized {
                    initialized = true;
                    send(
                        &mut out,
                        json!({"jsonrpc":"2.0","id":id,"result":{"capabilities":{"positionEncoding":"utf-16","textDocumentSync":2,"documentSymbolProvider":true,"completionProvider":{"triggerCharacters":["{","<",":"]},"hoverProvider":true,"definitionProvider":true,"experimental":{"asciidoc":{"protocolVersion":1,"render":true,"export":true,"overlays":true}}},"serverInfo":{"name":"adoc-engine","version":env!("CARGO_PKG_VERSION")}}}),
                    )?;
                    continue;
                }
                if !initialized {
                    if !id.is_null() {
                        error(&mut out, id, -32002, "server not initialized")?;
                    }
                    continue;
                }
                if shutdown {
                    if !id.is_null() {
                        error(&mut out, id, -32600, "server shut down")?;
                    }
                    continue;
                }
                if method == "initialized" {
                    continue;
                }
                if method == "shutdown" {
                    shutdown = true;
                    for (_, p) in pending.drain() {
                        p.control.cancel();
                        if !p.id.is_null() {
                            error(&mut out, p.id, -32800, "server shutting down")?;
                        }
                    }
                    send(&mut out, json!({"jsonrpc":"2.0","id":id,"result":null}))?;
                    continue;
                }
                let root_uri = params["textDocument"]["uri"]
                    .as_str()
                    .unwrap_or("")
                    .to_string();
                let handled = (|| -> Result<()> {
                    match method {
                        "textDocument/didOpen" => {
                            let d = &params["textDocument"];
                            let uri = required(d, "uri")?;
                            path(uri)?;
                            let text = required(d, "text")?.to_string();
                            if text.len() > 16 * 1024 * 1024 {
                                bail!("document too large");
                            }
                            let version = d["version"]
                                .as_i64()
                                .ok_or_else(|| anyhow!("version required"))?;
                            docs.insert(uri.into(), Document { version, text });
                        }
                        "textDocument/didChange" => {
                            let d = docs
                                .get_mut(&root_uri)
                                .ok_or_else(|| anyhow!("document not open"))?;
                            let version = params["textDocument"]["version"]
                                .as_i64()
                                .ok_or_else(|| anyhow!("version required"))?;
                            if version <= d.version {
                                return Ok(());
                            }
                            let mut text = d.text.clone();
                            for change in params["contentChanges"]
                                .as_array()
                                .ok_or_else(|| anyhow!("contentChanges required"))?
                            {
                                apply_change(&mut text, change)?;
                            }
                            if text.len() > 16 * 1024 * 1024 {
                                bail!("document too large");
                            }
                            *d = Document { version, text };
                        }
                        "textDocument/didClose" => {
                            docs.remove(&root_uri);
                            send(
                                &mut out,
                                json!({"jsonrpc":"2.0","method":"textDocument/publishDiagnostics","params":{"uri":root_uri,"diagnostics":[]}}),
                            )?;
                        }
                        "asciidoc/render"
                        | "asciidoc/export"
                        | "textDocument/completion"
                        | "textDocument/hover"
                        | "textDocument/definition"
                        | "textDocument/documentSymbol" => {}
                        _ => {
                            if !id.is_null() {
                                error(&mut out, id.clone(), -32601, "method not supported")?;
                            }
                            return Ok(());
                        }
                    }
                    let changed = matches!(
                        method,
                        "textDocument/didOpen" | "textDocument/didChange" | "textDocument/didClose"
                    );
                    if changed {
                        let stale: Vec<_> = pending
                            .iter()
                            .filter(|(_, p)| !p.uri.is_empty())
                            .map(|(k, _)| k.clone())
                            .collect();
                        for key in stale {
                            if let Some(p) = pending.remove(&key) {
                                p.control.cancel();
                                if !p.id.is_null() {
                                    error(&mut out, p.id, -32801, "workspace changed")?;
                                }
                            }
                        }
                    }
                    if method == "textDocument/didClose" {
                        return Ok(());
                    }
                    let known = matches!(
                        method,
                        "asciidoc/render"
                            | "asciidoc/export"
                            | "textDocument/completion"
                            | "textDocument/hover"
                            | "textDocument/definition"
                            | "textDocument/documentSymbol"
                    ) || changed;
                    if !known {
                        return Ok(());
                    }
                    let opts = options(&params, &root_uri, &docs)?;
                    let source = params["text"]
                        .as_str()
                        .map(str::to_owned)
                        .or_else(|| docs.get(&root_uri).map(|d| d.text.clone()))
                        .ok_or_else(|| anyhow!("text or an open textDocument required"))?;
                    let action = if method == "asciidoc/export" {
                        "export"
                    } else if changed || method == "asciidoc/render" {
                        "render"
                    } else {
                        "analyze"
                    };
                    serial += 1;
                    let key = if id.is_null() {
                        format!("notification-{serial}")
                    } else {
                        id.to_string()
                    };
                    if pending.contains_key(&key) {
                        bail!("duplicate request ID");
                    }
                    let control = Arc::new(Control::default());
                    let job = Job {
                        key: key.clone(),
                        control: control.clone(),
                        request: json!({"source":source,"options":opts,"action":action,"format":params["format"]}),
                    };
                    job_tx
                        .try_send(job)
                        .map_err(|_| anyhow!("server busy; retry request"))?;
                    pending.insert(
                        key,
                        Pending {
                            id: id.clone(),
                            uri: root_uri.clone(),
                            version: docs.get(&root_uri).map(|d| d.version),
                            method: if changed {
                                "diagnostics".into()
                            } else {
                                method.into()
                            },
                            params: params.clone(),
                            control,
                            started: Instant::now(),
                            options: opts,
                        },
                    );
                    Ok(())
                })();
                if let Err(e) = handled {
                    if !id.is_null() {
                        error(&mut out, id, -32602, &e.to_string())?;
                    } else {
                        send(
                            &mut out,
                            json!({"jsonrpc":"2.0","method":"window/logMessage","params":{"type":2,"message":e.to_string()}}),
                        )?;
                    }
                }
            }
        }
    }
    for (_, p) in pending {
        p.control.cancel();
    }
    drop(job_tx);
    for t in threads {
        let _ = t.join();
    }
    if !shutdown {
        bail!("LSP exited without shutdown");
    }
    Ok(())
}
fn finish(p: &Pending, mut value: Value) -> Result<Value> {
    if p.method == "asciidoc/render" {
        value["version"] = json!(p.version);
        return Ok(value);
    }
    if p.method == "asciidoc/export" {
        let dir = std::path::PathBuf::from(required(&p.params, "outputDirectory")?);
        let artifacts: Vec<(String, Vec<u8>)> = serde_json::from_value(value["artifacts"].clone())?;
        std::fs::create_dir_all(&dir)?;
        let mut paths = Vec::new();
        for (name, bytes) in artifacts {
            let target = dir.join(name);
            std::fs::write(&target, bytes)?;
            paths.push(target);
        }
        return Ok(json!({"artifacts":paths}));
    }
    let symbols = value["symbols"]
        .as_array()
        .ok_or_else(|| anyhow!("missing semantic symbols"))?;
    if p.method == "textDocument/completion" {
        return Ok(json!(
            symbols
                .iter()
                .map(|s| json!({"label":s["name"],"detail":s["detail"],"kind":18}))
                .collect::<Vec<_>>()
        ));
    }
    if p.method == "textDocument/documentSymbol" {
        return Ok(json!(
            symbols
                .iter()
                .filter(|s| uri_for(s["file"].as_str(), &p.options, &p.uri) == p.uri
                    && s["kind"] == "section")
                .map(|s| {
                    let n = s["line"].as_u64().unwrap_or(0);
                    let range =
                        json!({"start":{"line":n,"character":0},"end":{"line":n,"character":0}});
                    json!({"name":s["detail"],"kind":3,"range":range,"selectionRange":range})
                })
                .collect::<Vec<_>>()
        ));
    }
    let text = p
        .options
        .input_file
        .as_ref()
        .and_then(|f| p.options.overlays.get(f))
        .ok_or_else(|| anyhow!("open document required"))?;
    let line = text
        .lines()
        .nth(
            p.params["position"]["line"]
                .as_u64()
                .ok_or_else(|| anyhow!("line required"))? as usize,
        )
        .ok_or_else(|| anyhow!("invalid line"))?;
    let col = p.params["position"]["character"]
        .as_u64()
        .ok_or_else(|| anyhow!("character required"))? as usize;
    let byte = utf16_offset(line, col).ok_or_else(|| anyhow!("invalid UTF-16 position"))?;
    let start = line[..byte]
        .rfind(|c: char| !c.is_alphanumeric() && c != '_' && c != '-')
        .map_or(0, |n| n + 1);
    let end = line[byte..]
        .find(|c: char| !c.is_alphanumeric() && c != '_' && c != '-')
        .map_or(line.len(), |n| byte + n);
    let Some(s) = symbols
        .iter()
        .find(|s| s["name"].as_str() == Some(&line[start..end]))
    else {
        return Ok(Value::Null);
    };
    if p.method == "textDocument/hover" {
        return Ok(
            json!({"contents":{"kind":"plaintext","value":format!("{}: {}",s["name"].as_str().unwrap_or(""),s["detail"].as_str().unwrap_or(""))}}),
        );
    }
    let n = s["line"].as_u64().unwrap_or(0);
    Ok(
        json!({"uri":uri_for(s["file"].as_str(),&p.options,&p.uri),"range":{"start":{"line":n,"character":0},"end":{"line":n,"character":0}}}),
    )
}
fn utf16_offset(s: &str, col: usize) -> Option<usize> {
    let mut n = 0;
    for (i, c) in s.char_indices() {
        if n == col {
            return Some(i);
        }
        n += c.len_utf16();
        if n > col {
            return None;
        }
    }
    if n == col { Some(s.len()) } else { None }
}
fn position(text: &str, p: &Value) -> Result<usize> {
    let line = p["line"]
        .as_u64()
        .ok_or_else(|| anyhow::anyhow!("line required"))? as usize;
    let col = p["character"]
        .as_u64()
        .ok_or_else(|| anyhow::anyhow!("character required"))? as usize;
    let mut offset = 0;
    for (i, s) in text.split_inclusive('\n').enumerate() {
        if i == line {
            return utf16_offset(s.trim_end_matches(['\r', '\n']), col)
                .map(|b| offset + b)
                .ok_or_else(|| anyhow::anyhow!("invalid UTF-16 position"));
        }
        offset += s.len();
    }
    if col == 0 && line == text.bytes().filter(|b| *b == b'\n').count() {
        Ok(text.len())
    } else {
        bail!("invalid line")
    }
}
fn apply_change(text: &mut String, change: &Value) -> Result<()> {
    let replacement = change["text"]
        .as_str()
        .ok_or_else(|| anyhow::anyhow!("text required"))?;
    if let Some(range) = change.get("range") {
        let start = position(text, &range["start"])?;
        let end = position(text, &range["end"])?;
        if start > end {
            bail!("reversed edit");
        }
        text.replace_range(start..end, replacement);
    } else {
        *text = replacement.into();
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn edits_are_unicode_safe() {
        let mut text = "a😀b\n".to_string();
        apply_change(&mut text,&json!({"range":{"start":{"line":0,"character":1},"end":{"line":0,"character":3}},"text":"x"})).unwrap();
        assert_eq!(text, "axb\n");
        assert!(utf16_offset("😀", 1).is_none());
    }
}
