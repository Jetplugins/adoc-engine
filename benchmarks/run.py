#!/usr/bin/env python3
"""Pinned, sequential HTML benchmark. Requires npm ci and Ruby Asciidoctor 2.0.26.
Worker conversion timing excludes JSON transport; cold timing includes process startup.
All eligible results must pass normalized HTML structural parity against Ruby.
"""
import argparse, csv, hashlib, json, math, os, platform, random, statistics, subprocess, time, re, select, datetime
from pathlib import Path
from html.parser import HTMLParser
ROOT=Path(__file__).resolve().parent
class Canonical(HTMLParser):
    def __init__(self): super().__init__(convert_charrefs=True); self.tokens=[]; self.literal=0; self.tags=[]
    def handle_starttag(self,tag,attrs):
        self.tokens.append(('open',tag,tuple(sorted(attrs))))
        if tag not in ('img','br','hr','input','meta','link','col','wbr'):self.tags.append(tag)
        if tag in ('pre','code'): self.literal+=1
    def handle_startendtag(self,tag,attrs): self.handle_starttag(tag,attrs); self.handle_endtag(tag)
    def handle_endtag(self,tag):
        self.tokens.append(('close',tag))
        if tag in self.tags:self.tags=self.tags[:len(self.tags)-1-self.tags[::-1].index(tag)]
        if tag in ('pre','code'): self.literal=max(0,self.literal-1)
    def handle_data(self,s):
        text=s if self.literal else re.sub(r'[ \t\r\n\f]+',' ',s)
        inline=any(tag in ('h1','h2','h3','h4','h5','h6','p','span','a','strong','em','b','i','code','sup','sub','li','dt','dd','th','td')for tag in self.tags)
        if text and (self.literal or text.strip() or inline):self.tokens.append(('text',text))
def canonical(s):
    p=Canonical();p.feed(s);return p.tokens
class Worker:
    def __init__(self,command): self.p=subprocess.Popen(command,stdin=subprocess.PIPE,stdout=subprocess.PIPE,text=True,cwd=ROOT)
    def render(self,request):
        self.p.stdin.write(json.dumps(request)+'\n');self.p.stdin.flush()
        if not select.select([self.p.stdout],[],[],120)[0]:
            self.p.kill();raise TimeoutError('worker conversion exceeded 120 seconds')
        line=self.p.stdout.readline()
        if not line:raise RuntimeError('worker exited')
        response=json.loads(line)
        if 'error' in response: raise RuntimeError(response['error'])
        return response
    def close(self):
        self.p.stdin.close()
        try:self.p.wait(timeout=10)
        except subprocess.TimeoutExpired:self.p.kill();self.p.wait()
