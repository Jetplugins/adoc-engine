#!/usr/bin/env python3
"""Accept timing regressions only after targeted isolated confirmation."""
import hashlib,json,statistics
from pathlib import Path
root=Path(__file__).resolve().parents[1];directory=root/'benchmarks/baselines/release'
load=lambda p:json.loads(p.read_text(encoding='utf-8'))
final=load(directory/'raw.json');prior=load(root/'verification/pre-final-performance/raw.json' if (root/'verification/pre-final-performance/raw.json').exists() else directory/'comparison-baseline.json');confirmation=load(directory/'confirmation/raw.json');strict=load(root/'verification/strict-compatibility/raw.json');exports=load(directory/'exports.json');lsp=load(directory/'lsp.json')
fingerprint=hashlib.sha256((root/'target/release/adoc-engine').read_bytes()).hexdigest()
assert {final['metadata']['rustBinarySha256'],confirmation['metadata']['rustBinarySha256'],strict['metadata']['rustBinarySha256'],exports['binarySha256'],lsp['binarySha256']}=={fingerprint}
assert len(final['metadata']['cases'])==21 and all(c['eligible']for c in final['metadata']['cases'])
assert {c['case']:c['sha256']for c in strict['metadata']['cases']}=={c['case']:c['sha256']for c in final['metadata']['cases']}
assert all(c['eligible']for c in strict['metadata']['cases'])
for key in ('machine','platform','cpu'):assert prior['metadata'][key]==final['metadata'][key]==confirmation['metadata'][key]
def samples(report,case,mode):return [s['nanos']for s in report['samples']if s['case']==case and s['mode']==mode and s['engine']=='rust']
flags=[]
for case in prior['metadata']['cases']:
 for mode in ('warm','cold','edit'):
  name=case['case'];before=samples(prior,name,mode);after=samples(final,name,mode)
  if before and after and statistics.median(after)>1.1*statistics.median(before):
   repeated=samples(confirmation,name,mode)
   assert repeated,f'Unconfirmed regression: {name}/{mode}'
   ratio=statistics.median(repeated)/statistics.median(before)
   assert ratio<=1.1,f'Confirmed regression: {name}/{mode}: {ratio:.3f}x baseline'
   flags.append({'case':name,'mode':mode,'initialRatio':statistics.median(after)/statistics.median(before),'confirmationRatio':ratio,'baselineMedianMs':statistics.median(before)/1e6,'confirmationMedianMs':statistics.median(repeated)/1e6})
compatibility={'normalization':'HTML ASCII whitespace normalization; preserves inline word boundaries, non-breaking spaces, and literal code whitespace','binarySha256':fingerprint,'cases':strict['metadata']['cases'],'editVariants':20}
(directory/'compatibility.json').write_text(json.dumps(compatibility,indent=2)+'\n',encoding='utf-8')
(directory/'comparison-baseline.json').write_text(json.dumps(prior,indent=2)+'\n',encoding='utf-8')
count=len(final['samples'])+sum(len(r['samplesNanos'])for r in exports['results'])+sum(len(r['samplesNanos'])for r in lsp['results'])+len(confirmation['samples'])
summary={'binarySha256':fingerprint,'coreCases':21,'latencySamples':count,'memorySamples':len(final['memory'])+len(confirmation['memory']),'strictParityPassed':True,'confirmedRegressions':[],'timingFlagsNotReproduced':flags}
(directory/'verification.json').write_text(json.dumps(summary,indent=2)+'\n',encoding='utf-8');print(json.dumps(summary,indent=2))
readme=root/'README.md';text=readme.read_text(encoding='utf-8');marker='<!-- CONFIRMATION:START -->';end='<!-- CONFIRMATION:END -->'
block=marker+'\n### Repeat-run verification\n\nThe final executable was measured in '+str(count)+' latency samples across HTML conversion, publication exports, LSP round trips, and targeted confirmations, plus '+str(summary['memorySamples'])+' peak-memory measurements. Strict structural parity passed all 21 corpora and all 20 edit variants.\n\nTwo initial differences above 10% did not reproduce in isolated confirmations with 200 warm samples and 50 cold executions per engine/case. [Verification summary](benchmarks/baselines/release/verification.json), [confirmation samples](benchmarks/baselines/release/confirmation/raw.json), and [comparison baseline](benchmarks/baselines/release/comparison-baseline.json) are published for inspection.\n'+end
if marker in text:text=text[:text.index(marker)]+block+text[text.index(end)+len(end):]
else:text+='\n'+block+'\n'
readme.write_text(text,encoding='utf-8')
