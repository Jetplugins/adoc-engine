#!/usr/bin/env python3
"""Create a self-contained binary archive and checksums from verified release output."""
import argparse,hashlib,json,os,shutil,subprocess,tarfile,tempfile,zipfile,platform,datetime,tomllib
from pathlib import Path
p=argparse.ArgumentParser();p.add_argument('--target',required=True);p.add_argument('--binary',type=Path,required=True);p.add_argument('--output',type=Path,default=Path('dist'));a=p.parse_args()
root=Path(__file__).resolve().parents[1];version=tomllib.loads((root/'Cargo.toml').read_text())['workspace']['package']['version'];name=f'adoc-engine-{version}-{a.target}'
a.output.mkdir(parents=True,exist_ok=True)
notices=root/'THIRD_PARTY_LICENSES.html'
if not notices.exists():raise SystemExit('Generate THIRD_PARTY_LICENSES.html with cargo about first')
with tempfile.TemporaryDirectory()as tmp:
 stage=Path(tmp)/name;stage.mkdir();shutil.copy2(a.binary,stage/a.binary.name)
 for file in ['README.md','LICENSE','THIRD_PARTY.md','THIRD_PARTY_LICENSES.html','COMPATIBILITY.md','CHANGELOG.md','VERIFICATION.md']:shutil.copy2(root/file,stage/file)
 fonts=root/'FONT_LICENSES'
 if not fonts.exists():raise SystemExit('Missing embedded font notices')
 shutil.copytree(fonts,stage/'FONT_LICENSES')
 shutil.copytree(root/'docs',stage/'docs')
 evidence=root/'benchmarks/baselines/release'
 if not evidence.exists():raise SystemExit('Missing verified release performance evidence')
 shutil.copytree(evidence,stage/'benchmarks/baselines/release')
 (stage/'vendor').mkdir();shutil.copy2(root/'vendor/PATCHES.md',stage/'vendor/PATCHES.md')
 metadata={'version':version,'target':a.target,'builtUtc':datetime.datetime.now(datetime.timezone.utc).isoformat(),'buildPlatform':platform.platform(),'rustc':subprocess.check_output(['rustc','--version'],text=True,encoding='utf-8').strip(),'commit':subprocess.check_output(['git','rev-parse','HEAD'],cwd=root,text=True,encoding='utf-8').strip(),'binarySha256':hashlib.sha256(a.binary.read_bytes()).hexdigest()}
 (stage/'BUILD.json').write_text(json.dumps(metadata,indent=2)+'\n')
 if a.target.endswith('windows-msvc'):
  archive=a.output/f'{name}.zip'
  with zipfile.ZipFile(archive,'w',zipfile.ZIP_DEFLATED)as z:
   for file in sorted(stage.rglob('*')):
    if file.is_file():z.write(file,file.relative_to(stage.parent))
 else:
  archive=a.output/f'{name}.tar.gz'
  with tarfile.open(archive,'w:gz')as tar:tar.add(stage,arcname=name)
 digest=hashlib.sha256(archive.read_bytes()).hexdigest();archive.with_name(archive.name+'.sha256').write_text(f'{digest}  {archive.name}\n')
 print(archive)
