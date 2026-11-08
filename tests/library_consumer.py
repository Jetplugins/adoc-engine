#!/usr/bin/env python3
"""Ensure a downstream library consumer uses the fix without root Cargo patches."""
import os
from pathlib import Path
import subprocess
import tempfile

root=Path(__file__).resolve().parents[1]
with tempfile.TemporaryDirectory(prefix='adoc-engine-consumer-') as directory:
    project=Path(directory)
    dependency=(root/'crates/engine').as_posix().replace('"','\\"')
    manifest='[package]\nname = "adoc-engine-list-regression-consumer"\nversion = "0.0.0"\nedition = "2024"\n[workspace]\n[dependencies]\nadoc-engine = { path = "'+dependency+'" }\n'
    (project/'Cargo.toml').write_text(manifest)
    (project/'src').mkdir()
    (project/'src/main.rs').write_text('''fn main() {
    let source = ". Deploy.\\n+\\nTIP: Keep the previous release.\\n. Promote the verified release.\\n";
    let result = adoc_engine::render(source, &Default::default()).unwrap();
    assert!(result.html.contains("<p>Promote the verified release.</p>"), "{}", result.html);
    assert!(result.diagnostics.is_empty());
    println!("Downstream library consumer preserves the separate list item.");
}
''')
    env=os.environ.copy()
    env.setdefault('CARGO_TARGET_DIR',str(root/'target'))
    subprocess.run(['cargo','run','--offline','--manifest-path',str(project/'Cargo.toml')],check=True,env=env)
