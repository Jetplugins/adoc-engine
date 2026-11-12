#!/usr/bin/env python3
"""Verify native STEM in real HTML, EPUB and paginated exports, without MathJax."""
import argparse
import json
import unicodedata
from pathlib import Path
import subprocess
import tempfile
import xml.etree.ElementTree as ET
import zipfile

ROOT = Path(__file__).resolve().parents[1]
MATH = '{http://www.w3.org/1998/Math/MathML}'


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('binary', type=Path)
    parser.add_argument('--epubcheck', type=Path, required=True)
    args = parser.parse_args()
    binary = args.binary.resolve()
    source = ROOT / 'tests/fixtures/stem/equations.adoc'
    with tempfile.TemporaryDirectory(prefix='adoc-stem-') as directory:
        output = Path(directory)
        for format in ['html', 'epub', 'pdf', 'svg', 'svg-merged', 'png']:
            path = output / ('pages-' + format if format in ('svg', 'png') else 'equations.' + format)
            command = [str(binary), 'render', str(source), '-f', format, '-o', str(path)]
            if format == 'html': command += ['--standalone']
            subprocess.run(command, check=True, capture_output=True)
        html = (output / 'equations.html').read_text()
        assert 'MathJax' not in html and '<mfrac>' in html and '<msqrt>' in html
        assert html.count('<math') == 10, html
        with zipfile.ZipFile(output / 'equations.epub') as archive:
            chapters = [ET.fromstring(archive.read(name)) for name in archive.namelist()
                        if name.endswith('.xhtml') and not name.endswith(('nav.xhtml', 'toc.xhtml'))]
            equations = [node for chapter in chapters for node in chapter.iter(MATH+'math')]
            assert len(equations) == 10
            assert sum(node.get('display') == 'block' for node in equations) == 6
            assert all(node.get('alttext') for node in equations)
            assert any(node.tag == MATH+'mtable' for chapter in chapters for node in chapter.iter())
        checked = subprocess.run(['java', '-jar', str(args.epubcheck.resolve()), str(output/'equations.epub')], text=True, capture_output=True)
        assert checked.returncode == 0, checked.stdout + checked.stderr
        text = subprocess.check_output(['pdftotext', str(output/'equations.pdf'), '-'], text=True)
        assert 'Inline equations' in text and 'Display equations' in text, text
        for command in [r'\frac', r'\sqrt', r'\sum', r'\int', r'\begin', r'\hbar', 'sum_(i=1)']:
            assert command not in text, f'Unrendered equation in PDF: {command}\n{text}'
        normalized = unicodedata.normalize('NFKC', text)
        assert 'α' in normalized and 'β' in normalized, text
        # Notation aliases and positional block overrides must typeset, too.
        for name, attributes, block in [
            ('alias', ':stem: tex', '[stem]'),
            ('override', ':stem: asciimath', '[stem,latexmath]'),
        ]:
            document = f'= Notation check\n{attributes}\n\n{block}\n++++\n' + r'\frac{a}{b}' + '\n++++\n'
            path = output / (name + '.pdf')
            subprocess.run([str(binary), 'render', '-f', 'pdf', '-o', str(path)], input=document, text=True, check=True, capture_output=True)
            extracted = subprocess.check_output(['pdftotext', str(path), '-'], text=True)
            assert r'\frac' not in extracted, extracted
        assert list((output/'pages-png').glob('*.png'))
        for svg in [*(output/'pages-svg').glob('*.svg'), output/'equations.svg-merged']:
            assert ET.parse(svg).getroot().tag.endswith('svg')
        print(json.dumps({'equations': 10, 'inline': 4, 'display': 6, 'html': 'static MathML; no MathJax',
                          'epub': 'EPUBCheck passed', 'pdf': 'typeset math verified by text extraction',
                          'svg': 'XML verified', 'svg-merged': 'XML verified', 'png': 'rendered'}, indent=2))


if __name__ == '__main__':
    main()
