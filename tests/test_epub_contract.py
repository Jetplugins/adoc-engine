"""Prove the independent verifier rejects damaged archives instead of merely parsing XML."""
import tempfile
import unittest
import zipfile
from pathlib import Path
from epub_contract import verify

class ContractTest(unittest.TestCase):
    def make(self, directory, body='<p id="ok">Text</p>', css='p { color: inherit; }'):
        path = Path(directory)/'test.epub'
        with zipfile.ZipFile(path, 'w') as z:
            z.writestr('mimetype', 'application/epub+zip', compress_type=zipfile.ZIP_STORED)
            z.writestr('META-INF/container.xml', '<container><rootfiles><rootfile full-path="EPUB/package.opf"/></rootfiles></container>')
            z.writestr('EPUB/package.opf', '<package xmlns="http://www.idpf.org/2007/opf"><metadata xmlns:dc="http://purl.org/dc/elements/1.1/"><dc:title>Test</dc:title></metadata><manifest><item id="nav" href="nav.xhtml" media-type="application/xhtml+xml" properties="nav"/><item id="c" href="content.xhtml" media-type="application/xhtml+xml"/><item id="css" href="style.css" media-type="text/css"/></manifest><spine><itemref idref="c"/></spine></package>')
            z.writestr('EPUB/nav.xhtml', '<html xmlns="http://www.w3.org/1999/xhtml" xmlns:epub="http://www.idpf.org/2007/ops"><body><nav epub:type="toc"><ol><li><a href="content.xhtml#ok">Test</a></li></ol></nav></body></html>')
            z.writestr('EPUB/content.xhtml', f'<html xmlns="http://www.w3.org/1999/xhtml"><body>{body}</body></html>')
            z.writestr('EPUB/style.css', css)
        return path
    def test_valid_book(self):
        with tempfile.TemporaryDirectory() as d: self.assertEqual(verify(self.make(d))['title'], ['Test'])
    def test_missing_anchor(self):
        with tempfile.TemporaryDirectory() as d, self.assertRaisesRegex(AssertionError, 'missing anchor'): verify(self.make(d, '<p>Missing</p>'))
    def test_missing_image(self):
        with tempfile.TemporaryDirectory() as d, self.assertRaisesRegex(AssertionError, 'missing .*png'): verify(self.make(d, '<p id="ok"><img src="missing.png"/></p>'))
    def test_missing_css_font(self):
        with tempfile.TemporaryDirectory() as d, self.assertRaisesRegex(AssertionError, 'missing .*woff'): verify(self.make(d, css='@font-face { src: url("missing.woff"); }'))
    def test_duplicate_id(self):
        with tempfile.TemporaryDirectory() as d, self.assertRaisesRegex(AssertionError, 'duplicate IDs'): verify(self.make(d, '<p id="ok"/><p id="ok"/>'))
if __name__ == '__main__': unittest.main()
