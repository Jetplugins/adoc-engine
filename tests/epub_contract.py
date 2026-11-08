#!/usr/bin/env python3
"""Independent EPUB archive/link/metadata checks, shared with the Kotlin exporter tests."""
import argparse
import hashlib
import json
import posixpath
import re
import subprocess
import urllib.parse
import xml.etree.ElementTree as ET
import zipfile
from pathlib import Path

NS = {'o': 'http://www.idpf.org/2007/opf', 'd': 'http://purl.org/dc/elements/1.1/',
      'h': 'http://www.w3.org/1999/xhtml', 'm': 'http://www.w3.org/1998/Math/MathML',
      'n': 'http://www.daisy.org/z3986/2005/ncx/'}
EPUB_TYPE = '{http://www.idpf.org/2007/ops}type'

def verify(path, strict=True):
    link_errors = []
    with zipfile.ZipFile(path) as archive:
        infos = archive.infolist()
        assert infos[0].filename == 'mimetype' and infos[0].compress_type == zipfile.ZIP_STORED
        assert not infos[0].extra and archive.read('mimetype') == b'application/epub+zip'
        assert len({i.filename for i in infos}) == len(infos), 'duplicate archive paths'
        assert all(not i.filename.startswith('/') and '..' not in Path(i.filename).parts for i in infos)
        names = set(archive.namelist())
        xml = {name: ET.fromstring(archive.read(name)) for name in names if name.endswith(('.xml', '.xhtml', '.opf', '.ncx', '.svg'))}
        container = xml['META-INF/container.xml']
        rootfile = container.find('{*}rootfiles/{*}rootfile').attrib['full-path']
        package = xml[rootfile]
        base = posixpath.dirname(rootfile)
        manifest = {e.attrib['id']: e for e in package.findall('o:manifest/o:item', NS)}
        def target(origin, href):
            parsed = urllib.parse.urlsplit(href)
            if parsed.scheme or parsed.netloc:
                assert parsed.scheme.lower() not in ('file', 'javascript'), (origin, href)
                return None
            file = posixpath.normpath(posixpath.join(posixpath.dirname(origin), urllib.parse.unquote(parsed.path))) if parsed.path else origin
            assert file in names, f'missing {href!r} from {origin}'
            if parsed.fragment:
                assert file in xml, f'fragment targets a non-XML asset: {href}'
                fragment = urllib.parse.unquote(parsed.fragment)
                if not any(e.attrib.get('id') == fragment for e in xml[file].iter()):
                    message = f'missing anchor {href!r} from {origin}'
                    if strict: raise AssertionError(message)
                    link_errors.append(message)
            return file
        for item in manifest.values():
            target(rootfile, item.attrib['href'])
        spine = [e.attrib['idref'] for e in package.findall('o:spine/o:itemref', NS)]
        assert spine and all(e in manifest for e in spine)
        navs = [e for e in manifest.values() if 'nav' in e.attrib.get('properties', '').split()]
        assert len(navs) == 1
        nav_path = target(rootfile, navs[0].attrib['href'])
        assert xml[nav_path].find('.//h:nav[@' + EPUB_TYPE + '="toc"]', NS) is not None
        for name, root in xml.items():
            if name.endswith('.xhtml'):
                assert root.tag == '{http://www.w3.org/1999/xhtml}html', name
                ids = [e.attrib['id'] for e in root.iter() if 'id' in e.attrib]
                assert len(ids) == len(set(ids)), f'duplicate IDs in {name}'
                for e in root.iter():
                    tag = e.tag.rsplit('}', 1)[-1]
                    if tag in ('a', 'link', 'img', 'audio', 'video', 'source', 'image', 'use'):
                        for attribute in ('href', 'src', 'poster', '{http://www.w3.org/1999/xlink}href'):
                            if e.attrib.get(attribute): target(name, e.attrib[attribute])
            elif name.endswith('.ncx'):
                for e in root.findall('.//n:content', NS): target(name, e.attrib['src'])
        for name in names:
            if name.endswith('.css'):
                css = archive.read(name).decode('utf-8')
                for href in re.findall(r'url\(\s*[\'"]?([^\)\'"\s]+)', css, re.I): target(name, href)
                for href in re.findall(r'@import\s+[\'"]([^\'"]+)', css, re.I): target(name, href)
        metadata = package.find('o:metadata', NS)
        def values(key): return [e.text or '' for e in metadata.findall('d:' + key, NS)]
        roles = {e.attrib.get('refines', '').lstrip('#'): e.text for e in metadata.findall('o:meta', NS) if e.attrib.get('property') == 'role'}
        creators = [e.text for e in metadata.findall('d:creator', NS) if roles.get(e.attrib.get('id')) != 'mfr']
        contents = [xml[posixpath.join(base, manifest[e].attrib['href'])] for e in spine]
        headings = [''.join(e.itertext()).strip() for content in contents for e in content.iter() if e.tag in {f'{{{NS["h"]}}}h{i}' for i in range(1, 7)}]
        texts = [''.join(content.itertext()) for content in contents]
        types = [e.attrib.get(EPUB_TYPE, '') for content in contents for e in content.iter()]
        result = {'linkErrors': link_errors, 'title': values('title'), 'language': values('language'), 'authors': creators,
            'publisher': values('publisher'), 'rights': values('rights'), 'description': values('description'),
            'subjects': values('subject'), 'date': values('date'), 'source': values('source'),
            'series': [e.text for e in metadata.findall('o:meta', NS) if e.attrib.get('property') == 'belongs-to-collection'],
            'spineDocuments': [manifest[e].attrib['href'] for e in spine], 'headings': headings,
            'coverImages': sum('cover-image' in e.attrib.get('properties', '').split() for e in manifest.values()),
            'fonts': sum('font' in e.attrib['media-type'] or e.attrib['media-type'] == 'application/vnd.ms-opentype' for e in manifest.values()),
            'math': sum(len(c.findall('.//m:math', NS)) for c in contents),
            'noterefs': sum('noteref' in t.split() for t in types), 'footnotes': sum('footnote' in t.split() for t in types),
            'media': sum(len(c.findall('.//h:audio', NS)) + len(c.findall('.//h:video', NS)) for c in contents),
            'highlighted': any(any(('keyword' in e.attrib.get('class', '') or 'storage' in e.attrib.get('class', '')) or 'hljs-' in e.attrib.get('class', '') or e.attrib.get('class') in ('k', 'kd', 'kt') for e in c.iter()) for c in contents),
            'checkedItems': sum(t.count('✓') for t in texts), 'uncheckedItems': sum(t.count('❏') for t in texts),
            'unicodeCallouts': sum(t.count('①') for t in texts),
            'unresolvedXrefLabels': sum(1 for c in contents for e in c.iter() if e.tag == f'{{{NS["h"]}}}a' and 'xref' in e.attrib.get('class', '').split() and ''.join(e.itertext()).strip() == '[reference]'),
            'unicodePreserved': all(any(word in t for t in texts) for word in ('café', 'Ελληνικά', 'Кириллица')),
            'archiveEntries': len(names), 'sha256': hashlib.sha256(path.read_bytes()).hexdigest(), 'outputBytes': path.stat().st_size}
        return result

