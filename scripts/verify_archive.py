#!/usr/bin/env python3
"""Verify checksums, archive contents and optional runtime isolation after relocation."""
import argparse,hashlib,json,os,subprocess,tarfile,tempfile,zipfile
from pathlib import Path
p=argparse.ArgumentParser();p.add_argument('archive',type=Path,nargs='?');p.add_argument('--expected-commit');p.add_argument('--execute',action='store_true');a=p.parse_args()
if a.archive is None:
 archives=list(Path('dist').glob('*.tar.gz'))+list(Path('dist').glob('*.zip'));assert len(archives)==1,'expected one archive in dist';a.archive=archives[0]
checksum=a.archive.with_name(a.archive.name+'.sha256').read_text(encoding='utf-8').split()[0]
assert hashlib.sha256(a.archive.read_bytes()).hexdigest()==checksum,'archive checksum mismatch'
with tempfile.TemporaryDirectory()as tmp:
 directory=Path(tmp)
 if a.archive.suffix=='.zip':
  with zipfile.ZipFile(a.archive)as z:
   for name in z.namelist():assert not Path(name).is_absolute()and '..'not in Path(name).parts and '\\'not in name
   z.extractall(directory)
 else:
  with tarfile.open(a.archive)as tar:
   assert not any(m.issym()or m.islnk()for m in tar.getmembers())
   tar.extractall(directory,filter='data')
 roots=list(directory.iterdir());assert len(roots)==1;root=roots[0]
 for name in ['README.md','LICENSE','THIRD_PARTY_LICENSES.html','VERIFICATION.md','FONT_LICENSES/typst-assets-NOTICE','docs/protocol.md','benchmarks/baselines/release/verification.json']:
  assert (root/name).is_file(),f'missing {name}'
 build=json.loads((root/'BUILD.json').read_text(encoding='utf-8'))
 if a.expected_commit:assert build['commit']==a.expected_commit
 binary=root/('adoc-engine.exe'if build['target'].endswith('windows-msvc')else'adoc-engine')
 assert hashlib.sha256(binary.read_bytes()).hexdigest()==build['binarySha256'],'binary fingerprint mismatch'
 if a.execute:
  environment=os.environ.copy();environment['PATH']=''
  version=subprocess.check_output([str(binary),'--version'],cwd=root,env=environment,timeout=30).decode('utf-8');assert build['version']in version
  source='= Relocated binary\n\nNo external runtimes are available.\n'
  for format in ['html','pdf','epub','svg-merged','png']:
   output=root/('smoke-'+format)
   subprocess.run([str(binary),'render','--format',format,'--output',str(output)],input=source.encode('utf-8'),stdout=subprocess.PIPE,stderr=subprocess.PIPE,cwd=root,env=environment,check=True,timeout=120)
   if format=='pdf':assert output.read_bytes().startswith(b'%PDF')
   if format=='epub':
    with zipfile.ZipFile(output)as z:assert z.read('mimetype')==b'application/epub+zip'
   if format=='png':assert list(output.glob('*.png'))
  print('Relocated binary passed five-format smoke test with empty PATH')
 print(json.dumps({'archive':a.archive.name,'sha256':checksum,'target':build['target'],'commit':build['commit']},indent=2))
