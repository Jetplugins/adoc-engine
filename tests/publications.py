#!/usr/bin/env python3
"""Independent publication verification using EPUBCheck and Poppler."""
import argparse,json,re,subprocess,xml.etree.ElementTree as ET,zipfile
from pathlib import Path
p=argparse.ArgumentParser();p.add_argument('binary',type=Path);p.add_argument('--epubcheck',type=Path,required=True);p.add_argument('--output',type=Path,default=Path('verification/publication'));a=p.parse_args();binary=a.binary.resolve();a.output.mkdir(parents=True,exist_ok=True)
source=Path(__file__).with_name('publication.adoc')
def run(command):return subprocess.check_output([str(x)for x in command],stderr=subprocess.STDOUT,text=True,timeout=120)
results={}
for format in ['html','pdf','epub','svg','svg-merged','png','json']:
 output=a.output/('pages-'+format if format in ('svg','png')else 'document.'+('svg'if format=='svg-merged'else format))
 run([binary,'render',source,'--format',format,'--standalone','--output',output]);results[format]='generated'
pdf=a.output/'document.pdf';info=run(['pdfinfo',pdf]);pages=int(re.search(r'^Pages:\s+(\d+)',info,re.M)[1]);assert pages>=2,info
text=run(['pdftotext','-layout',pdf,'-'])
for expected in ['Publication Verification','Nested list one','Nested list two','A paragraph in a cell.','passthrough text','Last page','footnote text']:
 assert expected.lower()in text.lower(),f'PDF omitted {expected!r}'
(a.output/'pdf-text.txt').write_text(text);(a.output/'pdf-info.txt').write_text(info)
subprocess.run(['pdftoppm','-r','90','-png',str(pdf),str(a.output/'pdf-page')],check=True,timeout=120)
results['pdf']={'pages':pages,'textChecks':7,'renderedPages':len(list(a.output.glob('pdf-page-*.png')))}
epub=a.output/'document.epub';epublog=run(['java','-jar',a.epubcheck,epub]);(a.output/'epubcheck.txt').write_text(epublog);results['epub']='EPUBCheck 5.4.0 passed'
with zipfile.ZipFile(epub)as z:
 assert z.infolist()[0].filename=='mimetype'and z.infolist()[0].compress_type==0
 for name in z.namelist():
  if name.endswith(('.xhtml','.opf','.xml')):ET.fromstring(z.read(name))
for file in list((a.output/'pages-svg').glob('*.svg'))+[a.output/'document.svg']:ET.parse(file)
assert len(list((a.output/'pages-svg').glob('*.svg')))==pages
assert len(list((a.output/'pages-png').glob('*.png')))==pages
diagram_source='@startuml\nAlice -> Bob\n@enduml'
svg='<svg xmlns="http://www.w3.org/2000/svg" width="160" height="50"><rect x="1" y="1" width="158" height="48" fill="lightblue" stroke="black"/><text x="15" y="30" font-size="18">Alice to Bob</text></svg>'
for format in ['epub','pdf']:
 request={'source':'= Supplied diagram\n\n[plantuml]\n----\n'+diagram_source+'\n----','action':'export','format':format,'options':{'diagrams':{diagram_source:svg}}}
 response=json.loads(subprocess.check_output([str(binary),'worker'],input=json.dumps(request)+'\n',text=True,timeout=120))
 assert 'error'not in response,response
 file=a.output/('diagram.'+format);file.write_bytes(bytes(response['artifacts'][0][1]))
 if format=='epub':(a.output/'diagram-epubcheck.txt').write_text(run(['java','-jar',a.epubcheck,file]))
 else:subprocess.run(['pdftoppm','-r','90','-png',str(file),str(a.output/'diagram-page')],check=True,timeout=120)
results['suppliedDiagrams']='EPUBCheck passed; PDF rendered'
(a.output/'results.json').write_text(json.dumps(results,indent=2)+'\n');print(json.dumps(results,indent=2))