def check_case(result, case, reference=False):
    if case in ('book', 'depth', 'custom', 'reader-fonts'):
        assert result['title'] == ['Native EPUB Handbook'], result
        assert result['language'] == ['en']
        assert {'Jane Writer', 'John Reader'} <= set(result['authors']), result
        for field, expected in [('publisher', 'Example Press'), ('rights', 'Copyright 2026 Example Authors'), ('description', 'A portable & accessible book.'), ('date', '2026-11-09'), ('source', 'https://example.org/books/handbook')]:
            if reference and field in ('description', 'date'): continue # reference escapes twice and normalizes local publication date to UTC
            assert expected in result[field], (field, result[field])
        assert ({'EPUB', 'Rust', 'Kotlin'} if reference else {'EPUB', 'books, publishing', 'Rust', 'Kotlin'}) <= set(result['subjects']), result['subjects']
        assert result['series'] == ['Native Publishing'], result['series']
        assert result['coverImages'] == 1 and result['math'] >= 3 and result['noterefs'] >= 2 and result['footnotes'] >= 1, result
        assert result['unicodePreserved'] and result['highlighted'], result
        if not reference: assert result['checkedItems'] >= 1 and result['uncheckedItems'] >= 1 and result['unicodeCallouts'] >= 2 and result['unresolvedXrefLabels'] == 0, result
        if case == 'custom': assert result['spineDocuments'].index('front-matter-01.xhtml') < result['spineDocuments'].index('front-matter-02.xhtml')
        if case == 'reader-fonts': assert result['fonts'] == 0, result
    elif case == 'article':
        content = [n for n in result['spineDocuments'] if n not in ('nav.xhtml', 'toc.xhtml')]
        assert len(content) == 1, result
    elif case == 'media': assert result['media'] == 2, result

def main():
    p = argparse.ArgumentParser(description=__doc__)
    p.add_argument('directory', type=Path); p.add_argument('--epubcheck', type=Path); p.add_argument('--output', type=Path)
    args = p.parse_args(); records = {}
    for epub in sorted(args.directory.glob('*.epub')):
        record = verify(epub); check_case(record, epub.stem)
        if args.epubcheck:
            process = subprocess.run(['java', '-jar', str(args.epubcheck), str(epub)], text=True, stdout=subprocess.PIPE, stderr=subprocess.STDOUT, timeout=120)
            epub.with_suffix('.epubcheck.txt').write_text(process.stdout)
            assert process.returncode == 0, process.stdout
            record['epubcheck'] = 'passed'
        records[epub.stem] = record
    assert records, 'no EPUBs found'
    if args.output: args.output.write_text(json.dumps(records, indent=2, ensure_ascii=False) + '\n')
    print(json.dumps(records, indent=2, ensure_ascii=False))
if __name__ == '__main__': main()
