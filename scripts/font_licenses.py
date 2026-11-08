#!/usr/bin/env python3
"""Collect the exact embedded font licenses from the locked typst-assets package."""
import json,subprocess,shutil
from pathlib import Path
root=Path(__file__).resolve().parents[1]
metadata=json.loads(subprocess.check_output(['cargo','metadata','--locked','--format-version','1'],cwd=root,text=True,encoding='utf-8'))
package=next(p for p in metadata['packages']if p['name']=='typst-assets')
package_root=Path(package['manifest_path']).parent;source=package_root/'files/fonts';destination=root/'FONT_LICENSES';destination.mkdir(exist_ok=True)
found=[]
for name in ('NOTICE','LICENSE'):
 file=package_root/name
 if file.exists():shutil.copy2(file,destination/('typst-assets-'+name));found.append(str(file))
for file in source.rglob('*'):
 if file.is_file()and any(s in file.name.lower()for s in ('license','copying','ofl','notice')):
  target=destination/file.relative_to(source);target.parent.mkdir(parents=True,exist_ok=True);shutil.copy2(file,target);found.append(str(target))
if not found:raise SystemExit('No font licenses found; refusing to package')
print(f'Collected {len(found)} font notices')
