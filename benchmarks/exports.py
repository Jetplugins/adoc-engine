#!/usr/bin/env python3
"""Publication CLI latency, including process startup, compilation and file writes."""
import argparse,hashlib,json,math,statistics,subprocess,tempfile,time
from pathlib import Path
p=argparse.ArgumentParser();p.add_argument('--rust',type=Path,default=Path('target/release/adoc-engine'));p.add_argument('--samples',type=int,default=10);p.add_argument('--update-readme',action='store_true');a=p.parse_args();binary=a.rust.resolve();root=Path(__file__).resolve().parents[1];results=[]
with tempfile.TemporaryDirectory()as tmp:
 base=Path(tmp)
 for name,source in [('small',(root/'tests/publication.adoc').read_text()),('medium','= Publication benchmark\n:toc:\n\n'+'\n\n'.join(f'== Chapter {i}\n\n'+('A *formatted* paragraph with _emphasis_ and an https://example.org/[external link].\n\n'*5)for i in range(30)))]:
  input=base/(name+'.adoc');input.write_text(source)
  for format in ['html','pdf','epub','svg','svg-merged','png','json']:
   output=base/(name+'-'+format);command=[str(binary),'render',str(input),'--format',format,'--standalone','--output',str(output)]
   values=[]
   for i in range(a.samples+2):
    started=time.perf_counter_ns();subprocess.run(command,check=True,stdout=subprocess.DEVNULL,stderr=subprocess.PIPE,timeout=120);elapsed=time.perf_counter_ns()-started
    if i>=2:values.append(elapsed)
   bytes=sum(f.stat().st_size for f in output.rglob('*')if f.is_file())if output.is_dir()else output.stat().st_size
   row={'corpus':name,'format':format,'inputBytes':len(source.encode()),'sourceSha256':hashlib.sha256(source.encode()).hexdigest(),'outputBytes':bytes,'samplesNanos':values,'medianMs':statistics.median(values)/1e6,'p95Ms':sorted(values)[math.ceil(.95*len(values))-1]/1e6};results.append(row);print(f'{name}/{format}: {row["medianMs"]:.2f} ms',flush=True)
output=root/'benchmarks/baselines/release';output.mkdir(parents=True,exist_ok=True);(output/'exports.json').write_text(json.dumps({'binarySha256':hashlib.sha256(binary.read_bytes()).hexdigest(),'commit':subprocess.check_output(['git','rev-parse','HEAD'],cwd=root,text=True,encoding='utf-8').strip(),'method':'Two warmups, repeated full CLI executions, process startup and filesystem writes included; OS file cache is warm.','results':results},indent=2)+'\n')
if a.update_readme:
 section=['### Publication export performance','',f'End-to-end CLI medians in milliseconds: {a.samples} executions after two warm-ups per corpus/format. These include startup, layout, serialization, and filesystem writes with a warm OS cache; they are measured separately from HTML conversion.','','| Format | Small book | Medium book |','|---|---:|---:|']
 for format in ['html','pdf','epub','svg','svg-merged','png','json']:
  med={r['corpus']:r['medianMs']for r in results if r['format']==format};section.append(f"| {format} | {med['small']:.2f} | {med['medium']:.2f} |")
 section+=['','[Raw publication samples and p95](benchmarks/baselines/release/exports.json).']
 readme=root/'README.md';text=readme.read_text();start='<!-- EXPORT-PERF:START -->';end='<!-- EXPORT-PERF:END -->';block=start+'\n'+'\n'.join(section)+'\n'+end
 if start in text:text=text[:text.index(start)]+block+text[text.index(end)+len(end):]
 else:text+='\n'+block+'\n'
 readme.write_text(text)