def main():
    p=argparse.ArgumentParser();p.add_argument('--rust',default=str(ROOT.parent/'target/release/adoc-engine'));p.add_argument('--samples',type=int,default=50);p.add_argument('--warmup',type=int,default=10);p.add_argument('--cold',type=int,default=20);p.add_argument('--output',type=Path,default=ROOT/'results');p.add_argument('--update-readme',action='store_true');p.add_argument('--verify-only',action='store_true');p.add_argument('--cases',help='Comma-separated case names for targeted confirmation');a=p.parse_args()
    if min(a.samples,a.cold)<1 or a.warmup<0:p.error('invalid sample counts')
    commands={'js':['node',str(ROOT/'js-worker.cjs')],'ruby':['ruby',str(ROOT/'ruby-worker.rb')],'rust':[str(Path(a.rust).resolve()),'worker']}
    fixtures={f.stem:f.read_text() for f in sorted((ROOT/'fixtures').glob('*.adoc'))}
    for n in (100,1000,10000):fixtures[f'paragraphs-{n}']='= Benchmark\n\n'+'\n\n'.join(f'Paragraph {i} with *bold* and _italic_ and `code`.' for i in range(n))
    for size in (1024,10240,102400,1048576):
        paragraph='A paragraph with *strong* and _emphasis_ and `code`.\n\n'
        fixtures[f'bytes-{size}']='= Size benchmark\n\n'+paragraph*(size//len(paragraph))
    cases=[(name,{'source':source,'options':{'safeMode':'secure','attributes':{}}})for name,source in fixtures.items()]
    # Include expansion measured with the same directory and safe mode for all engines.
    cases.append(('includes',{'source':(ROOT/'fixtures/includes/main.adoc').read_text(),'options':{'safeMode':'safe','baseDir':str(ROOT/'fixtures/includes')}}))
    cases.append(('include-selection',{'source':(ROOT/'fixtures/includes/selection.adoc').read_text(),'options':{'safeMode':'safe','baseDir':str(ROOT/'fixtures/includes')}}))
    cases.append(('showcase',{'source':(ROOT/'fixtures/showcase/main.adoc').read_text(),'options':{'safeMode':'safe','baseDir':str(ROOT/'fixtures/showcase')}}))
    edit_variants=['= Editing\n\n'+('Base text with *formatting*.\n\n'*1000)+f'Changed line {i}.'for i in range(20)]
    cases.append(('repeated-edits',{'source':edit_variants[0],'options':{'safeMode':'secure','attributes':{}}}))
    if a.cases:
        requested=set(a.cases.split(','));known={name for name,_ in cases}
        if requested-known:p.error('Unknown cases: '+', '.join(sorted(requested-known)))
        cases=[case for case in cases if case[0]in requested]
    raw=[]; parity=[]; memory=[];rng=random.Random(42)
    for name,request in cases:
        outputs={}
        for engine,command in commands.items():
            w=Worker(command)
            try:outputs[engine]=w.render(request)['html']
            finally:w.close()
        eligible=all(canonical(outputs[e])==canonical(outputs['ruby'])for e in commands)
        parity.append({'case':name,'eligible':eligible,'sha256':hashlib.sha256(request['source'].encode()).hexdigest(),'inputBytes':len(request['source'].encode())})
        if not eligible:
            a.output.mkdir(parents=True,exist_ok=True)
            for e,s in outputs.items():(a.output/f'{name}-{e}.html').write_text(s)
            print(f'{name}: semantic mismatch; timings excluded',flush=True);continue
        if a.verify_only:
            if name=='repeated-edits':
                workers={e:Worker(command)for e,command in commands.items()}
                try:
                    for source in edit_variants:
                        converted={e:w.render(request|{'source':source})['html']for e,w in workers.items()}
                        if not all(canonical(converted[e])==canonical(converted['ruby'])for e in commands):raise RuntimeError('strict edit parity mismatch')
                finally:
                    for worker in workers.values():worker.close()
            print(f'{name}: strict structural parity passed',flush=True);continue
        order=list(commands);rng.shuffle(order)
        for e in order:
            w=Worker(commands[e])
            try:
                for _ in range(a.warmup):w.render(request)
                for i in range(a.samples):
                    result=w.render(request)
                    if result['html']!=outputs[e]:raise RuntimeError('unstable output')
                    raw.append({'case':name,'engine':e,'mode':'warm','sample':i,'nanos':result['nanos'],'bytes':len(result['html'].encode())})
            finally:w.close()
            for i in range(a.cold):
                start=time.perf_counter_ns()
                completed=subprocess.run(commands[e],input=json.dumps(request)+'\n',text=True,capture_output=True,cwd=ROOT,check=True,timeout=120)
                elapsed=time.perf_counter_ns()-start
                result=json.loads(completed.stdout)
                if result['html']!=outputs[e]:raise RuntimeError('cold output mismatch')
                raw.append({'case':name,'engine':e,'mode':'cold','sample':i,'nanos':elapsed,'bytes':len(result['html'].encode())})
        if name=='repeated-edits':
            reference=[]
            w=Worker(commands['ruby'])
            try:
                for source in edit_variants:reference.append(canonical(w.render(request|{'source':source})['html']))
            finally:w.close()
            for e in order:
                w=Worker(commands[e])
                try:
                    for _ in range(a.warmup):w.render(request)
                    for i in range(a.samples):
                        variant=i%len(edit_variants);result=w.render(request|{'source':edit_variants[variant]})
                        if canonical(result['html'])!=reference[variant]:raise RuntimeError('edit output mismatch')
                        raw.append({'case':name,'engine':e,'mode':'edit','sample':i,'nanos':result['nanos'],'bytes':len(result['html'].encode())})
                finally:w.close()
        # Measure whole-process peak RSS separately so instrumentation does not affect latency.
        for e,command in commands.items():
            flags=['-l'] if platform.system()=='Darwin' else ['-f','PEAK_RSS_KIB=%M']
            measured=subprocess.run(['/usr/bin/time',*flags,*command],input=json.dumps(request)+'\n',text=True,capture_output=True,cwd=ROOT,check=True,timeout=120)
            pattern=r'(\d+)\s+maximum resident set size' if platform.system()=='Darwin' else r'PEAK_RSS_KIB=(\d+)'
            match=re.search(pattern,measured.stderr)
            if match:memory.append({'case':name,'engine':e,'peakRssBytes':int(match[1])*(1 if platform.system()=='Darwin' else 1024)})
        print(f'{name}: complete',flush=True)
    a.output.mkdir(parents=True,exist_ok=True)
    def version(cmd):return subprocess.check_output(cmd,text=True,cwd=ROOT).strip()
    metadata={'timestampUtc':datetime.datetime.now(datetime.timezone.utc).isoformat(),'platform':platform.platform(),'cpu':subprocess.check_output(['sysctl','-n','machdep.cpu.brand_string'],text=True).strip() if platform.system()=='Darwin' else platform.processor(),'machine':platform.machine(),'python':platform.python_version(),'node':version(['node','--version']),'ruby':version(['ruby','--version']),'asciidoctorRuby':'2.0.26','asciidoctorJs':'3.0.4','rust':version([a.rust,'--version']),'rustBinarySha256':hashlib.sha256(Path(a.rust).read_bytes()).hexdigest(),'commit':subprocess.run(['git','rev-parse','HEAD'],cwd=ROOT,capture_output=True,text=True).stdout.strip(),'options':vars(a)|{'output':str(a.output)},'cases':parity}
    metadata['workingTreeDirty']=bool(subprocess.run(['git','status','--porcelain'],cwd=ROOT,capture_output=True,text=True,check=True).stdout.strip())
    (a.output/'raw.json').write_text(json.dumps({'metadata':metadata,'samples':raw,'memory':memory},indent=2))
    with (a.output/'raw.csv').open('w')as f:
        writer=csv.DictWriter(f,fieldnames=['case','engine','mode','sample','nanos','bytes']);writer.writeheader();writer.writerows(raw)
    report=['# HTML conversion benchmark','',f"Platform: {metadata['platform']}",'','Warm = worker conversion only. Cold = worker startup, one conversion, transport, and exit.','Cases with semantic mismatches are excluded. Peak process RSS is measured separately; browser painting is excluded.','','| Case | Mode | Engine | Median ms | p95 ms | CV | conversions/s | Ruby / engine |','|---|---|---|---:|---:|---:|---:|---:|']
    for name,_ in cases:
        for mode in ('warm','cold','edit'):
            base=[r['nanos']for r in raw if r['case']==name and r['mode']==mode and r['engine']=='ruby']
            for e in commands:
                ns=[r['nanos']for r in raw if r['case']==name and r['mode']==mode and r['engine']==e]
                if ns:
                    med=statistics.median(ns);p95=sorted(ns)[math.ceil(.95*len(ns))-1];cv=statistics.pstdev(ns)/statistics.mean(ns)
                    report.append(f'| {name} | {mode} | {e} | {med/1e6:.3f} | {p95/1e6:.3f} | {cv:.3f} | {1e9/statistics.mean(ns):.1f} | {statistics.median(base)/med:.2f} |')
    report+=['','## Peak process memory','','| Case | Engine | Peak RSS MiB |','|---|---|---:|']
    report += [f"| {m['case']} | {m['engine']} | {m['peakRssBytes']/1048576:.1f} |" for m in memory]
    (a.output/'report.md').write_text('\n'.join(report)+'\n')
    if a.update_readme and not a.verify_only and all(c['eligible']for c in parity):
        section=['## Verified performance','',f"Measured {metadata['timestampUtc']} on {metadata['cpu']} ({metadata['platform']}).",f"Pinned Asciidoctor.js 3.0.4 ({metadata['node']}), Ruby Asciidoctor 2.0.26 ({metadata['ruby']}), and release-built Rust.",f"Each case: {a.warmup} warm-ups, {a.samples} warm samples, {a.cold} cold executions per engine. All {len(parity)} cases passed structural HTML parity against Ruby.",'','Warm median conversion times (milliseconds); excludes IPC and browser painting:','','| Corpus | Input bytes | JS | Ruby | Rust | Ruby / Rust |','|---|---:|---:|---:|---:|---:|']
        for case in parity:
            med={e:statistics.median([s['nanos']for s in raw if s['case']==case['case']and s['mode']=='warm'and s['engine']==e])/1e6 for e in commands}
            section.append(f"| {case['case']} | {case['inputBytes']} | {med['js']:.3f} | {med['ruby']:.3f} | {med['rust']:.3f} | {med['ruby']/med['rust']:.2f}x |")
        section+=['','See [full report](benchmarks/baselines/release/report.md) and [raw samples and metadata](benchmarks/baselines/release/raw.json) for p95, variation, cold startup, repeated edits, peak RSS, and the executable checksum.','Results describe this corpus and machine; they are not universal speed guarantees.','The edit workload measures successive complete conversions in persistent processes; it does not claim incremental parsing.']
        readme=ROOT.parent/'README.md';text=readme.read_text();start='<!-- PERFORMANCE:START -->';end='<!-- PERFORMANCE:END -->';block=start+'\n'+'\n'.join(section)+'\n'+end
        if start in text:text=text[:text.index(start)]+block+text[text.index(end)+len(end):]
        else:text+='\n'+block+'\n'
        readme.write_text(text)
    if not all(c['eligible']for c in parity):raise SystemExit('Compatibility failures: inspect results; no speed claim for excluded cases.')
if __name__=='__main__':main()
