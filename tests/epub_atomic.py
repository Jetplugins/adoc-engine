#!/usr/bin/env python3
"""A failed CLI export must preserve the existing book and remove staging files."""
import subprocess
import sys
import tempfile
from pathlib import Path
binary = Path(sys.argv[1]).resolve()
with tempfile.TemporaryDirectory() as d:
    directory = Path(d); target = directory/'book.epub'; target.write_bytes(b'existing book')
    failed = subprocess.run([str(binary),'render','-f','epub','-o',str(target)], input='= Broken\n\nSee <<missing>>.', text=True, capture_output=True)
    assert failed.returncode != 0 and target.read_bytes() == b'existing book'
    subprocess.run([str(binary),'render','-f','epub','-a','epub-fonts=none','-o',str(target)], input='= Valid\n\nA book.', text=True, check=True)
    assert target.read_bytes().startswith(b'PK')
    assert not list(directory.glob('.adoc-export-*'))
print('Atomic EPUB replacement and failure preservation passed')
